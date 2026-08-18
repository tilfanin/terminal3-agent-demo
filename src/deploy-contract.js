// Deploy the xagent-deals WASM contract to the T3N sandbox.
// Usage: node src/deploy-contract.js  (requires T3N_API_KEY + T3N_UNSAFE_TRUST=1 in .env)
// Reproduces the exact register() call from the docs walkthrough.
import { readFile } from "fs/promises";
import {
  T3nClient,
  TenantClient,
  setEnvironment,
  getNodeUrl,
  loadWasmComponent,
  eth_get_address,
  metamask_sign,
  createEthAuthInput,
} from "@terminal3/t3n-sdk";
import "dotenv/config";

setEnvironment("sandbox");

const key = process.env.T3N_API_KEY;
if (!key || key === "your-sandbox-api-key-here") {
  console.error("Set T3N_API_KEY in .env (from go.terminal3.io/adk-community success screen)");
  process.exit(1);
}

const CONTRACT_TAIL = "xagent-deals";
const CONTRACT_VERSION = "0.1.0";
const WASM_PATH = "./contract/target/wasm32-wasip2/release/xagent_deals.wasm";

const address = eth_get_address(key);
console.log("Agent ETH address:", address);

const trustAnchor = process.env.T3N_UNSAFE_TRUST === "1"
  ? { unsafe_trust_server: true }
  : undefined;

const client = new T3nClient({
  wasmComponent: await loadWasmComponent(),
  trustAnchor,
  handlers: { EthSign: metamask_sign(address, undefined, key) },
});

await client.handshake();
const did = await client.authenticate(createEthAuthInput(address));
console.log("Authenticated as:", did.value);

const tenant = new TenantClient({
  t3n: client,
  baseUrl: getNodeUrl(),
  tenantDid: did.value,
});

const wasmBytes = await readFile(WASM_PATH);
console.log(`WASM bundle: ${WASM_PATH} (${wasmBytes.length} bytes)`);

const wasmBlob = new Blob([wasmBytes], { type: "application/wasm" });

const MAX_ATTEMPTS = Number(process.env.DEPLOY_ATTEMPTS ?? "30");
const RETRY_MS = Number(process.env.DEPLOY_RETRY_MS ?? "30000");

let result;
for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt++) {
  try {
    result = await tenant.contracts.register({
      tail: CONTRACT_TAIL,
      version: CONTRACT_VERSION,
      wasm: wasmBlob,
    });
    break;
  } catch (err) {
    console.log(`[${new Date().toISOString()}] register attempt ${attempt}/${MAX_ATTEMPTS} failed: ${err.cause?.code ?? err.message}`);
    if (attempt === MAX_ATTEMPTS) throw err;
    await new Promise((r) => setTimeout(r, RETRY_MS));
  }
}

console.log("Registered contract:", JSON.stringify(result, null, 2));
