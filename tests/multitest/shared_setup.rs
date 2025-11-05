use cosmwasm_std::{coins, Addr};
use cw_multi_test::{App, Contract, ContractWrapper, Executor, IntoAddr};
use k256::ecdsa::{SigningKey, VerifyingKey};
use tradeos_cw_sc::helpers::to_eth_signed_message_hash;
use tradeos_cw_sc::msg::{ClaimInfo, InstantiateMsg};

// Constants for testing
pub const DENOM: &str = "uxion";
pub const INITIAL_BALANCE: u128 = 1000000;

// Helper function to generate a test secp256k1 key pair
pub fn generate_test_keypair() -> (SigningKey, VerifyingKey) {
    use rand::thread_rng;
    let signing_key = SigningKey::random(&mut thread_rng());
    let verifying_key = *signing_key.verifying_key();
    (signing_key, verifying_key)
}

// Helper function to get compressed public key (33 bytes) as hex string
pub fn pubkey_to_hex(pk: &VerifyingKey) -> String {
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    // Get the affine point from the verifying key
    let affine = pk.as_affine();
    let encoded_point = affine.to_encoded_point(true); // compressed
    let bytes = encoded_point.as_bytes();
    format!("0x{}", hex::encode(bytes))
}

// Helper function to sign a message hash with the signing key
pub fn sign_message_hash(signing_key: &SigningKey, message_hash: &[u8; 32]) -> String {
    use k256::ecdsa::signature::Signer;
    let signature: k256::ecdsa::Signature = signing_key.sign(message_hash);
    let sig_bytes = signature.to_bytes();
    format!("0x{}", hex::encode(sig_bytes))
}

// Helper function to create a claim and generate a valid signature
// Note: This function needs the contract address to compute the hash correctly
// We use string conversion to bridge between cosmwasm-std 2.2.2 and 3.x types
pub fn create_claim_with_signature(
    app: &App,
    contract_addr: &str,
    claim: &ClaimInfo,
    signing_key: &SigningKey,
) -> String {
    let block_info = app.block_info();
    
    // Compute claim info hash manually to avoid type conversion issues
    // Use tiny-keccak to match the contract's implementation
    use tradeos_cw_sc::msg::AssetInfo;
    use tiny_keccak::{Hasher, Keccak};
    
    // Token identifier
    let token_bytes = match &claim.asset {
        AssetInfo::Native { .. } => vec![],
        AssetInfo::Cw20 { contract } => contract.as_bytes().to_vec(),
    };
    
    // To address
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
    let contract_bytes = contract_addr.as_bytes();
    
    // Chain ID: string as bytes
    let chain_id_bytes = block_info.chain_id.as_bytes();
    
    // abi.encodePacked equivalent: concatenate all values
    let mut packed = Vec::new();
    packed.extend_from_slice(&token_bytes);
    packed.extend_from_slice(to_bytes);
    packed.extend_from_slice(&value_bytes_vec);
    packed.extend_from_slice(&deadline_bytes);
    packed.extend_from_slice(comment_bytes);
    packed.extend_from_slice(contract_bytes);
    packed.extend_from_slice(chain_id_bytes);
    
    // keccak256 hash
    let mut hasher = Keccak::v256();
    hasher.update(&packed);
    let mut claim_info_hash = [0u8; 32];
    hasher.finalize(&mut claim_info_hash);
    
    // Apply EIP-191 prefix
    let eth_signed_message_hash = to_eth_signed_message_hash(&claim_info_hash);
    
    // Sign the message hash
    sign_message_hash(signing_key, &eth_signed_message_hash)
}

// Contract wrapper for multitest
fn contract_vault() -> Box<dyn Contract<cosmwasm_std::Empty>> {
    let contract = ContractWrapper::new(
        tradeos_cw_sc::contract::execute,
        tradeos_cw_sc::contract::instantiate,
        tradeos_cw_sc::contract::query,
    );
    Box::new(contract)
}

// Main setup function to deploy contracts and initialize test environment
pub fn setup_contract(app: &mut App, verifier_pubkey: Option<String>) -> Addr {
    // Create test accounts with proper bech32 addresses
    let admin = "admin".into_addr();
    let user1 = "user1".into_addr();
    let user2 = "user2".into_addr();

    // Fund accounts
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin, coins(INITIAL_BALANCE, DENOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &user1, coins(INITIAL_BALANCE, DENOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &user2, coins(INITIAL_BALANCE, DENOM))
            .unwrap();
    });

    // Deploy vault contract
    let contract_id = app.store_code(contract_vault());
    let msg = InstantiateMsg {
        owner: Some(admin.to_string()),
        verifier_pubkey: verifier_pubkey.unwrap_or_else(|| {
            let (_signing_key, verifying_key) = generate_test_keypair();
            pubkey_to_hex(&verifying_key)
        }),
    };

    let contract_addr = app
        .instantiate_contract(
            contract_id,
            admin.clone(),
            &msg,
            &[],
            "tradeos-vault",
            Some(admin.to_string()),
        )
        .unwrap();

    contract_addr
}
