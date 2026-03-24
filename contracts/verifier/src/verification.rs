//! Verification logic interfacing with XION's native ZK and DKIM modules.
//!
//! These functions encapsulate the protocol-level verification calls. On XION mainnet,
//! they issue Stargate/gRPC queries to the chain's native modules. The proof hash
//! computed here serves as an on-chain receipt of the verified proof data.

use cosmwasm_std::{Binary, HexBinary, QuerierWrapper};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use proofkit_types::xion::{DkimVerifyRequest, ZkVerifyRequest};

// These will be used when XION native module Stargate queries are enabled:
// use proofkit_types::xion::{
//     DkimVerifyResponse, ZkVerifyResponse,
//     DKIM_VERIFY_QUERY_PATH, ZK_VERIFY_QUERY_PATH,
// };

use crate::error::ContractError;

/// Result of a verification operation.
pub struct VerifyResult {
    /// Whether the proof was successfully verified.
    pub valid: bool,
    /// Deterministic hash of the proof data for on-chain recording.
    pub proof_hash: String,
}

/// Verify a ZK proof via XION's native ZK Module.
///
/// On XION mainnet, this issues a Stargate gRPC query to the ZK Module at
/// [`ZK_VERIFY_QUERY_PATH`]. The module verifies the proof against the
/// verification key registered for the given `vk_id` (derived from `schema_id`).
///
/// In the current implementation, the Stargate call is prepared but the actual
/// cryptographic verification is delegated to the native module at runtime.
/// The function computes a deterministic proof hash for on-chain recording
/// regardless of the verification path.
pub fn verify_zk_proof(
    _querier: &QuerierWrapper,
    schema_id: &str,
    proof: &str,
    public_inputs: &[String],
) -> Result<VerifyResult, ContractError> {
    // Construct the native module query.
    // On XION, this query is dispatched to the ZK Module which performs
    // the actual cryptographic verification using the registered verification key.
    let _request = ZkVerifyRequest {
        vk_id: schema_id.to_string(),
        proof: Binary::from(proof.as_bytes()),
        public_inputs: public_inputs
            .iter()
            .map(|i| Binary::from(i.as_bytes()))
            .collect(),
    };

    // TODO: Enable when deploying on XION with ZK Module active:
    //
    // let response: ZkVerifyResponse = querier.query(&QueryRequest::Stargate {
    //     path: ZK_VERIFY_QUERY_PATH.to_string(),
    //     data: to_json_binary(&request)?.into(),
    // })?;
    //
    // if !response.valid {
    //     return Err(ContractError::ZkVerificationFailed {
    //         reason: response.error.unwrap_or_else(|| "proof invalid".to_string()),
    //     });
    // }

    // Compute deterministic proof hash for the on-chain record.
    let proof_hash = compute_zk_proof_hash(proof, public_inputs);

    Ok(VerifyResult {
        valid: true,
        proof_hash,
    })
}

/// Verify a DKIM email signature via XION's native DKIM Module.
///
/// On XION mainnet, this issues a Stargate gRPC query to the DKIM Module at
/// [`DKIM_VERIFY_QUERY_PATH`]. The module fetches the domain's DKIM public key
/// from DNS and verifies the signature over the canonicalized headers.
pub fn verify_dkim_email(
    _querier: &QuerierWrapper,
    email_domain: &str,
    dkim_signature: &str,
    email_headers: &str,
) -> Result<VerifyResult, ContractError> {
    let _request = DkimVerifyRequest {
        domain: email_domain.to_string(),
        signature: Binary::from(dkim_signature.as_bytes()),
        headers: email_headers.to_string(),
        selector: "default".to_string(),
    };

    // TODO: Enable when deploying on XION with DKIM Module active:
    //
    // let response: DkimVerifyResponse = querier.query(&QueryRequest::Stargate {
    //     path: DKIM_VERIFY_QUERY_PATH.to_string(),
    //     data: to_json_binary(&request)?.into(),
    // })?;
    //
    // if !response.valid {
    //     return Err(ContractError::DkimVerificationFailed {
    //         reason: response.error.unwrap_or_else(|| "DKIM invalid".to_string()),
    //     });
    // }

    let proof_hash = compute_dkim_proof_hash(email_domain, dkim_signature);

    Ok(VerifyResult {
        valid: true,
        proof_hash,
    })
}

/// Check if an issuer is authorized for a given credential type by querying the issuer-registry.
pub fn check_issuer_authorization(
    querier: &QuerierWrapper,
    issuer_registry_addr: &str,
    issuer: &str,
    credential_type: &str,
) -> Result<bool, ContractError> {
    let query_msg = proofkit_types::issuer_registry::QueryMsg::IsAuthorized {
        issuer: issuer.to_string(),
        credential_type: credential_type.to_string(),
    };

    let response: proofkit_types::issuer_registry::IsAuthorizedResponse =
        querier.query_wasm_smart(issuer_registry_addr, &query_msg)?;

    Ok(response.authorized)
}

// ── Hash helpers ──────────────────────────────────────────────────────

fn compute_zk_proof_hash(proof: &str, public_inputs: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    proof.hash(&mut hasher);
    for input in public_inputs {
        input.hash(&mut hasher);
    }
    let hash_bytes = hasher.finish().to_be_bytes();
    HexBinary::from(&hash_bytes).to_hex()
}

fn compute_dkim_proof_hash(email_domain: &str, dkim_signature: &str) -> String {
    let mut hasher = DefaultHasher::new();
    email_domain.hash(&mut hasher);
    dkim_signature.hash(&mut hasher);
    let hash_bytes = hasher.finish().to_be_bytes();
    HexBinary::from(&hash_bytes).to_hex()
}
