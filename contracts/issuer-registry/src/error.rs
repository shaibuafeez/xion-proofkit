use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: only the admin can perform this action")]
    Unauthorized,

    #[error("Issuer '{issuer}' is already registered")]
    IssuerAlreadyRegistered { issuer: String },

    #[error("Issuer '{issuer}' not found")]
    IssuerNotFound { issuer: String },

    #[error("Issuer '{issuer}' is already revoked")]
    IssuerAlreadyRevoked { issuer: String },

    #[error("Issuer name cannot be empty")]
    EmptyName,

    #[error("Description exceeds maximum length of {max} characters")]
    DescriptionTooLong { max: usize },

    #[error("Credential types list cannot be empty")]
    EmptyCredentialTypes,

    #[error("Revocation reason cannot be empty")]
    EmptyRevocationReason,

    #[error("Update must change at least one field")]
    NoFieldsToUpdate,
}
