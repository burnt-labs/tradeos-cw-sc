use cw_storage_plus::{Item, Map};

/// compressed secp256k1 pubkey (33 bytes)
pub const VERIFIER_PUBKEY: Item<Vec<u8>> = Item::new("verifier_pubkey");
/// map: digest (32 bytes) -> bool
pub const CLAIMED: Map<&[u8], bool> = Map::new("claimed");
