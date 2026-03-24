use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use proofkit_types::credential_registry::*;
use proofkit_types::{DEFAULT_LIMIT, MAX_DESCRIPTION_LENGTH, MAX_ID_LENGTH, MAX_LIMIT};

use crate::error::ContractError;
use crate::state::*;

const CONTRACT_NAME: &str = "crates.io:proofkit-credential-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    ADMIN.save(deps.storage, &admin)?;
    PROOF_COUNT.save(deps.storage, &0u64)?;

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
        ExecuteMsg::RegisterSchema {
            schema_id,
            name,
            description,
            verifier_contract,
            credential_types,
        } => execute_register_schema(
            deps,
            env,
            info,
            schema_id,
            name,
            description,
            verifier_contract,
            credential_types,
        ),
        ExecuteMsg::RecordProof {
            schema_id,
            subject,
            proof_hash,
            issuer,
            expires_at,
        } => execute_record_proof(deps, env, info, schema_id, subject, proof_hash, issuer, expires_at),
        ExecuteMsg::RevokeProof { proof_id, reason } => {
            execute_revoke_proof(deps, env, info, proof_id, reason)
        }
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, info, new_admin),
    }
}

fn execute_register_schema(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    schema_id: String,
    name: String,
    description: String,
    verifier_contract: String,
    credential_types: Vec<String>,
) -> Result<Response, ContractError> {
    // Access control
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized);
    }

    // Validate inputs
    if schema_id.is_empty() {
        return Err(ContractError::EmptySchemaId);
    }
    if schema_id.len() > MAX_ID_LENGTH {
        return Err(ContractError::SchemaIdTooLong { max: MAX_ID_LENGTH });
    }
    if name.is_empty() {
        return Err(ContractError::EmptyName);
    }
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(ContractError::DescriptionTooLong {
            max: MAX_DESCRIPTION_LENGTH,
        });
    }
    if credential_types.is_empty() {
        return Err(ContractError::EmptyCredentialTypes);
    }

    // Check for duplicate
    if SCHEMAS.has(deps.storage, &schema_id) {
        return Err(ContractError::SchemaAlreadyExists {
            schema_id: schema_id.clone(),
        });
    }

    let verifier_addr = deps.api.addr_validate(&verifier_contract)?;

    let schema = CredentialSchema {
        schema_id: schema_id.clone(),
        name: name.clone(),
        description,
        verifier_contract: verifier_addr,
        credential_types,
        created_at: env.block.time.seconds(),
        active: true,
    };

    SCHEMAS.save(deps.storage, &schema_id, &schema)?;

    Ok(Response::new()
        .add_attribute("action", "register_schema")
        .add_attribute("schema_id", &schema_id)
        .add_attribute("name", &name))
}

fn execute_record_proof(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    schema_id: String,
    subject: String,
    proof_hash: String,
    issuer: String,
    expires_at: Option<u64>,
) -> Result<Response, ContractError> {
    // Validate inputs
    if proof_hash.is_empty() {
        return Err(ContractError::EmptyProofHash);
    }

    // Validate schema exists
    let schema = SCHEMAS
        .may_load(deps.storage, &schema_id)?
        .ok_or(ContractError::SchemaNotFound {
            schema_id: schema_id.clone(),
        })?;

    // Only the schema's verifier contract or admin can record proofs
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != schema.verifier_contract && info.sender != admin {
        return Err(ContractError::UnauthorizedRecorder);
    }

    let subject_addr = deps.api.addr_validate(&subject)?;
    let issuer_addr = deps.api.addr_validate(&issuer)?;

    // Increment proof counter
    let id = PROOF_COUNT.load(deps.storage)? + 1;
    PROOF_COUNT.save(deps.storage, &id)?;

    let record = ProofRecord {
        id,
        schema_id: schema_id.clone(),
        subject: subject_addr.clone(),
        proof_hash: proof_hash.clone(),
        issuer: issuer_addr,
        verified_at: env.block.time.seconds(),
        expires_at,
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
    };

    PROOFS.save(deps.storage, id, &record)?;
    PROOFS_BY_SUBJECT.save(deps.storage, (&subject_addr, id), &())?;
    LATEST_PROOF.save(deps.storage, (&schema_id, &subject_addr), &id)?;

    Ok(Response::new()
        .add_attribute("action", "record_proof")
        .add_attribute("proof_id", id.to_string())
        .add_attribute("schema_id", &schema_id)
        .add_attribute("subject", subject_addr.as_str())
        .add_attribute("proof_hash", &proof_hash))
}

fn execute_revoke_proof(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proof_id: u64,
    reason: String,
) -> Result<Response, ContractError> {
    if reason.is_empty() {
        return Err(ContractError::EmptyRevocationReason);
    }

    let mut record = PROOFS
        .may_load(deps.storage, proof_id)?
        .ok_or(ContractError::ProofNotFound { proof_id })?;

    if record.revoked {
        return Err(ContractError::ProofAlreadyRevoked { proof_id });
    }

    // Only admin or the original issuer can revoke
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin && info.sender != record.issuer {
        return Err(ContractError::Unauthorized);
    }

    record.revoked = true;
    record.revoked_at = Some(env.block.time.seconds());
    record.revocation_reason = Some(reason.clone());

    PROOFS.save(deps.storage, proof_id, &record)?;

    Ok(Response::new()
        .add_attribute("action", "revoke_proof")
        .add_attribute("proof_id", proof_id.to_string())
        .add_attribute("reason", &reason))
}

fn execute_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized);
    }

    let new_admin_addr = deps.api.addr_validate(&new_admin)?;
    ADMIN.save(deps.storage, &new_admin_addr)?;

    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("new_admin", new_admin_addr.as_str()))
}

// ── Query ─────────────────────────────────────────────────────────────

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsVerified { subject, schema_id } => {
            to_json_binary(&query_is_verified(deps, env, subject, schema_id)?)
        }
        QueryMsg::ProofRecord { proof_id } => {
            to_json_binary(&query_proof_record(deps, proof_id)?)
        }
        QueryMsg::ProofsBySubject {
            subject,
            start_after,
            limit,
        } => to_json_binary(&query_proofs_by_subject(deps, subject, start_after, limit)?),
        QueryMsg::Schema { schema_id } => to_json_binary(&query_schema(deps, schema_id)?),
        QueryMsg::ListSchemas { start_after, limit } => {
            to_json_binary(&query_list_schemas(deps, start_after, limit)?)
        }
        QueryMsg::Admin {} => to_json_binary(&query_admin(deps)?),
    }
}

fn query_is_verified(
    deps: Deps,
    env: Env,
    subject: String,
    schema_id: String,
) -> StdResult<IsVerifiedResponse> {
    let subject_addr = deps.api.addr_validate(&subject)?;

    let proof_id = LATEST_PROOF.may_load(deps.storage, (&schema_id, &subject_addr))?;

    match proof_id {
        Some(id) => {
            let record = PROOFS.load(deps.storage, id)?;
            let now = env.block.time.seconds();

            let expired = record.expires_at.map_or(false, |exp| now > exp);
            let verified = !record.revoked && !expired;

            Ok(IsVerifiedResponse {
                verified,
                proof_id: Some(id),
                expires_at: record.expires_at,
            })
        }
        None => Ok(IsVerifiedResponse {
            verified: false,
            proof_id: None,
            expires_at: None,
        }),
    }
}

fn query_proof_record(deps: Deps, proof_id: u64) -> StdResult<ProofRecordResponse> {
    let record = PROOFS.load(deps.storage, proof_id)?;
    Ok(ProofRecordResponse { record })
}

fn query_proofs_by_subject(
    deps: Deps,
    subject: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProofRecordsResponse> {
    let subject_addr = deps.api.addr_validate(&subject)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: Vec<ProofRecord> = PROOFS_BY_SUBJECT
        .prefix(&subject_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (proof_id, _) = item?;
            PROOFS.load(deps.storage, proof_id)
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ProofRecordsResponse { records })
}

fn query_schema(deps: Deps, schema_id: String) -> StdResult<SchemaResponse> {
    let schema = SCHEMAS.load(deps.storage, &schema_id)?;
    Ok(SchemaResponse { schema })
}

fn query_list_schemas(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<SchemasResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_deref().map(Bound::exclusive);

    let schemas: Vec<CredentialSchema> = SCHEMAS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, schema)| schema))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(SchemasResponse { schemas })
}

fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
    let admin = ADMIN.load(deps.storage)?;
    Ok(AdminResponse { admin })
}
