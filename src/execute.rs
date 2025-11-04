use crate::error::ContractError;
use crate::msg::{AssetInfo, ClaimInfo, ExecuteMsg, InstantiateMsg};
use crate::state::{CLAIMED, OWNER, VERIFIER_PUBKEY};
use crate::{CONTRACT_NAME, CONTRACT_VERSION};
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;

// ---------------------------
// Helpers
// ---------------------------

fn decode_hex_or_b64(s: &str) -> Result<Vec<u8>, ()> {
    let t = s.trim();
    if t.starts_with("0x") || t.starts_with("0X") {
        return hex::decode(&t[2..]).map_err(|_| ());
    }
    // Use base64 engine for decoding
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(t)
        .map_err(|_| ())
}

fn digest_for_claim(env: &Env, claim: &ClaimInfo) -> [u8; 32] {
    use crate::msg::SignableClaimV1;
    use sha2::{Digest, Sha256};
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

fn ensure_owner(storage: &dyn cosmwasm_std::Storage, sender: &Addr) -> Result<(), ContractError> {
    let owner = OWNER.load(storage)?;
    if &owner != sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

// ---------------------------
// Entry points
// ---------------------------

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = match msg.owner {
        Some(o) => deps.api.addr_validate(&o)?,
        None => info.sender.clone(),
    };
    OWNER.save(deps.storage, &owner)?;

    let pk = decode_hex_or_b64(&msg.verifier_pubkey).map_err(|_| {
        ContractError::Std(StdError::generic_err("invalid verifier_pubkey encoding"))
    })?;
    if pk.len() != 33 {
        return Err(ContractError::BadPublicKeyLength);
    }
    VERIFIER_PUBKEY.save(deps.storage, &pk)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner.to_string())
        .add_attribute("verifier_pubkey_len", pk.len().to_string()))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim { claim, signature } => exec_claim(deps, env, claim, signature),
        ExecuteMsg::SetVerifier { verifier_pubkey } => {
            exec_set_verifier(deps, info, verifier_pubkey)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            exec_transfer_ownership(deps, info, new_owner)
        }
        ExecuteMsg::EmergencyWithdraw { asset, to, value } => {
            exec_emergency_withdraw(deps, info, asset, to, value)
        }
    }
}

fn exec_set_verifier(
    deps: DepsMut,
    info: MessageInfo,
    verifier_pubkey: String,
) -> Result<Response, ContractError> {
    ensure_owner(deps.storage, &info.sender)?;
    let pk = decode_hex_or_b64(&verifier_pubkey).map_err(|_| {
        ContractError::Std(StdError::generic_err("invalid verifier_pubkey encoding"))
    })?;
    if pk.len() != 33 {
        return Err(ContractError::BadPublicKeyLength);
    }
    VERIFIER_PUBKEY.save(deps.storage, &pk)?;
    Ok(Response::new()
        .add_attribute("action", "set_verifier")
        .add_attribute("verifier_pubkey_len", pk.len().to_string()))
}

fn exec_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    ensure_owner(deps.storage, &info.sender)?;
    let old_owner = OWNER.load(deps.storage)?;
    let new_owner_addr = deps.api.addr_validate(&new_owner)?;

    OWNER.save(deps.storage, &new_owner_addr)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_attribute("old_owner", old_owner.to_string())
        .add_attribute("new_owner", new_owner_addr.to_string()))
}

fn exec_emergency_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    asset: AssetInfo,
    to: String,
    value: Uint128,
) -> Result<Response, ContractError> {
    ensure_owner(deps.storage, &info.sender)?;
    let to_addr = deps.api.addr_validate(&to)?;

    let msg: CosmosMsg = match asset {
        AssetInfo::Native { denom } => {
            let coin = Coin {
                denom,
                amount: value,
            };
            BankMsg::Send {
                to_address: to_addr.to_string(),
                amount: vec![coin],
            }
            .into()
        }
        AssetInfo::Cw20 { contract } => {
            let cw20_addr = deps.api.addr_validate(&contract)?;
            let exec = Cw20ExecuteMsg::Transfer {
                recipient: to_addr.to_string(),
                amount: value,
            };
            WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&exec)?,
                funds: vec![],
            }
            .into()
        }
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "emergency_withdraw")
        .add_attribute("to", to_addr.to_string())
        .add_attribute("value", value.to_string()))
}

fn exec_claim(
    deps: DepsMut,
    env: Env,
    claim: ClaimInfo,
    signature: String,
) -> Result<Response, ContractError> {
    // Basic validations
    if claim.value.is_zero() {
        return Err(ContractError::InvalidValue);
    }
    let to_addr = deps
        .api
        .addr_validate(&claim.to)
        .map_err(|_| ContractError::Std(StdError::generic_err("invalid recipient address")))?;
    if claim.deadline != 0 && claim.deadline < env.block.time.seconds() {
        return Err(ContractError::SignatureExpired);
    }

    // Compute digest
    let digest = digest_for_claim(&env, &claim);

    // Replay protection
    if CLAIMED.may_load(deps.storage, &digest)?.unwrap_or(false) {
        return Err(ContractError::AlreadyClaimed);
    }

    // Verify signature against stored pubkey
    let mut sig = decode_hex_or_b64(&signature).map_err(|_| ContractError::InvalidSignature)?;
    if sig.len() == 65 {
        sig.truncate(64); // drop v if provided
    }
    if sig.len() != 64 {
        return Err(ContractError::InvalidSignature);
    }
    let pk = VERIFIER_PUBKEY.load(deps.storage)?;
    let ok = deps.api.secp256k1_verify(&digest, &sig, &pk)?;
    if !ok {
        return Err(ContractError::InvalidSignature);
    }

    // Mark claimed then transfer
    CLAIMED.save(deps.storage, &digest, &true)?;

    let transfer_msg: CosmosMsg = match &claim.asset {
        AssetInfo::Native { denom } => {
            let coin = Coin {
                denom: denom.clone(),
                amount: claim.value,
            };
            BankMsg::Send {
                to_address: to_addr.to_string(),
                amount: vec![coin],
            }
            .into()
        }
        AssetInfo::Cw20 { contract } => {
            let cw20_addr = deps.api.addr_validate(contract)?;
            let exec = Cw20ExecuteMsg::Transfer {
                recipient: to_addr.to_string(),
                amount: claim.value,
            };
            WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&exec)?,
                funds: vec![],
            }
            .into()
        }
    };

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("action", "claim")
        .add_attribute(
            "asset",
            match &claim.asset {
                AssetInfo::Native { denom } => format!("N:{denom}"),
                AssetInfo::Cw20 { contract } => format!("C:{contract}"),
            },
        )
        .add_attribute("to", to_addr.to_string())
        .add_attribute("value", claim.value.to_string())
        .add_attribute("digest_hex", hex::encode(digest)))
}
