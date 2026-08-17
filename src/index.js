import { T3nClient, loadWasmComponent, setEnvironment, createEthAuthInput, eth_get_address, metamask_sign, fetchTrustedManifest } from "@terminal3/t3n-sdk";
import "dotenv/config";

setEnvironment("sandbox");

const key = process.env.T3N_API_KEY;
if (!key || key === "your-sandbox-api-key-here") {
  console.error("Set T3N_API_KEY in .env (from go.terminal3.io/adk-community success screen)");
  process.exit(1);
}

const address = eth_get_address(key);
console.log("Agent ETH address:", address);

const trustAnchor = process.env.T3N_UNSAFE_TRUST === "1"
  ? { unsafe_trust_server: true }
  : await fetchTrustedManifest("sandbox");

const client = new T3nClient({
  wasmComponent: await loadWasmComponent(),
  trustAnchor,
  handlers: { EthSign: metamask_sign(address, undefined, key) },
});

await client.handshake();
await client.authenticate(createEthAuthInput(address));

const { balance } = await client.getUsage();
console.log("Credits available:", balance.available);
console.log("Connection: OK (sandbox)");