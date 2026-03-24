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
    /// Register a new trusted credential issuer (admin only).
    RegisterIssuer {
        /// The issuer's on-chain address.
        issuer: String,
        /// Human-readable name for the issuer.
        name: String,
        /// Description of the issuer organization.
        description: String,
        /// Credential types this issuer is authorized to validate.
        credential_types: Vec<String>,
    },
    /// Revoke a previously registered issuer (admin only).
    RevokeIssuer {
        issuer: String,
        reason: String,
    },
    /// Update issuer details (admin only).
    UpdateIssuer {
        issuer: String,
        name: Option<String>,
        description: Option<String>,
        credential_types: Option<Vec<String>>,
    },
    /// Update the admin address (admin only).
    UpdateAdmin {
        new_admin: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Check if an issuer is authorized for a specific credential type.
    #[returns(IsAuthorizedResponse)]
    IsAuthorized {
        issuer: String,
        credential_type: String,
    },
    /// Get details of a specific issuer.
    #[returns(IssuerResponse)]
    Issuer {
        issuer: String,
    },
    /// List all registered issuers.
    #[returns(IssuersResponse)]
    ListIssuers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// List issuers authorized for a specific credential type.
    #[returns(IssuersResponse)]
    IssuersByType {
        credential_type: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Get the current admin address.
    #[returns(AdminResponse)]
    Admin {},
}

// ── State types ───────────────────────────────────────────────────────

#[cw_serde]
pub struct IssuerRecord {
    pub address: Addr,
    pub name: String,
    pub description: String,
    pub credential_types: Vec<String>,
    pub registered_at: u64,
    pub active: bool,
    pub revoked_at: Option<u64>,
    pub revocation_reason: Option<String>,
}

// ── Response types ────────────────────────────────────────────────────

#[cw_serde]
pub struct IsAuthorizedResponse {
    pub authorized: bool,
    pub issuer_name: Option<String>,
}

#[cw_serde]
pub struct IssuerResponse {
    pub issuer: IssuerRecord,
}

#[cw_serde]
pub struct IssuersResponse {
    pub issuers: Vec<IssuerRecord>,
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr,
}
