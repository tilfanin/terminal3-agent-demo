use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::host::{
    interfaces::kv_store,
    tenant::tenant_context,
};

const DEALS_MAP: &str = "deals";

fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Serialize, Deserialize)]
struct Deal {
    id: String,
    proposer: String,
    counterparty: String,
    token: String,
    amount: String,
    description_hash: String,
    state: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
struct ProposeInput {
    counterparty: String,
    token: String,
    amount: String,
    description_hash: String,
}

#[derive(Deserialize)]
struct DealIdInput {
    deal_id: String,
}

#[derive(Deserialize)]
struct DisputeInput {
    deal_id: String,
    reason: String,
}

fn tenant_did() -> Result<String, String> {
    let did = tenant_context::tenant_did();
    Ok(hex::encode(did))
}

fn map_name(tenant: &str, map: &str) -> String {
    format!("z:{tenant}:{map}")
}

fn read_deal(tenant: &str, id: &str) -> Result<Option<Deal>, String> {
    let map = map_name(tenant, DEALS_MAP);
    let raw = kv_store::get(&map, id.as_bytes())?;
    match raw {
        Some(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|e| e.to_string()),
        None => Ok(None),
    }
}

fn write_deal(tenant: &str, deal: &Deal) -> Result<(), String> {
    let map = map_name(tenant, DEALS_MAP);
    let bytes = serde_json::to_vec(deal).map_err(|e| e.to_string())?;
    kv_store::put(&map, deal.id.as_bytes(), &bytes)
}

fn denied(reason: &str) -> Result<Vec<u8>, String> {
    Ok(serde_json::json!({ "denied": reason }).to_string().into_bytes())
}

fn ok_json(value: serde_json::Value) -> Result<Vec<u8>, String> {
    Ok(value.to_string().into_bytes())
}

fn handle_propose(input: &[u8]) -> Result<Vec<u8>, String> {
    let req: ProposeInput = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    let tenant = tenant_did()?;
    let id = format!("deal-{:016x}", fnv1a(input));
    if read_deal(&tenant, &id)?.is_some() {
        return denied("deal id collision");
    }
    let deal = Deal {
        id: id.clone(),
        proposer: tenant.clone(),
        counterparty: req.counterparty,
        token: req.token,
        amount: req.amount,
        description_hash: req.description_hash,
        state: "created".to_string(),
        reason: None,
    };
    write_deal(&tenant, &deal)?;
    ok_json(serde_json::json!({ "deal_id": id, "state": "created" }))
}

fn transition(tenant: &str, input: &[u8], allowed: &[&str], to: &str) -> Result<Vec<u8>, String> {
    let req: DealIdInput = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    let mut deal = match read_deal(tenant, &req.deal_id)? {
        Some(d) => d,
        None => return denied("deal not found"),
    };
    if !allowed.contains(&deal.state.as_str()) {
        return denied(format!("invalid state transition {} -> {}", deal.state, to).as_str());
    }
    deal.state = to.to_string();
    write_deal(tenant, &deal)?;
    ok_json(serde_json::json!({ "deal_id": deal.id, "state": deal.state }))
}

pub fn propose_deal(input: &[u8]) -> Result<Vec<u8>, String> {
    handle_propose(input)
}

pub fn accept_deal(input: &[u8]) -> Result<Vec<u8>, String> {
    transition(&tenant_did()?, input, &["created"], "accepted")
}

pub fn fulfill_deal(input: &[u8]) -> Result<Vec<u8>, String> {
    transition(&tenant_did()?, input, &["accepted"], "fulfilled")
}

pub fn dispute_deal(input: &[u8]) -> Result<Vec<u8>, String> {
    let tenant = tenant_did()?;
    let req: DisputeInput = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    let mut deal = match read_deal(&tenant, &req.deal_id)? {
        Some(d) => d,
        None => return denied("deal not found"),
    };
    if !matches!(deal.state.as_str(), "created" | "accepted") {
        return denied(format!("invalid state transition {} -> disputed", deal.state).as_str());
    }
    deal.state = "disputed".to_string();
    deal.reason = Some(req.reason);
    write_deal(&tenant, &deal)?;
    ok_json(serde_json::json!({ "deal_id": deal.id, "state": deal.state, "reason": deal.reason }))
}

pub fn cancel_deal(input: &[u8]) -> Result<Vec<u8>, String> {
    transition(&tenant_did()?, input, &["created"], "cancelled")
}

pub fn get_deal(input: &[u8]) -> Result<Vec<u8>, String> {
    let tenant = tenant_did()?;
    let req: DealIdInput = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    match read_deal(&tenant, &req.deal_id)? {
        Some(d) => ok_json(serde_json::to_value(&d).map_err(|e| e.to_string())?),
        None => denied("deal not found"),
    }
}

pub fn list_deals(_input: &[u8]) -> Result<Vec<u8>, String> {
    let tenant = tenant_did()?;
    let map = map_name(&tenant, DEALS_MAP);
    let pairs = kv_store::scan(&map, &[], &[0xFF], 256)
        .map_err(|e| e.to_string())?;
    let mut deals = Vec::new();
    for (_, value) in pairs {
        if let Ok(d) = serde_json::from_slice::<Deal>(&value) {
            deals.push(d);
        }
    }
    ok_json(serde_json::to_value(&deals).map_err(|e| e.to_string())?)
}