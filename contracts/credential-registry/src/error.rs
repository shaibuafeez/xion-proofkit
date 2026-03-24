use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: only the admin can perform this action")]
    Unauthorized,

    #[error("Schema '{schema_id}' already exists")]
    SchemaAlreadyExists { schema_id: String },

    #[error("Schema '{schema_id}' not found")]
    SchemaNotFound { schema_id: String },

    #[error("Proof record {proof_id} not found")]
    ProofNotFound { proof_id: u64 },

    #[error("Proof record {proof_id} is already revoked")]
    ProofAlreadyRevoked { proof_id: u64 },

    #[error("Schema ID exceeds maximum length of {max} characters")]
    SchemaIdTooLong { max: usize },

    #[error("Schema ID cannot be empty")]
    EmptySchemaId,

    #[error("Name cannot be empty")]
    EmptyName,

    #[error("Description exceeds maximum length of {max} characters")]
    DescriptionTooLong { max: usize },

    #[error("Proof hash cannot be empty")]
    EmptyProofHash,

    #[error("Credential types list cannot be empty")]
    EmptyCredentialTypes,

    #[error("Revocation reason cannot be empty")]
    EmptyRevocationReason,

    #[error("Only the verifier contract or admin can record proofs")]
    UnauthorizedRecorder,
}
