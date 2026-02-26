use crate::error::ContractError;
use crate::helpers::{decode_hex_or_b64, get_claim_info_hash, to_eth_signed_message_hash};
use crate::msg::{AssetInfo, ClaimInfo, ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{CLAIMED, VERIFIER_PUBKEY};
use crate::{CONTRACT_NAME, CONTRACT_VERSION};
use cosmwasm_std::{
    to_json_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ExecuteMsg;
use cw_ownable::{assert_owner, initialize_owner, update_ownership, Action};

// ---------------------------
// Entry points
// ---------------------------

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = msg.owner.unwrap_or_else(|| info.sender.to_string());
    initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

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
        ExecuteMsg::UpdateOwnership(action) => exec_update_ownership(deps, env, info, action),
        ExecuteMsg::Claim { claim, signature } => exec_claim(deps, env, claim, signature),
        ExecuteMsg::SetVerifier { verifier_pubkey } => {
            exec_set_verifier(deps, info, verifier_pubkey)
        }
        ExecuteMsg::EmergencyWithdraw { asset, to, value } => {
            exec_emergency_withdraw(deps, info, asset, to, value)
        }
    }
}

fn exec_update_ownership(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: Action,
) -> Result<Response, ContractError> {
    let action_name = match action {
        Action::TransferOwnership { .. } => "transfer_ownership",
        Action::AcceptOwnership => "accept_ownership",
        Action::RenounceOwnership => "renounce_ownership",
    };

    update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::new()
        .add_attribute("action", "update_ownership")
        .add_attribute("ownership_action", action_name))
}

fn exec_set_verifier(
    deps: DepsMut,
    info: MessageInfo,
    verifier_pubkey: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &info.sender)?;
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

fn exec_emergency_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    asset: AssetInfo,
    to: String,
    value: Uint128,
) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &info.sender)?;
    if value.is_zero() {
        return Err(ContractError::InvalidValue);
    }
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

    // Compute claimInfoHash (same as Solidity's getClaimInfoHash)
    let claim_info_hash = get_claim_info_hash(&env, &claim);

    // Apply EIP-191 prefix (same as Solidity's toEthSignedMessageHash())
    let eth_signed_message_hash = to_eth_signed_message_hash(&claim_info_hash);

    // Replay protection - store ethSignedMessageHash (matching Solidity behavior)
    if CLAIMED
        .may_load(deps.storage, &eth_signed_message_hash)?
        .unwrap_or(false)
    {
        return Err(ContractError::AlreadyClaimed);
    }

    // Verify signature against stored pubkey
    // Note: In Solidity, we use recover() to get the signer address, then compare with verifier
    // In CosmWasm, we verify the signature directly against the stored pubkey
    let mut sig = decode_hex_or_b64(&signature).map_err(|_| ContractError::InvalidSignature)?;
    if sig.len() == 65 {
        sig.truncate(64); // drop v if provided
    }
    if sig.len() != 64 {
        return Err(ContractError::InvalidSignature);
    }
    let pk = VERIFIER_PUBKEY.load(deps.storage)?;
    // Verify signature against ethSignedMessageHash (matching Solidity behavior)
    let ok = deps
        .api
        .secp256k1_verify(&eth_signed_message_hash, &sig, &pk)?;
    if !ok {
        return Err(ContractError::InvalidSignature);
    }

    // Mark claimed using ethSignedMessageHash (matching Solidity behavior)
    CLAIMED.save(deps.storage, &eth_signed_message_hash, &true)?;

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
        .add_attribute("claim_info_hash", hex::encode(claim_info_hash))
        .add_attribute(
            "eth_signed_message_hash",
            hex::encode(eth_signed_message_hash),
        ))
}

// ---------------------------
// Migration
// ---------------------------

pub fn migrate(deps: DepsMut, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    // Verify that we're migrating from the same contract
    if contract_version.contract != CONTRACT_NAME {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Cannot migrate from {} to {}",
            contract_version.contract, CONTRACT_NAME
        ))));
    }

    // Update contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("from_version", contract_version.version)
        .add_attribute("to_version", CONTRACT_VERSION))
}
