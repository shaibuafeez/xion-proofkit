//! Types for interfacing with XION's native Cosmos SDK modules.
//!
//! XION provides two protocol-level modules that ProofKit leverages:
//! - **ZK Module**: Verifies zero-knowledge proofs on-chain
//! - **DKIM Module**: Verifies DKIM email signatures on-chain
//!
//! These are accessed via Stargate queries from CosmWasm contracts.

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, CustomQuery, QueryRequest};

// ── ZK Module ─────────────────────────────────────────────────────────

/// Stargate type URL for ZK proof verification queries.
pub const ZK_VERIFY_QUERY_PATH: &str = "/xion.zkmodule.v1.Query/VerifyProof";

/// Request payload for XION ZK Module verification.
/// Sent as a Stargate query to the native ZK module.
#[cw_serde]
pub struct ZkVerifyRequest {
    /// The verification key identifier registered with the ZK module.
    pub vk_id: String,
    /// The ZK proof bytes (base64-encoded).
    pub proof: Binary,
    /// Public inputs for the ZK circuit (base64-encoded values).
    pub public_inputs: Vec<Binary>,
}

/// Response from XION ZK Module verification.
#[cw_serde]
pub struct ZkVerifyResponse {
    /// Whether the proof is valid.
    pub valid: bool,
    /// Optional error message if verification failed.
    pub error: Option<String>,
}

// ── DKIM Module ───────────────────────────────────────────────────────

/// Stargate type URL for DKIM verification queries.
pub const DKIM_VERIFY_QUERY_PATH: &str = "/xion.dkim.v1.Query/VerifyDkim";

/// Request payload for XION DKIM Module verification.
/// Sent as a Stargate query to the native DKIM module.
#[cw_serde]
pub struct DkimVerifyRequest {
    /// The email domain to verify (e.g., "company.com").
    pub domain: String,
    /// The DKIM signature from the email headers (base64-encoded).
    pub signature: Binary,
    /// The canonicalized email headers used for DKIM signing.
    pub headers: String,
    /// The DKIM selector (e.g., "google", "default").
    pub selector: String,
}

/// Response from XION DKIM Module verification.
#[cw_serde]
pub struct DkimVerifyResponse {
    /// Whether the DKIM signature is valid.
    pub valid: bool,
    /// The verified email domain.
    pub domain: String,
    /// Optional error message if verification failed.
    pub error: Option<String>,
}

// ── Custom query type for XION ────────────────────────────────────────

/// Custom query enum for XION-specific Stargate queries.
/// Used with `QuerierWrapper<XionQuery>` for type-safe native module access.
#[cw_serde]
pub enum XionQuery {
    VerifyZkProof(ZkVerifyRequest),
    VerifyDkim(DkimVerifyRequest),
}

impl CustomQuery for XionQuery {}

// ── Helper constructors ───────────────────────────────────────────────

/// Build a Stargate query request for ZK proof verification.
pub fn build_zk_verify_query(request: &ZkVerifyRequest) -> QueryRequest<XionQuery> {
    QueryRequest::Custom(XionQuery::VerifyZkProof(request.clone()))
}

/// Build a Stargate query request for DKIM verification.
pub fn build_dkim_verify_query(request: &DkimVerifyRequest) -> QueryRequest<XionQuery> {
    QueryRequest::Custom(XionQuery::VerifyDkim(request.clone()))
}
