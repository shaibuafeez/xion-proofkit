pub mod credential_registry;
pub mod issuer_registry;
pub mod verifier;
pub mod xion;

/// Maximum length for string identifiers (schema IDs, issuer names, etc.)
pub const MAX_ID_LENGTH: usize = 128;
/// Maximum length for description/metadata fields
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;
/// Default pagination limit
pub const DEFAULT_LIMIT: u32 = 10;
/// Maximum pagination limit
pub const MAX_LIMIT: u32 = 50;
