use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use proofkit_types::verifier::{Config, VerificationRecord};

/// Contract configuration (admin, linked registries).
pub const CONFIG: Item<Config> = Item::new("config");

/// Auto-incrementing counter for verification record IDs.
pub const VERIFICATION_COUNT: Item<u64> = Item::new("verification_count");

/// Verification records indexed by auto-incrementing ID.
pub const VERIFICATIONS: Map<u64, VerificationRecord> = Map::new("verifications");

/// Secondary index: (subject_addr, verification_id) → () for querying by subject.
pub const VERIFICATIONS_BY_SUBJECT: Map<(&Addr, u64), ()> = Map::new("verifications_by_subject");
