use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, Addr, Timestamp};

use proofkit_types::credential_registry::*;

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;

type Deps = cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
>;

struct TestAddrs {
    admin: Addr,
    verifier_contract: Addr,
    user1: Addr,
    issuer1: Addr,
    not_admin: Addr,
}

fn addrs(deps: &Deps) -> TestAddrs {
    TestAddrs {
        admin: deps.api.addr_make("admin"),
        verifier_contract: deps.api.addr_make("verifier_contract"),
        user1: deps.api.addr_make("user1"),
        issuer1: deps.api.addr_make("issuer1"),
        not_admin: deps.api.addr_make("not_admin"),
    }
}

fn setup_contract() -> (Deps, cosmwasm_std::Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let a = addrs(&deps);
    let info = message_info(&a.admin, &[]);
    let msg = InstantiateMsg { admin: None };
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    (deps, env)
}

fn register_schema(deps: &mut Deps, env: &cosmwasm_std::Env) {
    let a = addrs(deps);
    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RegisterSchema {
        schema_id: "age_verification".to_string(),
        name: "Age Verification".to_string(),
        description: "Proves the subject is over a certain age".to_string(),
        verifier_contract: a.verifier_contract.to_string(),
        credential_types: vec!["age".to_string()],
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
}

// ── Instantiation tests ──────────────────────────────────────────────

#[test]
fn proper_instantiation() {
    let (deps, _env) = setup_contract();
    let a = addrs(&deps);
    let res: AdminResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Admin {}).unwrap()).unwrap();
    assert_eq!(res.admin, a.admin);
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

// ── Schema registration tests ────────────────────────────────────────

#[test]
fn register_schema_success() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);

    let res: SchemaResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Schema {
                schema_id: "age_verification".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(res.schema.schema_id, "age_verification");
    assert_eq!(res.schema.name, "Age Verification");
    assert!(res.schema.active);
    assert_eq!(res.schema.credential_types, vec!["age".to_string()]);
}

#[test]
fn register_schema_unauthorized() {
    let (mut deps, env) = setup_contract();
    let a = addrs(&deps);
    let info = message_info(&a.not_admin, &[]);
    let msg = ExecuteMsg::RegisterSchema {
        schema_id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        verifier_contract: a.verifier_contract.to_string(),
        credential_types: vec!["test".to_string()],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}

#[test]
fn register_schema_duplicate() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);

    let a = addrs(&deps);
    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RegisterSchema {
        schema_id: "age_verification".to_string(),
        name: "Duplicate".to_string(),
        description: "Dup".to_string(),
        verifier_contract: a.verifier_contract.to_string(),
        credential_types: vec!["age".to_string()],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::SchemaAlreadyExists {
            schema_id: "age_verification".to_string()
        }
    );
}

#[test]
fn register_schema_empty_id() {
    let (mut deps, env) = setup_contract();
    let a = addrs(&deps);
    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RegisterSchema {
        schema_id: "".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        verifier_contract: a.verifier_contract.to_string(),
        credential_types: vec!["test".to_string()],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptySchemaId);
}

#[test]
fn register_schema_empty_credential_types() {
    let (mut deps, env) = setup_contract();
    let a = addrs(&deps);
    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RegisterSchema {
        schema_id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        verifier_contract: a.verifier_contract.to_string(),
        credential_types: vec![],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyCredentialTypes);
}

// ── Proof recording tests ────────────────────────────────────────────

#[test]
fn record_proof_success() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 5);

    let res: IsVerifiedResponse = from_json(
        query(
            deps.as_ref(),
            env,
            QueryMsg::IsVerified {
                subject: a.user1.to_string(),
                schema_id: "age_verification".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(res.verified);
    assert_eq!(res.proof_id, Some(1));
}

#[test]
fn record_proof_unauthorized() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let random = deps.api.addr_make("random_user");
    let info = message_info(&random, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::UnauthorizedRecorder);
}

#[test]
fn record_proof_schema_not_found() {
    let (mut deps, env) = setup_contract();
    let a = addrs(&deps);
    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "nonexistent".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::SchemaNotFound {
            schema_id: "nonexistent".to_string()
        }
    );
}

#[test]
fn record_proof_empty_hash() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);
    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyProofHash);
}

// ── Proof revocation tests ───────────────────────────────────────────

#[test]
fn revoke_proof_by_admin() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RevokeProof {
        proof_id: 1,
        reason: "Fraudulent credential".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res: IsVerifiedResponse = from_json(
        query(
            deps.as_ref(),
            env,
            QueryMsg::IsVerified {
                subject: a.user1.to_string(),
                schema_id: "age_verification".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res.verified);
}

#[test]
fn revoke_proof_by_issuer() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = message_info(&a.issuer1, &[]);
    let msg = ExecuteMsg::RevokeProof {
        proof_id: 1,
        reason: "Issuer-initiated revocation".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();
}

#[test]
fn revoke_proof_unauthorized() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let random = deps.api.addr_make("random_user");
    let info = message_info(&random, &[]);
    let msg = ExecuteMsg::RevokeProof {
        proof_id: 1,
        reason: "Malicious revocation".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}

#[test]
fn revoke_proof_already_revoked() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RevokeProof {
        proof_id: 1,
        reason: "First revocation".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RevokeProof {
        proof_id: 1,
        reason: "Second revocation".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::ProofAlreadyRevoked { proof_id: 1 });
}

#[test]
fn revoke_proof_empty_reason() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = message_info(&a.admin, &[]);
    let msg = ExecuteMsg::RevokeProof {
        proof_id: 1,
        reason: "".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyRevocationReason);
}

// ── Expiration tests ─────────────────────────────────────────────────

#[test]
fn expired_proof_not_verified() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    let info = message_info(&a.verifier_contract, &[]);
    let msg = ExecuteMsg::RecordProof {
        schema_id: "age_verification".to_string(),
        subject: a.user1.to_string(),
        proof_hash: "abc123hash".to_string(),
        issuer: a.issuer1.to_string(),
        expires_at: Some(1000),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let mut future_env = mock_env();
    future_env.block.time = Timestamp::from_seconds(2000);

    let res: IsVerifiedResponse = from_json(
        query(
            deps.as_ref(),
            future_env,
            QueryMsg::IsVerified {
                subject: a.user1.to_string(),
                schema_id: "age_verification".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res.verified);
}

// ── Query tests ──────────────────────────────────────────────────────

#[test]
fn query_not_verified_no_proof() {
    let (deps, env) = setup_contract();
    let a = addrs(&deps);
    let res: IsVerifiedResponse = from_json(
        query(
            deps.as_ref(),
            env,
            QueryMsg::IsVerified {
                subject: a.user1.to_string(),
                schema_id: "age_verification".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res.verified);
    assert_eq!(res.proof_id, None);
}

#[test]
fn query_proofs_by_subject_pagination() {
    let (mut deps, env) = setup_contract();
    register_schema(&mut deps, &env);
    let a = addrs(&deps);

    for i in 0..3 {
        let info = message_info(&a.verifier_contract, &[]);
        let msg = ExecuteMsg::RecordProof {
            schema_id: "age_verification".to_string(),
            subject: a.user1.to_string(),
            proof_hash: format!("hash_{}", i),
            issuer: a.issuer1.to_string(),
            expires_at: None,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    let res: ProofRecordsResponse = from_json(
        query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::ProofsBySubject {
                subject: a.user1.to_string(),
                start_after: None,
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.records.len(), 2);
    assert_eq!(res.records[0].id, 1);
    assert_eq!(res.records[1].id, 2);

    let res: ProofRecordsResponse = from_json(
        query(
            deps.as_ref(),
            env,
            QueryMsg::ProofsBySubject {
                subject: a.user1.to_string(),
                start_after: Some(2),
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.records.len(), 1);
    assert_eq!(res.records[0].id, 3);
}

#[test]
fn list_schemas_pagination() {
    let (mut deps, env) = setup_contract();
    let a = addrs(&deps);
    let verifier = a.verifier_contract.to_string();

    let info = message_info(&a.admin, &[]);
    for id in ["alpha", "beta", "gamma"] {
        let msg = ExecuteMsg::RegisterSchema {
            schema_id: id.to_string(),
            name: id.to_string(),
            description: "desc".to_string(),
            verifier_contract: verifier.clone(),
            credential_types: vec!["type".to_string()],
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    }

    let res: SchemasResponse = from_json(
        query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::ListSchemas {
                start_after: None,
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.schemas.len(), 2);
    assert_eq!(res.schemas[0].schema_id, "alpha");
    assert_eq!(res.schemas[1].schema_id, "beta");

    let res: SchemasResponse = from_json(
        query(
            deps.as_ref(),
            env,
            QueryMsg::ListSchemas {
                start_after: Some("beta".to_string()),
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.schemas.len(), 1);
    assert_eq!(res.schemas[0].schema_id, "gamma");
}

// ── Update admin tests ───────────────────────────────────────────────

#[test]
fn update_admin_success() {
    let (mut deps, env) = setup_contract();
    let a = addrs(&deps);
    let new_admin = deps.api.addr_make("new_admin");
    let info = message_info(&a.admin, &[]);
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
    let a = addrs(&deps);
    let new_admin = deps.api.addr_make("new_admin");
    let info = message_info(&a.not_admin, &[]);
    let msg = ExecuteMsg::UpdateAdmin {
        new_admin: new_admin.to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}
