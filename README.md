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
- **Attestation quote fetch can stall mid-stream**:
  `fetchTrustedManifest("sandbox")` succeeds (trust-manifest is 518 B) but the ML-KEM
  handshake then fetches the ~37 KB TDX attestation quote, which can download at ~330 B/s
  and get killed mid-stream (`TypeError: terminated` / `ECONNRESET`) on some networks. All
  other endpoints (`GET /status`, `POST /api/rpc`, `POST /api/invoke`) work fine.
  - **Workaround**: `trustAnchor: { unsafe_trust_server: true }` skips the attestation
    fetch entirely → handshake + authenticate + getUsage complete. Safe for sandbox test
    credits only (never for production nodes with real funds).
  - **Alternative**: stateless `invoke()` — single `POST /api/invoke` with an
    `X-T3N-Api-Key` header, no handshake/session/attestation required.
- (fill in during execution: any further errors hit, e.g. `tenant.me()` throwing at "Set Up Dev Environment", with reproduction steps and workarounds)

## License
MIT