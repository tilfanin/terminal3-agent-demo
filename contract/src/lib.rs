//! XAgent Deals v0.1.0 — deal/escrow lifecycle for the T3N TEE.
//! `propose-deal` creates (state `created`); `accept-deal`: `created` -> `accepted`;
//! `fulfill-deal`: `accepted` -> `fulfilled`; `dispute-deal`: `created`/`accepted` -> `disputed`;
//! `cancel-deal`: `created` -> `cancelled`. `get-deal` / `list-deals` are read paths.
//! State lives in the `deals` KV map (`z:<tid>:deals`), keyed by deal id.
//! Business denials return `Ok(denied JSON)`; `Err` is reserved for infra faults
//! (reference GOTCHAS #3: host KV rolls back writes from a call returning Err).

pub const CONTRACT_VERSION: &str = "0.1.0";

wit_bindgen::generate!({
    world: "xagent-deals",
    path: "wit",
    additional_derives: [
        serde::Deserialize,
        serde::Serialize,
    ],
    generate_all,
});

mod deals;

struct Component;

#[cfg(target_arch = "wasm32")]
impl exports::z::xagent_deals::contracts::Guest for Component {
    fn propose_deal(req: exports::z::xagent_deals::contracts::GenericInput) -> Result<Vec<u8>, String> {
        deals::propose_deal(&req.input.ok_or("propose-deal: missing input")?)
    }

    fn accept_deal(req: exports::z::xagent_deals::contracts::GenericInput) -> Result<Vec<u8>, String> {
        deals::accept_deal(&req.input.ok_or("accept-deal: missing input")?)
    }

    fn fulfill_deal(req: exports::z::xagent_deals::contracts::GenericInput) -> Result<Vec<u8>, String> {
        deals::fulfill_deal(&req.input.ok_or("fulfill-deal: missing input")?)
    }

    fn dispute_deal(req: exports::z::xagent_deals::contracts::GenericInput) -> Result<Vec<u8>, String> {
        deals::dispute_deal(&req.input.ok_or("dispute-deal: missing input")?)
    }

    fn cancel_deal(req: exports::z::xagent_deals::contracts::GenericInput) -> Result<Vec<u8>, String> {
        deals::cancel_deal(&req.input.ok_or("cancel-deal: missing input")?)
    }

    fn get_deal(req: exports::z::xagent_deals::contracts::GenericInput) -> Result<Vec<u8>, String> {
        deals::get_deal(&req.input.ok_or("get-deal: missing input")?)
    }

    fn list_deals(req: exports::z::xagent_deals::contracts::GenericInput) -> Result<Vec<u8>, String> {
        deals::list_deals(&req.input.unwrap_or_default())
    }
}

#[cfg(target_arch = "wasm32")]
export!(Component);

#[cfg(test)]
mod tests {
    use super::CONTRACT_VERSION;

    #[test]
    fn contract_version_is_semver() {
        let parts: Vec<&str> = CONTRACT_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "CONTRACT_VERSION must be MAJOR.MINOR.PATCH");
        for part in parts {
            assert!(part.parse::<u32>().is_ok(), "each part must be a number");
        }
    }
}
