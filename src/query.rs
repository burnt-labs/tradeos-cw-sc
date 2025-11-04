use crate::msg::{ClaimInfo, ClaimedResponse, ConfigResponse, GetClaimDigestResponse, QueryMsg};
use crate::state::{CLAIMED, OWNER, VERIFIER_PUBKEY};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError, StdResult};

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetClaimDigest { claim } => {
            let d = digest_for_claim(&env, &claim);
            to_json_binary(&GetClaimDigestResponse {
                digest_hex: hex::encode(d),
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

fn parse_hex_32(s: &str) -> StdResult<[u8; 32]> {
    let h = if s.starts_with("0x") || s.starts_with("0X") {
        &s[2..]
    } else {
        s
    };
    let v = hex::decode(h).map_err(|_| StdError::generic_err("invalid hex"))?;
    if v.len() != 32 {
        return Err(StdError::generic_err("hex must be 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&v);
    Ok(out)
}

fn digest_for_claim(env: &Env, claim: &ClaimInfo) -> [u8; 32] {
    use crate::msg::SignableClaimV1;
    use sha2::{Sha256, Digest};
    let signable = SignableClaimV1 {
        asset: claim.asset.clone(),
        to: claim.to.clone(),
        value: claim.value,
        deadline: claim.deadline,
        comment: claim.comment.clone(),
        contract_addr: env.contract.address.as_str().to_string(),
        chain_id: env.block.chain_id.clone(),
    };
    let bytes = to_json_binary(&signable).expect("serialize SignableClaimV1");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&hash);
    arr
}

