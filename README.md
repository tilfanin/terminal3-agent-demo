# Terminal3 T3N Agent — Quickstart & Walkthrough

Superteam Earn bounty submission: **LOL ventures — Create Agent ID, claim free tokens, & deploy first RUST contract on the network**.

This repo demonstrates the Terminal3 Agent Developer Kit (T3N) sandbox flow:
claim free test credits, create an Agent ID (DID), and run authenticated agent sessions.

## Prerequisites
- Node.js 18+
- Sandbox API key from https://terminal3.io/products/agent-developer-kit (Google sign-in → copy the API key + DID on the success screen; the key is shown once and cannot be retrieved later)

## Setup
```bash
cp .env.example .env   # paste your T3N_API_KEY
npm install
```

## Run
```bash
npm start
```
Expected output:
```
Agent ETH address: 0x...
Credits available: 20000000000000000000000  (20,000 test credits)
Connection: OK (sandbox)
```

## What this covers
1. **SDK init** — `setEnvironment("sandbox")`, `T3nClient` with WASM component
2. **Authentication** — handshake + authenticate with the sandbox API key
3. **Credit balance check** — `getUsage()` → spendable credits
4. **Agent ID** — the key itself encodes the agent identity (DID)

## Bugs / observations
See [`BUGS.md`](BUGS.md) for the full report (9 items: 3 verified bugs with
reproduction steps + 6 contract-development findings). Summary:

- **`agent card`/`set-card` rejected by the sandbox**: `missing field script_name` —
  the SDK's card code path never sends a field the sandbox validator requires.
- **Attestation endpoint stalls mid-stream** on some networks (~37 KB quote killed at
  ~330 B/s): `trustAnchor: { unsafe_trust_server: true }` skips it (sandbox-only), or
  use stateless `invoke()` with an `X-T3N-Api-Key` header — no handshake required.
- **`tenant.me()` missing** from the shipped SDK though the docs call it.

## Contract (going beyond the quickstart)
`contract/` is a Rust WASM contract for the Trinity TEE: **xagent-deals** — an
on-chain deal/escrow lifecycle (propose → accept → fulfill / dispute / cancel) with
all state durable in the host KV store. Builds clean to `wasm32-wasip2`:

```bash
cd contract
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
# target/wasm32-wasip2/release/xagent_deals.wasm (178,719 bytes)
```

Deploy to the sandbox (script included, reproducible):

```bash
node src/deploy-contract.js
# Authenticated as: did:t3n:...
# Registered contract: { name, contract_id }
```

## License
MIT