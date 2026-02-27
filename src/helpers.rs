use crate::msg::{AssetInfo, ClaimInfo};
use cosmwasm_std::{Env, StdError, StdResult};
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    InvalidEncoding,
}

/// Decode hex or base64 string to bytes
pub fn decode_hex_or_b64(s: &str) -> Result<Vec<u8>, DecodeError> {
    let t = s.trim();
    if t.starts_with("0x") || t.starts_with("0X") {
        return hex::decode(&t[2..]).map_err(|_| DecodeError::InvalidEncoding);
    }
    // Use base64 engine for decoding
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(t)
        .map_err(|_| DecodeError::InvalidEncoding)
}

/// Parse hex string to 32-byte array
pub fn parse_hex_32(s: &str) -> StdResult<[u8; 32]> {
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

/// Compute claimInfoHash as keccak256(
///   len(token) || token ||
///   len(to) || to ||
///   value(32-byte big-endian) ||
///   deadline(8-byte big-endian) ||
///   len(comment) || comment ||
///   len(contract_addr) || contract_addr ||
///   len(chain_id) || chain_id
/// ).
/// All string/variable-length fields are length-prefixed (u32 big-endian) to avoid hash collisions.
/// Uses bech32 addresses directly as bytes (no conversion to Ethereum address format).
pub fn get_claim_info_hash(env: &Env, claim: &ClaimInfo) -> [u8; 32] {
    // Token identifier: denom string for native, contract address string for cw20
    let token_bytes = match &claim.asset {
        AssetInfo::Native { denom } => {
            // Use denom string as bytes for native token
            denom.as_bytes().to_vec()
        }
        AssetInfo::Cw20 { contract } => {
            // Use contract address string directly as bytes
            contract.as_bytes().to_vec()
        }
    };

    // To address: use bech32 address string directly as bytes
    let to_bytes = claim.to.as_bytes();

    // Value: 32-byte big-endian
    let value_bytes = claim.value.to_be_bytes();
    let mut value_bytes_vec = vec![0u8; 32];
    value_bytes_vec[32 - value_bytes.len()..].copy_from_slice(&value_bytes);

    // Deadline: 8-byte (u64) big-endian
    let deadline_bytes = claim.deadline.to_be_bytes();

    // Comment: string as bytes
    let comment_bytes = claim.comment.as_bytes();

    // Contract address: use bech32 address string directly as bytes
    let contract_bytes = env.contract.address.as_str().as_bytes();

    // Chain ID: string as bytes
    let chain_id_bytes = env.block.chain_id.as_bytes();

    // Length-prefix variable-width fields to avoid boundary ambiguity collisions.
    let mut packed = Vec::new();
    let append_len_prefixed = |dst: &mut Vec<u8>, bytes: &[u8]| {
        let len = bytes.len() as u32;
        dst.extend_from_slice(&len.to_be_bytes());
        dst.extend_from_slice(bytes);
    };

    append_len_prefixed(&mut packed, &token_bytes);
    append_len_prefixed(&mut packed, to_bytes);
    packed.extend_from_slice(&value_bytes_vec);
    packed.extend_from_slice(&deadline_bytes);
    append_len_prefixed(&mut packed, comment_bytes);
    append_len_prefixed(&mut packed, contract_bytes);
    append_len_prefixed(&mut packed, chain_id_bytes);

    // keccak256 hash
    let mut hasher = Keccak::v256();
    hasher.update(&packed);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    hash
}

/// Apply EIP-191 prefix: \x19Ethereum Signed Message:\n32 + hash
/// Equivalent to Solidity's toEthSignedMessageHash()
pub fn to_eth_signed_message_hash(hash: &[u8; 32]) -> [u8; 32] {
    let prefix = b"\x19Ethereum Signed Message:\n32";
    let mut hasher = Keccak::v256();
    hasher.update(prefix);
    hasher.update(hash);
    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::get_claim_info_hash;
    use crate::msg::{AssetInfo, ClaimInfo};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::Uint128;

    #[test]
    fn claim_hash_should_differ_for_different_claims() {
        let env = mock_env();

        let claim_a = ClaimInfo {
            asset: AssetInfo::Native {
                denom: "uxion".to_string(),
            },
            to: "xion1abc".to_string(),
            value: Uint128::new(42),
            deadline: 1_700_000_000,
            comment: "hello".to_string(),
        };

        let claim_b = ClaimInfo {
            asset: AssetInfo::Native {
                denom: "uxionx".to_string(),
            },
            to: "ion1abc".to_string(),
            value: Uint128::new(42),
            deadline: 1_700_000_000,
            comment: "hello".to_string(),
        };

        let hash_a = get_claim_info_hash(&env, &claim_a);
        let hash_b = get_claim_info_hash(&env, &claim_b);

        assert_ne!(
            hash_a, hash_b,
            "distinct claims must produce distinct hashes"
        );
    }

    #[test]
    fn claim_hash_collision_scenario_is_prevented() {
        // Without length-prefixing, token||to would both become "abcdef":
        // claim_a: token="abc", to="def"
        // claim_b: token="ab",  to="cdef"
        let env = mock_env();

        let claim_a = ClaimInfo {
            asset: AssetInfo::Native {
                denom: "abc".to_string(),
            },
            to: "def".to_string(),
            value: Uint128::new(42),
            deadline: 1_700_000_000,
            comment: "collision-test".to_string(),
        };

        let claim_b = ClaimInfo {
            asset: AssetInfo::Native {
                denom: "ab".to_string(),
            },
            to: "cdef".to_string(),
            value: Uint128::new(42),
            deadline: 1_700_000_000,
            comment: "collision-test".to_string(),
        };

        let hash_a = get_claim_info_hash(&env, &claim_a);
        let hash_b = get_claim_info_hash(&env, &claim_b);

        assert_ne!(
            hash_a, hash_b,
            "claims that would collide under naive concatenation must remain distinct"
        );
    }
}
