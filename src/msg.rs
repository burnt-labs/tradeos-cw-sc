use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

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

#[cw_ownable_execute]
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

#[cw_ownable_query]
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
    pub owner: Option<String>,
    pub verifier_pubkey_hex: String,
}

#[cw_serde]
pub struct MigrateMsg {}
