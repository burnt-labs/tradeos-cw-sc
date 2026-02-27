pub mod error;
pub mod execute;
pub mod helpers;
pub mod msg;
pub mod query;
pub mod state;

// Only include contract.rs (which now hosts the entrypoints) when not building as a library
#[cfg(not(feature = "library"))]
pub mod contract;

pub const CONTRACT_NAME: &str = "tradeos-cw-sc";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
