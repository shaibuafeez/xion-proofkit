use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use proofkit_types::issuer_registry::*;
use proofkit_types::{DEFAULT_LIMIT, MAX_DESCRIPTION_LENGTH, MAX_LIMIT};

use crate::error::ContractError;
use crate::state::*;

const CONTRACT_NAME: &str = "crates.io:proofkit-issuer-registry";
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
        ExecuteMsg::RegisterIssuer {
            issuer,
            name,
            description,
            credential_types,
        } => execute_register_issuer(deps, env, info, issuer, name, description, credential_types),
        ExecuteMsg::RevokeIssuer { issuer, reason } => {
            execute_revoke_issuer(deps, env, info, issuer, reason)
        }
        ExecuteMsg::UpdateIssuer {
            issuer,
            name,
            description,
            credential_types,
        } => execute_update_issuer(deps, info, issuer, name, description, credential_types),
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, info, new_admin),
    }
}

fn execute_register_issuer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    issuer: String,
    name: String,
    description: String,
    credential_types: Vec<String>,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized);
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

    let issuer_addr = deps.api.addr_validate(&issuer)?;

    if ISSUERS.has(deps.storage, &issuer_addr) {
        return Err(ContractError::IssuerAlreadyRegistered {
            issuer: issuer.clone(),
        });
    }

    let record = IssuerRecord {
        address: issuer_addr.clone(),
        name: name.clone(),
        description,
        credential_types: credential_types.clone(),
        registered_at: env.block.time.seconds(),
        active: true,
        revoked_at: None,
        revocation_reason: None,
    };

    ISSUERS.save(deps.storage, &issuer_addr, &record)?;

    // Build secondary indexes
    for ct in &credential_types {
        ISSUERS_BY_TYPE.save(deps.storage, (ct.as_str(), &issuer_addr), &())?;
    }

    Ok(Response::new()
        .add_attribute("action", "register_issuer")
        .add_attribute("issuer", issuer_addr.as_str())
        .add_attribute("name", &name)
        .add_attribute("credential_types", credential_types.join(",")))
}

fn execute_revoke_issuer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    issuer: String,
    reason: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized);
    }

    if reason.is_empty() {
        return Err(ContractError::EmptyRevocationReason);
    }

    let issuer_addr = deps.api.addr_validate(&issuer)?;

    let mut record = ISSUERS
        .may_load(deps.storage, &issuer_addr)?
        .ok_or(ContractError::IssuerNotFound {
            issuer: issuer.clone(),
        })?;

    if !record.active {
        return Err(ContractError::IssuerAlreadyRevoked {
            issuer: issuer.clone(),
        });
    }

    // Remove secondary indexes
    for ct in &record.credential_types {
        ISSUERS_BY_TYPE.remove(deps.storage, (ct.as_str(), &issuer_addr));
    }

    record.active = false;
    record.revoked_at = Some(env.block.time.seconds());
    record.revocation_reason = Some(reason.clone());

    ISSUERS.save(deps.storage, &issuer_addr, &record)?;

    Ok(Response::new()
        .add_attribute("action", "revoke_issuer")
        .add_attribute("issuer", issuer_addr.as_str())
        .add_attribute("reason", &reason))
}

fn execute_update_issuer(
    deps: DepsMut,
    info: MessageInfo,
    issuer: String,
    name: Option<String>,
    description: Option<String>,
    credential_types: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized);
    }

    if name.is_none() && description.is_none() && credential_types.is_none() {
        return Err(ContractError::NoFieldsToUpdate);
    }

    let issuer_addr = deps.api.addr_validate(&issuer)?;

    let mut record = ISSUERS
        .may_load(deps.storage, &issuer_addr)?
        .ok_or(ContractError::IssuerNotFound {
            issuer: issuer.clone(),
        })?;

    if let Some(new_name) = name {
        if new_name.is_empty() {
            return Err(ContractError::EmptyName);
        }
        record.name = new_name;
    }

    if let Some(new_desc) = description {
        if new_desc.len() > MAX_DESCRIPTION_LENGTH {
            return Err(ContractError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH,
            });
        }
        record.description = new_desc;
    }

    if let Some(new_types) = credential_types {
        if new_types.is_empty() {
            return Err(ContractError::EmptyCredentialTypes);
        }

        // Remove old indexes
        for ct in &record.credential_types {
            ISSUERS_BY_TYPE.remove(deps.storage, (ct.as_str(), &issuer_addr));
        }

        // Add new indexes (only if issuer is active)
        if record.active {
            for ct in &new_types {
                ISSUERS_BY_TYPE.save(deps.storage, (ct.as_str(), &issuer_addr), &())?;
            }
        }

        record.credential_types = new_types;
    }

    ISSUERS.save(deps.storage, &issuer_addr, &record)?;

    Ok(Response::new()
        .add_attribute("action", "update_issuer")
        .add_attribute("issuer", issuer_addr.as_str()))
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsAuthorized {
            issuer,
            credential_type,
        } => to_json_binary(&query_is_authorized(deps, issuer, credential_type)?),
        QueryMsg::Issuer { issuer } => to_json_binary(&query_issuer(deps, issuer)?),
        QueryMsg::ListIssuers { start_after, limit } => {
            to_json_binary(&query_list_issuers(deps, start_after, limit)?)
        }
        QueryMsg::IssuersByType {
            credential_type,
            start_after,
            limit,
        } => to_json_binary(&query_issuers_by_type(
            deps,
            credential_type,
            start_after,
            limit,
        )?),
        QueryMsg::Admin {} => to_json_binary(&query_admin(deps)?),
    }
}

fn query_is_authorized(
    deps: Deps,
    issuer: String,
    credential_type: String,
) -> StdResult<IsAuthorizedResponse> {
    let issuer_addr = deps.api.addr_validate(&issuer)?;

    match ISSUERS.may_load(deps.storage, &issuer_addr)? {
        Some(record) if record.active && record.credential_types.contains(&credential_type) => {
            Ok(IsAuthorizedResponse {
                authorized: true,
                issuer_name: Some(record.name),
            })
        }
        _ => Ok(IsAuthorizedResponse {
            authorized: false,
            issuer_name: None,
        }),
    }
}

fn query_issuer(deps: Deps, issuer: String) -> StdResult<IssuerResponse> {
    let issuer_addr = deps.api.addr_validate(&issuer)?;
    let record = ISSUERS.load(deps.storage, &issuer_addr)?;
    Ok(IssuerResponse { issuer: record })
}

fn query_list_issuers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<IssuersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let start = start_after
        .as_ref()
        .map(|s| deps.api.addr_validate(s))
        .transpose()?;
    let start_bound = start.as_ref().map(Bound::exclusive);

    let issuers: Vec<IssuerRecord> = ISSUERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, record)| record))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(IssuersResponse { issuers })
}

fn query_issuers_by_type(
    deps: Deps,
    credential_type: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<IssuersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let start = start_after
        .as_ref()
        .map(|s| deps.api.addr_validate(s))
        .transpose()?;
    let start_bound = start.as_ref().map(Bound::exclusive);

    let issuers: Vec<IssuerRecord> = ISSUERS_BY_TYPE
        .prefix(&credential_type)
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (issuer_addr, _) = item?;
            ISSUERS.load(deps.storage, &issuer_addr)
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(IssuersResponse { issuers })
}

fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
    let admin = ADMIN.load(deps.storage)?;
    Ok(AdminResponse { admin })
}
