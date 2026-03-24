use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: only the admin can perform this action")]
    Unauthorized,

    #[error("Schema ID cannot be empty")]
    EmptySchemaId,

    #[error("Proof data cannot be empty")]
    EmptyProof,

    #[error("Public inputs cannot be empty")]
    EmptyPublicInputs,

    #[error("DKIM signature cannot be empty")]
    EmptyDkimSignature,

    #[error("Email headers cannot be empty")]
    EmptyEmailHeaders,

    #[error("Email domain cannot be empty")]
    EmptyEmailDomain,

    #[error("Batch verification list cannot be empty")]
    EmptyBatch,

    #[error("Batch size exceeds maximum of {max}")]
    BatchTooLarge { max: usize },

    #[error("Verification record {id} not found")]
    VerificationNotFound { id: u64 },

    #[error("ZK proof verification failed: {reason}")]
    ZkVerificationFailed { reason: String },

    #[error("DKIM verification failed: {reason}")]
    DkimVerificationFailed { reason: String },

    #[error("Issuer '{issuer}' is not authorized for credential type '{credential_type}'")]
    IssuerNotAuthorized {
        issuer: String,
        credential_type: String,
    },
}
