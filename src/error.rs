use cosmwasm_std::{StdError, VerificationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Verification error: {0}")]
    Verification(#[from] VerificationError),
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
