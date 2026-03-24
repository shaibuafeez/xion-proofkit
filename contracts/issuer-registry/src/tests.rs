use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, Addr};

use proofkit_types::issuer_registry::*;

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;

type Deps = cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
>;

fn addr(deps: &Deps, label: &str) -> Addr {
    deps.api.addr_make(label)
}

fn setup_contract() -> (Deps, cosmwasm_std::Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let admin = deps.api.addr_make("admin");
    let info = message_info(&admin, &[]);
    let msg = InstantiateMsg { admin: None };
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    (deps, env)
}

fn register_issuer(deps: &mut Deps, env: &cosmwasm_std::Env, issuer_label: &str, types: Vec<&str>) {
    let admin = deps.api.addr_make("admin");
    let issuer = deps.api.addr_make(issuer_label);
    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        issuer: issuer.to_string(),
        name: format!("Issuer {}", issuer_label),
        description: "A trusted issuer".to_string(),
        credential_types: types.into_iter().map(String::from).collect(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
}

// ── Instantiation ────────────────────────────────────────────────────

#[test]
fn proper_instantiation() {
    let (deps, _) = setup_contract();
    let admin = addr(&deps, "admin");
    let res: AdminResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Admin {}).unwrap()).unwrap();
    assert_eq!(res.admin, admin);
}

#[test]
fn instantiation_with_custom_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let creator = deps.api.addr_make("creator");
    let custom_admin = deps.api.addr_make("custom_admin");
    let info = message_info(&creator, &[]);
    let msg = InstantiateMsg {
        admin: Some(custom_admin.to_string()),
    };
    instantiate(deps.as_mut(), env, info, msg).unwrap();

    let res: AdminResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Admin {}).unwrap()).unwrap();
    assert_eq!(res.admin, custom_admin);
}

// ── Register issuer ──────────────────────────────────────────────────

#[test]
fn register_issuer_success() {
    let (mut deps, env) = setup_contract();
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age", "identity"]);

    let res: IssuerResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Issuer {
                issuer: issuer1.to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(res.issuer.address, issuer1);
    assert_eq!(res.issuer.name, "Issuer issuer1");
    assert!(res.issuer.active);
    assert_eq!(res.issuer.credential_types, vec!["age", "identity"]);
}

#[test]
fn register_issuer_unauthorized() {
    let (mut deps, env) = setup_contract();
    let not_admin = addr(&deps, "not_admin");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&not_admin, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        issuer: issuer1.to_string(),
        name: "Issuer 1".to_string(),
        description: "desc".to_string(),
        credential_types: vec!["age".to_string()],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}

#[test]
fn register_issuer_duplicate() {
    let (mut deps, env) = setup_contract();
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let admin = addr(&deps, "admin");
    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        issuer: issuer1.to_string(),
        name: "Dup".to_string(),
        description: "dup".to_string(),
        credential_types: vec!["age".to_string()],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::IssuerAlreadyRegistered {
            issuer: issuer1.to_string()
        }
    );
}

#[test]
fn register_issuer_empty_name() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        issuer: issuer1.to_string(),
        name: "".to_string(),
        description: "desc".to_string(),
        credential_types: vec!["age".to_string()],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyName);
}

#[test]
fn register_issuer_empty_credential_types() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        issuer: issuer1.to_string(),
        name: "Name".to_string(),
        description: "desc".to_string(),
        credential_types: vec![],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyCredentialTypes);
}

// ── IsAuthorized ─────────────────────────────────────────────────────

#[test]
fn is_authorized_active_issuer() {
    let (mut deps, env) = setup_contract();
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age", "identity"]);

    let res: IsAuthorizedResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsAuthorized {
                issuer: issuer1.to_string(),
                credential_type: "age".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(res.authorized);
    assert_eq!(res.issuer_name, Some("Issuer issuer1".to_string()));
}

#[test]
fn is_authorized_wrong_type() {
    let (mut deps, env) = setup_contract();
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let res: IsAuthorizedResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsAuthorized {
                issuer: issuer1.to_string(),
                credential_type: "employment".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res.authorized);
}

#[test]
fn is_authorized_nonexistent_issuer() {
    let (deps, _) = setup_contract();
    let nonexistent = addr(&deps, "nonexistent");
    let res: IsAuthorizedResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsAuthorized {
                issuer: nonexistent.to_string(),
                credential_type: "age".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res.authorized);
}

// ── Revoke issuer ────────────────────────────────────────────────────

#[test]
fn revoke_issuer_success() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RevokeIssuer {
        issuer: issuer1.to_string(),
        reason: "Compromised keys".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let res: IsAuthorizedResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsAuthorized {
                issuer: issuer1.to_string(),
                credential_type: "age".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res.authorized);

    let res: IssuerResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Issuer {
                issuer: issuer1.to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res.issuer.active);
    assert!(res.issuer.revoked_at.is_some());
    assert_eq!(
        res.issuer.revocation_reason,
        Some("Compromised keys".to_string())
    );
}

#[test]
fn revoke_issuer_unauthorized() {
    let (mut deps, env) = setup_contract();
    let not_admin = addr(&deps, "not_admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&not_admin, &[]);
    let msg = ExecuteMsg::RevokeIssuer {
        issuer: issuer1.to_string(),
        reason: "reason".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}

#[test]
fn revoke_issuer_already_revoked() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RevokeIssuer {
        issuer: issuer1.to_string(),
        reason: "First".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RevokeIssuer {
        issuer: issuer1.to_string(),
        reason: "Second".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::IssuerAlreadyRevoked {
            issuer: issuer1.to_string()
        }
    );
}

#[test]
fn revoke_issuer_not_found() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let nonexistent = addr(&deps, "nonexistent");
    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RevokeIssuer {
        issuer: nonexistent.to_string(),
        reason: "reason".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::IssuerNotFound {
            issuer: nonexistent.to_string()
        }
    );
}

#[test]
fn revoke_issuer_empty_reason() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::RevokeIssuer {
        issuer: issuer1.to_string(),
        reason: "".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyRevocationReason);
}

// ── Update issuer ────────────────────────────────────────────────────

#[test]
fn update_issuer_name() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::UpdateIssuer {
        issuer: issuer1.to_string(),
        name: Some("New Name".to_string()),
        description: None,
        credential_types: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let res: IssuerResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Issuer {
                issuer: issuer1.to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.issuer.name, "New Name");
}

#[test]
fn update_issuer_credential_types() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::UpdateIssuer {
        issuer: issuer1.to_string(),
        name: None,
        description: None,
        credential_types: Some(vec!["age".to_string(), "employment".to_string()]),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let res: IsAuthorizedResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsAuthorized {
                issuer: issuer1.to_string(),
                credential_type: "employment".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(res.authorized);
}

#[test]
fn update_issuer_no_fields() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::UpdateIssuer {
        issuer: issuer1.to_string(),
        name: None,
        description: None,
        credential_types: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::NoFieldsToUpdate);
}

#[test]
fn update_issuer_unauthorized() {
    let (mut deps, env) = setup_contract();
    let not_admin = addr(&deps, "not_admin");
    let issuer1 = addr(&deps, "issuer1");
    register_issuer(&mut deps, &env, "issuer1", vec!["age"]);

    let info = message_info(&not_admin, &[]);
    let msg = ExecuteMsg::UpdateIssuer {
        issuer: issuer1.to_string(),
        name: Some("New".to_string()),
        description: None,
        credential_types: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}

// ── List / pagination ────────────────────────────────────────────────

#[test]
fn list_issuers_pagination() {
    let (mut deps, env) = setup_contract();
    register_issuer(&mut deps, &env, "alice", vec!["age"]);
    register_issuer(&mut deps, &env, "bob", vec!["employment"]);
    register_issuer(&mut deps, &env, "charlie", vec!["age", "employment"]);

    let res: IssuersResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListIssuers {
                start_after: None,
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.issuers.len(), 2);

    let last = &res.issuers[1].address;
    let res2: IssuersResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListIssuers {
                start_after: Some(last.to_string()),
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res2.issuers.len(), 1);
}

#[test]
fn issuers_by_type() {
    let (mut deps, env) = setup_contract();
    register_issuer(&mut deps, &env, "issuer1", vec!["age", "identity"]);
    register_issuer(&mut deps, &env, "issuer2", vec!["employment"]);
    register_issuer(&mut deps, &env, "issuer3", vec!["age"]);

    let res: IssuersResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IssuersByType {
                credential_type: "age".to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.issuers.len(), 2);

    let res: IssuersResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IssuersByType {
                credential_type: "employment".to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.issuers.len(), 1);
}

// ── Update admin ─────────────────────────────────────────────────────

#[test]
fn update_admin_success() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let new_admin = addr(&deps, "new_admin");
    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::UpdateAdmin {
        new_admin: new_admin.to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let res: AdminResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Admin {}).unwrap()).unwrap();
    assert_eq!(res.admin, new_admin);
}

#[test]
fn update_admin_unauthorized() {
    let (mut deps, env) = setup_contract();
    let not_admin = addr(&deps, "not_admin");
    let new_admin = addr(&deps, "new_admin");
    let info = message_info(&not_admin, &[]);
    let msg = ExecuteMsg::UpdateAdmin {
        new_admin: new_admin.to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}
