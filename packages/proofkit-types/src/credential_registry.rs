use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

// ── Messages ──────────────────────────────────────────────────────────

#[cw_serde]
pub struct InstantiateMsg {
    /// Optional admin address; defaults to the instantiator.
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Register a new credential schema (admin only).
    RegisterSchema {
        /// Unique identifier for the schema (e.g., "age_verification").
        schema_id: String,
        /// Human-readable name.
        name: String,
        /// Description of what this credential proves.
        description: String,
        /// Address of the verifier contract that validates proofs for this schema.
        verifier_contract: String,
        /// The credential types this schema supports (e.g., ["age", "identity"]).
        credential_types: Vec<String>,
    },
    /// Record a verified proof on-chain (called by the verifier contract).
    RecordProof {
        /// Schema this proof belongs to.
        schema_id: String,
        /// The address whose credential was verified.
        subject: String,
        /// Hash of the proof data for auditability.
        proof_hash: String,
        /// Address of the issuer who signed the credential.
        issuer: String,
        /// Optional expiration timestamp (seconds since epoch).
        expires_at: Option<u64>,
    },
    /// Revoke a previously recorded proof (issuer or admin only).
    RevokeProof {
        /// The proof record ID to revoke.
        proof_id: u64,
        /// Reason for revocation.
        reason: String,
    },
    /// Update the admin address (admin only).
    UpdateAdmin {
        new_admin: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Check if a subject has a valid (non-revoked, non-expired) proof for a schema.
    #[returns(IsVerifiedResponse)]
    IsVerified {
        subject: String,
        schema_id: String,
    },
    /// Get a specific proof record by ID.
    #[returns(ProofRecordResponse)]
    ProofRecord {
        proof_id: u64,
    },
    /// List all proof records for a subject.
    #[returns(ProofRecordsResponse)]
    ProofsBySubject {
        subject: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Get schema details.
    #[returns(SchemaResponse)]
    Schema {
        schema_id: String,
    },
    /// List all registered schemas.
    #[returns(SchemasResponse)]
    ListSchemas {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Get the current admin address.
    #[returns(AdminResponse)]
    Admin {},
}

// ── State types ───────────────────────────────────────────────────────

#[cw_serde]
pub struct CredentialSchema {
    pub schema_id: String,
    pub name: String,
    pub description: String,
    pub verifier_contract: Addr,
    pub credential_types: Vec<String>,
    pub created_at: u64,
    pub active: bool,
}

#[cw_serde]
pub struct ProofRecord {
    pub id: u64,
    pub schema_id: String,
    pub subject: Addr,
    pub proof_hash: String,
    pub issuer: Addr,
    pub verified_at: u64,
    pub expires_at: Option<u64>,
    pub revoked: bool,
    pub revoked_at: Option<u64>,
    pub revocation_reason: Option<String>,
}

// ── Response types ────────────────────────────────────────────────────

#[cw_serde]
pub struct IsVerifiedResponse {
    pub verified: bool,
    pub proof_id: Option<u64>,
    pub expires_at: Option<u64>,
}

#[cw_serde]
pub struct ProofRecordResponse {
    pub record: ProofRecord,
}

#[cw_serde]
pub struct ProofRecordsResponse {
    pub records: Vec<ProofRecord>,
}

#[cw_serde]
pub struct SchemaResponse {
    pub schema: CredentialSchema,
}

#[cw_serde]
pub struct SchemasResponse {
    pub schemas: Vec<CredentialSchema>,
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr,
}
