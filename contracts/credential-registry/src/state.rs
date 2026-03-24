use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use proofkit_types::credential_registry::{CredentialSchema, ProofRecord};

/// Contract admin address.
pub const ADMIN: Item<Addr> = Item::new("admin");

/// Auto-incrementing counter for proof record IDs.
pub const PROOF_COUNT: Item<u64> = Item::new("proof_count");

/// Credential schemas indexed by schema_id.
pub const SCHEMAS: Map<&str, CredentialSchema> = Map::new("schemas");

/// Proof records indexed by auto-incrementing ID.
pub const PROOFS: Map<u64, ProofRecord> = Map::new("proofs");

/// Secondary index: (subject_addr, proof_id) → () for querying by subject.
pub const PROOFS_BY_SUBJECT: Map<(&Addr, u64), ()> = Map::new("proofs_by_subject");

/// Secondary index: (schema_id, subject_addr) → latest proof_id for quick verification lookup.
pub const LATEST_PROOF: Map<(&str, &Addr), u64> = Map::new("latest_proof");
