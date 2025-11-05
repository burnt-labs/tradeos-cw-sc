use cosmwasm_std::Uint128;
use cw_multi_test::{App, Executor, IntoAddr};
use tradeos_cw_sc::msg::{
    AssetInfo, ClaimInfo, ClaimedResponse, ConfigResponse, ExecuteMsg, GetClaimDigestResponse,
    QueryMsg,
};

use super::shared_setup::{
    create_claim_with_signature, generate_test_keypair, pubkey_to_hex, setup_contract, DENOM,
};

#[test]
fn test_contract_instantiation() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);

    // Query contract config
    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(config.owner, "admin".into_addr().to_string());
    assert!(!config.verifier_pubkey_hex.is_empty());
    assert!(config.verifier_pubkey_hex.starts_with("0x"));
}

#[test]
fn test_get_claim_digest() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: "user1".into_addr().to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim".to_string(),
    };

    let response: GetClaimDigestResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetClaimDigest {
                claim: claim.clone(),
            },
        )
        .unwrap();

    // Verify the digest is a valid hex string (32 bytes = 64 hex chars)
    assert_eq!(response.digest_hex.len(), 64);
    assert!(hex::decode(&response.digest_hex).is_ok());
}

#[test]
fn test_is_claimed_query() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: "user1".into_addr().to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim".to_string(),
    };

    // Get digest
    let digest_response: GetClaimDigestResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetClaimDigest {
                claim: claim.clone(),
            },
        )
        .unwrap();

    // Check if claimed (should be false initially)
    let is_claimed: ClaimedResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::IsClaimed {
                digest_hex: digest_response.digest_hex.clone(),
            },
        )
        .unwrap();

    assert!(!is_claimed.claimed);
}

#[test]
fn test_set_verifier() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let admin = "admin".into_addr();

    let (_signing_key, verifying_key) = generate_test_keypair();
    let new_pubkey = pubkey_to_hex(&verifying_key);

    // Owner can set verifier
    let msg = ExecuteMsg::SetVerifier {
        verifier_pubkey: new_pubkey.clone(),
    };
    let res = app.execute_contract(admin.clone(), contract_addr.clone(), &msg, &[]);
    assert!(res.is_ok());

    // Verify the verifier was updated
    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
        .unwrap();

    // Verify the verifier was updated (check it's a valid hex string)
    assert!(config.verifier_pubkey_hex.starts_with("0x"));
    assert_eq!(config.verifier_pubkey_hex.len(), 68); // 0x + 66 hex chars (33 bytes)
}

#[test]
fn test_set_verifier_unauthorized() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let user1 = "user1".into_addr();

    let (_signing_key, verifying_key) = generate_test_keypair();
    let new_pubkey = pubkey_to_hex(&verifying_key);

    // Non-owner cannot set verifier
    let msg = ExecuteMsg::SetVerifier {
        verifier_pubkey: new_pubkey,
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_transfer_ownership() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let admin = "admin".into_addr();
    let user1 = "user1".into_addr();

    // Owner can transfer ownership
    let msg = ExecuteMsg::TransferOwnership {
        new_owner: user1.to_string(),
    };
    let res = app.execute_contract(admin.clone(), contract_addr.clone(), &msg, &[]);
    assert!(res.is_ok());

    // Verify ownership was transferred
    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(config.owner, user1.to_string());

    // New owner can now perform owner operations
    let (_signing_key, verifying_key) = generate_test_keypair();
    let new_pubkey = pubkey_to_hex(&verifying_key);
    let msg = ExecuteMsg::SetVerifier {
        verifier_pubkey: new_pubkey,
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn test_transfer_ownership_unauthorized() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let user1 = "user1".into_addr();
    let user2 = "user2".into_addr();

    // Non-owner cannot transfer ownership
    let msg = ExecuteMsg::TransferOwnership {
        new_owner: user2.to_string(),
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_emergency_withdraw_native() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let admin = "admin".into_addr();
    let user1 = "user1".into_addr();

    // First, send some funds to the contract
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &contract_addr, cosmwasm_std::coins(5000, DENOM))
            .unwrap();
    });

    // Owner can emergency withdraw
    let msg = ExecuteMsg::EmergencyWithdraw {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(2000u128),
    };
    let res = app.execute_contract(admin, contract_addr, &msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn test_emergency_withdraw_unauthorized() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let user1 = "user1".into_addr();

    // Non-owner cannot emergency withdraw
    let msg = ExecuteMsg::EmergencyWithdraw {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(1000u128),
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_claim_native_token_success() {
    let mut app = App::default();

    // Generate key pair for verifier
    let (signing_key, verifying_key) = generate_test_keypair();
    let verifier_pubkey = pubkey_to_hex(&verifying_key);

    let contract_addr = setup_contract(&mut app, Some(verifier_pubkey));
    let user1 = "user1".into_addr();

    // Fund the contract
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &contract_addr, cosmwasm_std::coins(10000, DENOM))
            .unwrap();
    });

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim".to_string(),
    };

    // Verify public key was stored correctly
    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::Config {})
        .unwrap();
    let stored_pubkey_bytes = hex::decode(&config.verifier_pubkey_hex[2..]).unwrap();
    assert_eq!(
        stored_pubkey_bytes.len(),
        33,
        "Stored public key must be 33 bytes"
    );

    // Generate valid signature
    let signature =
        create_claim_with_signature(&app, &contract_addr.to_string(), &claim, &signing_key);

    // Verify signature format: should be 64 bytes (128 hex chars + "0x")
    let sig_bytes = hex::decode(&signature[2..]).unwrap();
    assert_eq!(sig_bytes.len(), 64, "Signature must be 64 bytes");

    let msg = ExecuteMsg::Claim {
        claim: claim.clone(),
        signature,
    };
    let res = app.execute_contract(user1.clone(), contract_addr.clone(), &msg, &[]);
    if let Err(e) = &res {
        println!("Claim failed: {:?}", e);
    }
    assert!(res.is_ok());

    // Verify the claim was marked as claimed
    let digest_response: GetClaimDigestResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetClaimDigest { claim })
        .unwrap();

    let is_claimed: ClaimedResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::IsClaimed {
                digest_hex: digest_response.digest_hex,
            },
        )
        .unwrap();

    assert!(is_claimed.claimed);
}

#[test]
fn test_claim_invalid_value() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let user1 = "user1".into_addr();

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::zero(), // Invalid: zero value
        deadline: 0,
        comment: "test claim".to_string(),
    };

    let msg = ExecuteMsg::Claim {
        claim,
        signature: "0x".to_string() + &"a1".repeat(64),
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_claim_expired() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let user1 = "user1".into_addr();

    // Set block time to a future time
    app.update_block(|block| {
        block.time = block.time.plus_seconds(1000);
    });

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(1000u128),
        deadline: app.block_info().time.seconds() - 100, // Expired deadline
        comment: "test claim".to_string(),
    };

    let msg = ExecuteMsg::Claim {
        claim,
        signature: "0x".to_string() + &"a1".repeat(64),
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_claim_invalid_signature() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let user1 = "user1".into_addr();

    // Fund the contract
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &contract_addr, cosmwasm_std::coins(10000, DENOM))
            .unwrap();
    });

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim".to_string(),
    };

    // Invalid signature (wrong length)
    let msg = ExecuteMsg::Claim {
        claim,
        signature: "0xinvalid".to_string(),
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_claim_wrong_signature() {
    let mut app = App::default();

    // Generate key pair for verifier
    let (_signing_key, verifying_key) = generate_test_keypair();
    let verifier_pubkey = pubkey_to_hex(&verifying_key);

    let contract_addr = setup_contract(&mut app, Some(verifier_pubkey));
    let user1 = "user1".into_addr();

    // Fund the contract
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &contract_addr, cosmwasm_std::coins(10000, DENOM))
            .unwrap();
    });

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim".to_string(),
    };

    // Use signature from a different key pair
    let (wrong_signing_key, _wrong_verifying_key) = generate_test_keypair();
    let wrong_signature =
        create_claim_with_signature(&app, &contract_addr.to_string(), &claim, &wrong_signing_key);

    let msg = ExecuteMsg::Claim {
        claim,
        signature: wrong_signature,
    };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_claim_already_claimed() {
    let mut app = App::default();

    // Generate key pair for verifier
    let (signing_key, verifying_key) = generate_test_keypair();
    let verifier_pubkey = pubkey_to_hex(&verifying_key);

    let contract_addr = setup_contract(&mut app, Some(verifier_pubkey));
    let user1 = "user1".into_addr();

    // Fund the contract
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &contract_addr, cosmwasm_std::coins(10000, DENOM))
            .unwrap();
    });

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim".to_string(),
    };

    // Generate valid signature
    let signature =
        create_claim_with_signature(&app, &contract_addr.to_string(), &claim, &signing_key);

    // First claim should succeed
    let msg = ExecuteMsg::Claim {
        claim: claim.clone(),
        signature: signature.clone(),
    };
    let res = app.execute_contract(user1.clone(), contract_addr.clone(), &msg, &[]);
    if let Err(e) = &res {
        println!("Claim failed: {:?}", e);
    }
    assert!(res.is_ok());

    // Second claim with same signature should fail
    let msg = ExecuteMsg::Claim { claim, signature };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_err());
}

#[test]
fn test_digest_computation_consistency() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);

    let claim1 = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: "user1".into_addr().to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim".to_string(),
    };

    let claim2 = claim1.clone();

    // Get digest for both claims
    let digest1: GetClaimDigestResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetClaimDigest {
                claim: claim1.clone(),
            },
        )
        .unwrap();

    let digest2: GetClaimDigestResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::GetClaimDigest { claim: claim2 })
        .unwrap();

    // Same claim should produce same digest
    assert_eq!(digest1.digest_hex, digest2.digest_hex);
}

#[test]
fn test_digest_different_claims() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);

    let claim1 = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: "user1".into_addr().to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim 1".to_string(),
    };

    let claim2 = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: "user1".into_addr().to_string(),
        value: Uint128::from(1000u128),
        deadline: 0,
        comment: "test claim 2".to_string(), // Different comment
    };

    // Get digest for both claims
    let digest1: GetClaimDigestResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetClaimDigest { claim: claim1 },
        )
        .unwrap();

    let digest2: GetClaimDigestResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::GetClaimDigest { claim: claim2 })
        .unwrap();

    // Different claims should produce different digests
    assert_ne!(digest1.digest_hex, digest2.digest_hex);
}

#[test]
fn test_claim_with_deadline() {
    let mut app = App::default();

    // Generate key pair for verifier
    let (signing_key, verifying_key) = generate_test_keypair();
    let verifier_pubkey = pubkey_to_hex(&verifying_key);

    let contract_addr = setup_contract(&mut app, Some(verifier_pubkey));
    let user1 = "user1".into_addr();

    // Fund the contract
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &contract_addr, cosmwasm_std::coins(10000, DENOM))
            .unwrap();
    });

    let claim = ClaimInfo {
        asset: AssetInfo::Native {
            denom: DENOM.to_string(),
        },
        to: user1.to_string(),
        value: Uint128::from(1000u128),
        deadline: app.block_info().time.seconds() + 3600, // Valid deadline (1 hour from now)
        comment: "test claim with deadline".to_string(),
    };

    // Generate valid signature
    let signature =
        create_claim_with_signature(&app, &contract_addr.to_string(), &claim, &signing_key);

    let msg = ExecuteMsg::Claim { claim, signature };
    let res = app.execute_contract(user1, contract_addr, &msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn test_set_verifier_invalid_pubkey_length() {
    let mut app = App::default();
    let contract_addr = setup_contract(&mut app, None);
    let admin = "admin".into_addr();

    // Invalid pubkey (wrong length)
    let invalid_pubkey = "0x1234".to_string();

    let msg = ExecuteMsg::SetVerifier {
        verifier_pubkey: invalid_pubkey,
    };
    let res = app.execute_contract(admin, contract_addr, &msg, &[]);
    assert!(res.is_err());
}
