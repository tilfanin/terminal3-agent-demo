# Bugs found while completing the Terminal3 ADK quickstart + contract walkthrough

All findings verified against the live T3N sandbox (`@terminal3/t3n-sdk@4.40.0`,
Node v24.14.0, testnet node) during an independent zero-capital integration on
2026-08-17/18. Each item records what was actually hit, in what order, and how it
was worked around.

---

## Bug 1 — `agent card/set-card` is rejected by the sandbox: `missing field script_name`

**Severity:** High — blocks publishing an agent card to the registry.

**Reproduction:** with a valid sandbox API key, run
`t3n agent host-card --file agent-card.json --env sandbox` or
`t3n agent set-card --uri <node>/api/agent-card/<did> --env sandbox`.

**Expected:** agent card stored + published; agent URI registered in the registry.

**Actual:**
```
RPC Error: Invalid action request: missing field script_name at line 1 column N
```
Different RPC request ids on each run (`6b3e2ea5-...`, `42818e11-...`, `6c184b75-...`),
so it is a request-schema rejection, not a transient error.

**Analysis:** the sandbox action-request validator requires a `script_name` field
that the `@terminal3/t3n-sdk@4.40.0` card/set-card code path never sends. SDK/server
schema mismatch. `agent registry <did>` returns `agent: (none)` afterwards — the
agent URI stays unregistered.

**Workaround:** none in the SDK. `t3n agent create-card --out agent-card.json`
(offline scaffold) works; only hosting/publishing to the sandbox fails.

---

## Bug 2 — The attestation endpoint stalls mid-stream on some networks

**Severity:** Medium — blocks the secure handshake path.

**Reproduction:** run `fetchTrustedManifest("sandbox")` then `client.handshake()`.

**Expected:** handshake completes; the ML-KEM key exchange fetches the TDX v4
attestation quote (~37 KB).

**Actual:** `GET /status?attestation=1` downloads at ~330 B/s and the connection is
killed mid-stream (`TypeError: terminated` / `ECONNRESET`) on some networks. Every
other endpoint works fine: `GET /api/trust-manifest` (518 B), `GET /status`
(2.6 KB), `POST /api/rpc`, `POST /api/invoke`.

**Workaround:** `trustAnchor: { unsafe_trust_server: true }` skips the attestation
fetch entirely → handshake + authenticate + getUsage complete. Safe for sandbox
test credits only; never against a production node with real funds. Stateless
alternative: `invoke()` — single `POST /api/invoke` with an `X-T3N-Api-Key` header,
no handshake/session/attestation at all.

---

## Bug 3 — `tenant.me()` throws at "Set Up Development Environment"

**Severity:** Low — docs reference a method the shipped SDK does not expose.

**Reproduction:** follow the walkthrough; call `await tenant.me()`.

**Expected:** returns the current tenant profile.

**Actual:** `TypeError: tenant.me is not a function` — the method is absent from
`TenantClient` in `@terminal3/t3n-sdk@4.40.0` though the docs call it.

**Workaround:** skip the verification step; continue with the rest of the walkthrough.

---

## Contract-development findings (hit while building `xagent-deals`)

These came out of actually writing a Rust WASM contract for the Trinity TEE, not
from reading docs. Each one cost real debugging time against the live sandbox.

### Finding 1 — host-import paths: `crate::host::...`, not the compiler's suggestion
The compiler's suggested fix points at `crate::exports::z::...`; the real paths are
`crate::host::{interfaces::kv_store, tenant::tenant_context}`.

### Finding 2 — `kv_store::get/put` take `&[u8]` keys
Pass `id.as_bytes()`, not `Vec<u8>`. Type mismatch fails the WIT bindgen layer.

### Finding 3 — `kv_store::scan` needs `limit > 0`
`scan(map, &[]..&[0xFF], 0)` is rejected; a whole-map enumeration needs a positive
limit.

### Finding 4 — the host KV runtime rolls back writes from a call that returns `Err`
A ledger append that succeeds, followed by an early `Err` return, silently vanishes.
Expected business-logic denials must be returned as `Ok(response)` with a
`reason`/`denied` field — reserve `Err` for genuine infrastructure faults. A mock
in-memory harness has no such rollback, so local tests can all pass and the real
ledger still behaves differently.

### Finding 5 — `pii_did` must be set explicitly on metered/delegated calls
Omitting it silently defaults to the agent's own DID → the node looks up
`AGENT_AUTH_MAP[agent_did]` (empty) instead of `AGENT_AUTH_MAP[tenant_did]` →
`host/http.egress_denied`, which reads like an allowlist misconfiguration. It is a
wrong lookup subject. `getAgentAuth()` read-backs look perfect the whole time.

### Finding 6 — cap response sizes before generic JSON decode inside the enclave
Decoding an unbounded external body into `serde_json::Value` can trigger a
WASM-level allocation/stack-pressure trap that aborts below `Result`/`?` handling
and surfaces as a bare `Internal error`. Enforce a hard byte ceiling (e.g. 256 KiB)
before deserialization.

### Finding 7 — a contract version bump mints a new `contract_id`
Re-registering a new version allocates a new numeric id; existing KV maps stay
ACL'd to the old one. Re-point each map's ACL at the new id on every bump.

### Finding 8 — x402 prices arrive as raw on-chain base-unit integers
A price field comes back as USDC's 6-decimal base-unit integer with no dollar
figure alongside. Passing it straight through can produce amounts off by orders of
magnitude. Normalize by the currency's decimal base at inspection time.

### Finding 9 — a durable ledger's read path must not fail on one malformed entry
`collect::<Result<Vec<_>, _>>()` over a stored scan makes a single malformed row
kill every future read permanently (nothing deletes ledger entries). Iterate +
accumulate, skip bad rows, and surface a `malformed_entries` count.

---

*Environment: Windows 11, Node v24.14.0, `@terminal3/t3n-sdk@4.40.0`, Rust 1.92.0,
`wasm32-wasip2` target. The companion `xagent-deals` contract in `contract/`
compiles clean from this tree (178,719-byte `.wasm`).*