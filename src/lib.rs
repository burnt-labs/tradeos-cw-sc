pub mod execute;
pub mod msg;
pub mod state;
pub mod error;
pub mod query;

// Only include contract.rs (which now hosts the entrypoints) when not building as a library
#[cfg(not(feature = "library"))]
pub mod contract;

pub const CONTRACT_NAME: &str = "tradeos-cw-sc";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

