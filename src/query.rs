use crate::helpers::{get_claim_info_hash, parse_hex_32, to_eth_signed_message_hash};
use crate::msg::{ClaimedResponse, ConfigResponse, GetClaimDigestResponse, QueryMsg};
use crate::state::{CLAIMED, OWNER, VERIFIER_PUBKEY};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetClaimDigest { claim } => {
            // Return ethSignedMessageHash (matching Solidity's behavior)
            let claim_info_hash = get_claim_info_hash(&env, &claim);
            let eth_signed_message_hash = to_eth_signed_message_hash(&claim_info_hash);
            to_json_binary(&GetClaimDigestResponse {
                digest_hex: hex::encode(eth_signed_message_hash),
            })
        }
        QueryMsg::IsClaimed { digest_hex } => {
            let d = parse_hex_32(&digest_hex)?;
            let claimed = CLAIMED.may_load(deps.storage, &d)?.unwrap_or(false);
            to_json_binary(&ClaimedResponse { claimed })
        }
        QueryMsg::Config {} => {
            let owner = OWNER.load(deps.storage)?;
            let pk = VERIFIER_PUBKEY.load(deps.storage)?;
            to_json_binary(&ConfigResponse {
                owner: owner.to_string(),
                verifier_pubkey_hex: format!("0x{}", hex::encode(pk)),
            })
        }
    }
}


