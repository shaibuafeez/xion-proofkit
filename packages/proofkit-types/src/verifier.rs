use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

// ── Messages ──────────────────────────────────────────────────────────

#[cw_serde]
pub struct InstantiateMsg {
    /// Optional admin address; defaults to the instantiator.
    pub admin: Option<String>,
    /// Address of the credential registry contract for recording proofs.
    pub credential_registry: String,
    /// Address of the issuer registry contract for authorization checks.
    pub issuer_registry: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Verify a ZK credential proof via XION's ZK Module.
    VerifyCredential {
        /// The schema to verify against.
        schema_id: String,
        /// The subject whose credential is being verified.
        subject: String,
        /// The issuer who signed the credential.
        issuer: String,
        /// Raw ZK proof bytes (base64-encoded).
        proof: String,
        /// Public inputs for the ZK verification (base64-encoded).
        public_inputs: Vec<String>,
        /// Optional expiration for the proof record.
        expires_at: Option<u64>,
    },
    /// Verify an email-based credential using XION's DKIM Module.
    VerifyEmailCredential {
        /// The schema to verify against.
        schema_id: String,
        /// The subject whose credential is being verified.
        subject: String,
        /// The issuer who signed the credential.
        issuer: String,
        /// The email domain to verify (e.g., "company.com").
        email_domain: String,
        /// DKIM signature data (base64-encoded).
        dkim_signature: String,
        /// The email headers used for DKIM verification.
        email_headers: String,
        /// Optional expiration for the proof record.
        expires_at: Option<u64>,
    },
    /// Batch-verify multiple credentials in a single transaction.
    BatchVerify {
        verifications: Vec<VerificationRequest>,
    },
    /// Update the admin address (admin only).
    UpdateAdmin {
        new_admin: String,
    },
    /// Update the credential registry address (admin only).
    UpdateRegistry {
        credential_registry: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get a specific verification result by ID.
    #[returns(VerificationResultResponse)]
    VerificationResult {
        verification_id: u64,
    },
    /// List verification results for a subject.
    #[returns(VerificationResultsResponse)]
    VerificationsBySubject {
        subject: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Get the current contract configuration.
    #[returns(ConfigResponse)]
    Config {},
}

// ── Verification request for batch operations ────────────────────────

#[cw_serde]
pub enum VerificationRequest {
    ZkProof {
        schema_id: String,
        subject: String,
        issuer: String,
        proof: String,
        public_inputs: Vec<String>,
        expires_at: Option<u64>,
    },
    EmailProof {
        schema_id: String,
        subject: String,
        issuer: String,
        email_domain: String,
        dkim_signature: String,
        email_headers: String,
        expires_at: Option<u64>,
    },
}

// ── State types ───────────────────────────────────────────────────────

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub credential_registry: Addr,
    pub issuer_registry: Addr,
}

#[cw_serde]
pub struct VerificationRecord {
    pub id: u64,
    pub schema_id: String,
    pub subject: Addr,
    pub issuer: Addr,
    pub verification_type: VerificationType,
    pub verified: bool,
    pub verified_at: u64,
    pub proof_hash: String,
}

#[cw_serde]
pub enum VerificationType {
    ZkProof,
    EmailDkim,
}

// ── Response types ────────────────────────────────────────────────────

#[cw_serde]
pub struct VerificationResult {
    pub verified: bool,
    pub verification_id: u64,
    pub schema_id: String,
    pub subject: String,
    pub verification_type: VerificationType,
    pub message: String,
}

#[cw_serde]
pub struct VerificationResultResponse {
    pub result: VerificationRecord,
}

#[cw_serde]
pub struct VerificationResultsResponse {
    pub results: Vec<VerificationRecord>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct BatchVerifyResponse {
    pub results: Vec<VerificationResult>,
}
