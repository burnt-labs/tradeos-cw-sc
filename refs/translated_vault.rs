// lib.rs — Single-file CosmWasm Vault (XION-ready, minimal hashing & verify)
//
// Summary:
// - Digest: sha256(JSON(SignableClaimV1))  (no keccak, no EIP-191 prefix)
// - Verify: deps.api.secp256k1_verify(digest32, r||s, pubkey33)
// - Verifier stored as 33-byte compressed secp256k1 pubkey (hex "0x..." or base64)
// - Assets: Native { denom }, Cw20 { contract }
// - Owner: can rotate verifier pubkey and perform emergency withdraw
//
// Messages:
//   Execute:
//     - Claim { claim, signature }
//     - SetVerifier { verifier_pubkey }
//     - EmergencyWithdraw { asset, to, value }
//   Query:
//     - GetClaimDigest { claim } -> { digest_hex }
//     - IsClaimed { digest_hex } -> { claimed }
//     - Config {} -> { owner, verifier_pubkey_hex }
//
// Build & deploy to XION as any CosmWasm contract.

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_storage_plus::{Item, Map};
use thiserror::Error;

// ---------------------------
// Types & Messages
// ---------------------------

#[cw_serde]
pub enum AssetInfo {
    Native { denom: String },
    Cw20 { contract: String },
}

#[cw_serde]
pub struct ClaimInfo {
    pub asset: AssetInfo,
    pub to: String, // bech32 on XION
    pub value: Uint128,
    pub deadline: u64, // 0 = no deadline
    pub comment: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    /// 33-byte compressed secp256k1 pubkey (hex "0x..." or base64)
    pub verifier_pubkey: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {
        claim: ClaimInfo,
        signature: String,
    },
    SetVerifier {
        verifier_pubkey: String,
    },
    EmergencyWithdraw {
        asset: AssetInfo,
        to: String,
        value: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Deterministically compute the digest the verifier signs.
    #[returns(GetClaimDigestResponse)]
    GetClaimDigest { claim: ClaimInfo },

    /// Check if a digest has been claimed (digest as hex).
    #[returns(ClaimedResponse)]
    IsClaimed { digest_hex: String },

    /// Owner + verifier info.
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct GetClaimDigestResponse {
    pub digest_hex: String,
}

#[cw_serde]
pub struct ClaimedResponse {
    pub claimed: bool,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub verifier_pubkey_hex: String,
}

// Internal, deterministic struct we hash and sign.
#[cw_serde]
struct SignableClaimV1<'a> {
    asset: &'a AssetInfo,
    to: &'a str,
    value: &'a Uint128,
    deadline: u64,
    comment: &'a str,
    // domain-separation to prevent cross-contract/chain replay
    contract_addr: &'a str,
    chain_id: &'a str,
}

// ---------------------------
// Errors
// ---------------------------

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("InvalidSignature")]
    InvalidSignature,
    #[error("InvalidValue")]
    InvalidValue,
    #[error("SignatureExpired")]
    SignatureExpired,
    #[error("AlreadyClaimed")]
    AlreadyClaimed,
    #[error("Verifier pubkey must be 33 bytes (compressed)")]
    BadPublicKeyLength,
}

// ---------------------------
// State
// ---------------------------

const CONTRACT_NAME: &str = "vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const OWNER: Item<Addr> = Item::new("owner");
/// compressed secp256k1 pubkey (33 bytes)
const VERIFIER_PUBKEY: Item<Vec<u8>> = Item::new("verifier_pubkey");
/// map: digest (32 bytes) -> bool
const CLAIMED: Map<&[u8], bool> = Map::new("claimed");

// ---------------------------
// Helpers
// ---------------------------

fn decode_hex_or_b64(s: &str) -> Result<Vec<u8>, ()> {
    let t = s.trim();
    if t.starts_with("0x") || t.starts_with("0X") {
        return hex::decode(&t[2..]).map_err(|_| ());
    }
    base64::decode(t).map_err(|_| ())
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
    let signable = SignableClaimV1 {
        asset: &claim.asset,
        to: &claim.to,
        value: &claim.value,
        deadline: claim.deadline,
        comment: &claim.comment,
        contract_addr: env.contract.address.as_str(),
        chain_id: &env.block.chain_id,
    };
    let bytes = to_json_binary(&signable).expect("serialize SignableClaimV1");
    let vec32 = cosmwasm_std::sha256(&bytes); // Vec<u8>(32)
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&vec32);
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

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = match msg.owner {
        Some(o) => deps.api.addr_validate(&o)?,
        None => info.sender.clone(),
    };
    OWNER.save(deps.storage, &owner)?;

    let pk = decode_hex_or_b64(&msg.verifier_pubkey)
        .map_err(|_| StdError::generic_err("invalid verifier_pubkey encoding"))?;
    if pk.len() != 33 {
        return Err(ContractError::BadPublicKeyLength);
    }
    VERIFIER_PUBKEY.save(deps.storage, &pk)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("verifier_pubkey_len", pk.len().to_string()))
}

#[entry_point]
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
    let pk = decode_hex_or_b64(&verifier_pubkey)
        .map_err(|_| StdError::generic_err("invalid verifier_pubkey encoding"))?;
    if pk.len() != 33 {
        return Err(ContractError::BadPublicKeyLength);
    }
    VERIFIER_PUBKEY.save(deps.storage, &pk)?;
    Ok(Response::new().add_attribute("action", "set_verifier"))
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
        .add_attribute("to", to_addr)
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
        .map_err(|_| StdError::generic_err("invalid recipient address"))?;
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
        .add_attribute("to", to_addr)
        .add_attribute("value", claim.value.to_string())
        .add_attribute("digest_hex", hex::encode(digest)))
}

#[entry_point]
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
