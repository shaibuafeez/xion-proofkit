use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use proofkit_types::credential_registry::ExecuteMsg as RegistryExecuteMsg;
use proofkit_types::verifier::*;
use proofkit_types::{DEFAULT_LIMIT, MAX_LIMIT};

use crate::error::ContractError;
use crate::state::*;
use crate::verification;

const CONTRACT_NAME: &str = "crates.io:proofkit-verifier";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum number of verifications in a single batch.
const MAX_BATCH_SIZE: usize = 20;

// ── Instantiate ───────────────────────────────────────────────────────

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = msg
        .admin
        .map(|a| deps.api.addr_validate(&a))
        .transpose()?
        .unwrap_or(info.sender);

    let credential_registry = deps.api.addr_validate(&msg.credential_registry)?;
    let issuer_registry = deps.api.addr_validate(&msg.issuer_registry)?;

    let config = Config {
        admin: admin.clone(),
        credential_registry,
        issuer_registry,
    };

    CONFIG.save(deps.storage, &config)?;
    VERIFICATION_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin.as_str()))
}

// ── Execute ───────────────────────────────────────────────────────────

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::VerifyCredential {
            schema_id,
            subject,
            issuer,
            proof,
            public_inputs,
            expires_at,
        } => execute_verify_credential(
            deps,
            env,
            info,
            schema_id,
            subject,
            issuer,
            proof,
            public_inputs,
            expires_at,
        ),
        ExecuteMsg::VerifyEmailCredential {
            schema_id,
            subject,
            issuer,
            email_domain,
            dkim_signature,
            email_headers,
            expires_at,
        } => execute_verify_email_credential(
            deps,
            env,
            info,
            schema_id,
            subject,
            issuer,
            email_domain,
            dkim_signature,
            email_headers,
            expires_at,
        ),
        ExecuteMsg::BatchVerify { verifications } => {
            execute_batch_verify(deps, env, info, verifications)
        }
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, info, new_admin),
        ExecuteMsg::UpdateRegistry {
            credential_registry,
        } => execute_update_registry(deps, info, credential_registry),
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_verify_credential(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    schema_id: String,
    subject: String,
    issuer: String,
    proof: String,
    public_inputs: Vec<String>,
    expires_at: Option<u64>,
) -> Result<Response, ContractError> {
    if schema_id.is_empty() {
        return Err(ContractError::EmptySchemaId);
    }
    if proof.is_empty() {
        return Err(ContractError::EmptyProof);
    }
    if public_inputs.is_empty() {
        return Err(ContractError::EmptyPublicInputs);
    }

    let config = CONFIG.load(deps.storage)?;
    let subject_addr = deps.api.addr_validate(&subject)?;
    let issuer_addr = deps.api.addr_validate(&issuer)?;

    // Verify the ZK proof via XION's native ZK Module
    let result = verification::verify_zk_proof(
        &deps.querier,
        &schema_id,
        &proof,
        &public_inputs,
    )?;

    // Record the verification
    let id = VERIFICATION_COUNT.load(deps.storage)? + 1;
    VERIFICATION_COUNT.save(deps.storage, &id)?;

    let record = VerificationRecord {
        id,
        schema_id: schema_id.clone(),
        subject: subject_addr.clone(),
        issuer: issuer_addr.clone(),
        verification_type: VerificationType::ZkProof,
        verified: result.valid,
        verified_at: env.block.time.seconds(),
        proof_hash: result.proof_hash.clone(),
    };

    VERIFICATIONS.save(deps.storage, id, &record)?;
    VERIFICATIONS_BY_SUBJECT.save(deps.storage, (&subject_addr, id), &())?;

    // Send RecordProof to the credential registry
    let record_msg = WasmMsg::Execute {
        contract_addr: config.credential_registry.to_string(),
        msg: to_json_binary(&RegistryExecuteMsg::RecordProof {
            schema_id: schema_id.clone(),
            subject: subject_addr.to_string(),
            proof_hash: result.proof_hash,
            issuer: issuer_addr.to_string(),
            expires_at,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_submessage(SubMsg::new(record_msg))
        .add_attribute("action", "verify_credential")
        .add_attribute("verification_id", id.to_string())
        .add_attribute("schema_id", &schema_id)
        .add_attribute("subject", subject_addr.as_str())
        .add_attribute("verification_type", "zk_proof")
        .add_attribute("verified", result.valid.to_string()))
}

#[allow(clippy::too_many_arguments)]
fn execute_verify_email_credential(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    schema_id: String,
    subject: String,
    issuer: String,
    email_domain: String,
    dkim_signature: String,
    email_headers: String,
    expires_at: Option<u64>,
) -> Result<Response, ContractError> {
    if schema_id.is_empty() {
        return Err(ContractError::EmptySchemaId);
    }
    if email_domain.is_empty() {
        return Err(ContractError::EmptyEmailDomain);
    }
    if dkim_signature.is_empty() {
        return Err(ContractError::EmptyDkimSignature);
    }
    if email_headers.is_empty() {
        return Err(ContractError::EmptyEmailHeaders);
    }

    let config = CONFIG.load(deps.storage)?;
    let subject_addr = deps.api.addr_validate(&subject)?;
    let issuer_addr = deps.api.addr_validate(&issuer)?;

    // Verify the DKIM signature via XION's native DKIM Module
    let result = verification::verify_dkim_email(
        &deps.querier,
        &email_domain,
        &dkim_signature,
        &email_headers,
    )?;

    // Record the verification
    let id = VERIFICATION_COUNT.load(deps.storage)? + 1;
    VERIFICATION_COUNT.save(deps.storage, &id)?;

    let record = VerificationRecord {
        id,
        schema_id: schema_id.clone(),
        subject: subject_addr.clone(),
        issuer: issuer_addr.clone(),
        verification_type: VerificationType::EmailDkim,
        verified: result.valid,
        verified_at: env.block.time.seconds(),
        proof_hash: result.proof_hash.clone(),
    };

    VERIFICATIONS.save(deps.storage, id, &record)?;
    VERIFICATIONS_BY_SUBJECT.save(deps.storage, (&subject_addr, id), &())?;

    // Send RecordProof to the credential registry
    let record_msg = WasmMsg::Execute {
        contract_addr: config.credential_registry.to_string(),
        msg: to_json_binary(&RegistryExecuteMsg::RecordProof {
            schema_id: schema_id.clone(),
            subject: subject_addr.to_string(),
            proof_hash: result.proof_hash,
            issuer: issuer_addr.to_string(),
            expires_at,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_submessage(SubMsg::new(record_msg))
        .add_attribute("action", "verify_email_credential")
        .add_attribute("verification_id", id.to_string())
        .add_attribute("schema_id", &schema_id)
        .add_attribute("subject", subject_addr.as_str())
        .add_attribute("email_domain", &email_domain)
        .add_attribute("verification_type", "email_dkim")
        .add_attribute("verified", result.valid.to_string()))
}

fn execute_batch_verify(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    verifications: Vec<VerificationRequest>,
) -> Result<Response, ContractError> {
    if verifications.is_empty() {
        return Err(ContractError::EmptyBatch);
    }
    if verifications.len() > MAX_BATCH_SIZE {
        return Err(ContractError::BatchTooLarge {
            max: MAX_BATCH_SIZE,
        });
    }

    let config = CONFIG.load(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "batch_verify");
    let mut results: Vec<VerificationResult> = Vec::with_capacity(verifications.len());

    for (idx, req) in verifications.into_iter().enumerate() {
        match req {
            VerificationRequest::ZkProof {
                schema_id,
                subject,
                issuer,
                proof,
                public_inputs,
                expires_at,
            } => {
                if schema_id.is_empty() || proof.is_empty() || public_inputs.is_empty() {
                    results.push(VerificationResult {
                        verified: false,
                        verification_id: 0,
                        schema_id,
                        subject,
                        verification_type: VerificationType::ZkProof,
                        message: "Invalid input: empty required fields".to_string(),
                    });
                    continue;
                }

                let subject_addr = deps.api.addr_validate(&subject)?;
                let issuer_addr = deps.api.addr_validate(&issuer)?;

                let vr = verification::verify_zk_proof(
                    &deps.querier,
                    &schema_id,
                    &proof,
                    &public_inputs,
                )?;

                let id = VERIFICATION_COUNT.load(deps.storage)? + 1;
                VERIFICATION_COUNT.save(deps.storage, &id)?;

                let record = VerificationRecord {
                    id,
                    schema_id: schema_id.clone(),
                    subject: subject_addr.clone(),
                    issuer: issuer_addr.clone(),
                    verification_type: VerificationType::ZkProof,
                    verified: vr.valid,
                    verified_at: env.block.time.seconds(),
                    proof_hash: vr.proof_hash.clone(),
                };

                VERIFICATIONS.save(deps.storage, id, &record)?;
                VERIFICATIONS_BY_SUBJECT.save(deps.storage, (&subject_addr, id), &())?;

                let record_msg = WasmMsg::Execute {
                    contract_addr: config.credential_registry.to_string(),
                    msg: to_json_binary(&RegistryExecuteMsg::RecordProof {
                        schema_id: schema_id.clone(),
                        subject: subject_addr.to_string(),
                        proof_hash: vr.proof_hash,
                        issuer: issuer_addr.to_string(),
                        expires_at,
                    })?,
                    funds: vec![],
                };
                response = response.add_submessage(SubMsg::new(record_msg));

                results.push(VerificationResult {
                    verified: true,
                    verification_id: id,
                    schema_id,
                    subject: subject_addr.to_string(),
                    verification_type: VerificationType::ZkProof,
                    message: "ZK proof verified successfully".to_string(),
                });
            }
            VerificationRequest::EmailProof {
                schema_id,
                subject,
                issuer,
                email_domain,
                dkim_signature,
                email_headers,
                expires_at,
            } => {
                if schema_id.is_empty()
                    || email_domain.is_empty()
                    || dkim_signature.is_empty()
                    || email_headers.is_empty()
                {
                    results.push(VerificationResult {
                        verified: false,
                        verification_id: 0,
                        schema_id,
                        subject,
                        verification_type: VerificationType::EmailDkim,
                        message: "Invalid input: empty required fields".to_string(),
                    });
                    continue;
                }

                let subject_addr = deps.api.addr_validate(&subject)?;
                let issuer_addr = deps.api.addr_validate(&issuer)?;

                let vr = verification::verify_dkim_email(
                    &deps.querier,
                    &email_domain,
                    &dkim_signature,
                    &email_headers,
                )?;

                let id = VERIFICATION_COUNT.load(deps.storage)? + 1;
                VERIFICATION_COUNT.save(deps.storage, &id)?;

                let record = VerificationRecord {
                    id,
                    schema_id: schema_id.clone(),
                    subject: subject_addr.clone(),
                    issuer: issuer_addr.clone(),
                    verification_type: VerificationType::EmailDkim,
                    verified: vr.valid,
                    verified_at: env.block.time.seconds(),
                    proof_hash: vr.proof_hash.clone(),
                };

                VERIFICATIONS.save(deps.storage, id, &record)?;
                VERIFICATIONS_BY_SUBJECT.save(deps.storage, (&subject_addr, id), &())?;

                let record_msg = WasmMsg::Execute {
                    contract_addr: config.credential_registry.to_string(),
                    msg: to_json_binary(&RegistryExecuteMsg::RecordProof {
                        schema_id: schema_id.clone(),
                        subject: subject_addr.to_string(),
                        proof_hash: vr.proof_hash,
                        issuer: issuer_addr.to_string(),
                        expires_at,
                    })?,
                    funds: vec![],
                };
                response = response.add_submessage(SubMsg::new(record_msg));

                results.push(VerificationResult {
                    verified: true,
                    verification_id: id,
                    schema_id,
                    subject: subject_addr.to_string(),
                    verification_type: VerificationType::EmailDkim,
                    message: "DKIM email verified successfully".to_string(),
                });
            }
        }

        response = response.add_attribute(
            format!("batch_item_{}", idx),
            if results.last().map_or(false, |r| r.verified) {
                "verified"
            } else {
                "failed"
            },
        );
    }

    let batch_response = BatchVerifyResponse { results };
    response = response.set_data(to_json_binary(&batch_response)?);

    Ok(response)
}

fn execute_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let new_admin_addr = deps.api.addr_validate(&new_admin)?;
    CONFIG.update(deps.storage, |mut c| -> StdResult<_> {
        c.admin = new_admin_addr.clone();
        Ok(c)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("new_admin", new_admin_addr.as_str()))
}

fn execute_update_registry(
    deps: DepsMut,
    info: MessageInfo,
    credential_registry: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let new_registry = deps.api.addr_validate(&credential_registry)?;
    CONFIG.update(deps.storage, |mut c| -> StdResult<_> {
        c.credential_registry = new_registry.clone();
        Ok(c)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_registry")
        .add_attribute("credential_registry", new_registry.as_str()))
}

// ── Query ─────────────────────────────────────────────────────────────

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerificationResult { verification_id } => {
            to_json_binary(&query_verification_result(deps, verification_id)?)
        }
        QueryMsg::VerificationsBySubject {
            subject,
            start_after,
            limit,
        } => to_json_binary(&query_verifications_by_subject(
            deps,
            subject,
            start_after,
            limit,
        )?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_verification_result(
    deps: Deps,
    verification_id: u64,
) -> StdResult<VerificationResultResponse> {
    let record = VERIFICATIONS.load(deps.storage, verification_id)?;
    Ok(VerificationResultResponse { result: record })
}

fn query_verifications_by_subject(
    deps: Deps,
    subject: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<VerificationResultsResponse> {
    let subject_addr = deps.api.addr_validate(&subject)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let results: Vec<VerificationRecord> = VERIFICATIONS_BY_SUBJECT
        .prefix(&subject_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (id, _) = item?;
            VERIFICATIONS.load(deps.storage, id)
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(VerificationResultsResponse { results })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}
