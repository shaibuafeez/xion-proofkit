use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, Addr, SubMsg, WasmMsg};

use proofkit_types::verifier::*;

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
    let registry = deps.api.addr_make("registry_contract");
    let issuer_reg = deps.api.addr_make("issuer_registry_contract");
    let info = message_info(&admin, &[]);
    let msg = InstantiateMsg {
        admin: None,
        credential_registry: registry.to_string(),
        issuer_registry: issuer_reg.to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    (deps, env)
}

// ── Instantiation ────────────────────────────────────────────────────

#[test]
fn proper_instantiation() {
    let (deps, _) = setup_contract();
    let admin = addr(&deps, "admin");
    let registry = addr(&deps, "registry_contract");
    let issuer_reg = addr(&deps, "issuer_registry_contract");

    let res: ConfigResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(res.config.admin, admin);
    assert_eq!(res.config.credential_registry, registry);
    assert_eq!(res.config.issuer_registry, issuer_reg);
}

#[test]
fn instantiation_with_custom_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let creator = deps.api.addr_make("creator");
    let custom_admin = deps.api.addr_make("custom_admin");
    let registry = deps.api.addr_make("registry");
    let issuer_reg = deps.api.addr_make("issuer_reg");
    let info = message_info(&creator, &[]);
    let msg = InstantiateMsg {
        admin: Some(custom_admin.to_string()),
        credential_registry: registry.to_string(),
        issuer_registry: issuer_reg.to_string(),
    };
    instantiate(deps.as_mut(), env, info, msg).unwrap();

    let res: ConfigResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(res.config.admin, custom_admin);
}

// ── ZK Credential verification ──────────────────────────────────────

#[test]
fn verify_credential_success() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");
    let registry = addr(&deps, "registry_contract");

    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyCredential {
        schema_id: "age_verification".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        proof: "base64encodedproof".to_string(),
        public_inputs: vec!["input1".to_string(), "input2".to_string()],
        expires_at: Some(9999999999),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert!(res
        .attributes
        .iter()
        .any(|a| a.key == "action" && a.value == "verify_credential"));
    assert!(res
        .attributes
        .iter()
        .any(|a| a.key == "verification_type" && a.value == "zk_proof"));
    assert!(res
        .attributes
        .iter()
        .any(|a| a.key == "verified" && a.value == "true"));

    assert_eq!(res.messages.len(), 1);
    match &res.messages[0] {
        SubMsg { msg, .. } => match msg {
            cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
                assert_eq!(contract_addr, registry.as_str());
            }
            _ => panic!("Expected WasmMsg::Execute"),
        },
    }

    let ver_res: VerificationResultResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::VerificationResult {
                verification_id: 1,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(ver_res.result.verified);
    assert_eq!(ver_res.result.schema_id, "age_verification");
    assert_eq!(ver_res.result.subject, user1);
    assert!(matches!(
        ver_res.result.verification_type,
        VerificationType::ZkProof
    ));
}

#[test]
fn verify_credential_empty_schema() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyCredential {
        schema_id: "".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        proof: "proof".to_string(),
        public_inputs: vec!["input".to_string()],
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptySchemaId);
}

#[test]
fn verify_credential_empty_proof() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyCredential {
        schema_id: "schema".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        proof: "".to_string(),
        public_inputs: vec!["input".to_string()],
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyProof);
}

#[test]
fn verify_credential_empty_public_inputs() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyCredential {
        schema_id: "schema".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        proof: "proof".to_string(),
        public_inputs: vec![],
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyPublicInputs);
}

// ── Email credential verification ───────────────────────────────────

#[test]
fn verify_email_credential_success() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");

    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyEmailCredential {
        schema_id: "employment_verification".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        email_domain: "company.com".to_string(),
        dkim_signature: "dkim_sig_data".to_string(),
        email_headers: "From: hr@company.com\nTo: user@example.com".to_string(),
        expires_at: None,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert!(res
        .attributes
        .iter()
        .any(|a| a.key == "verification_type" && a.value == "email_dkim"));
    assert!(res
        .attributes
        .iter()
        .any(|a| a.key == "email_domain" && a.value == "company.com"));
    assert_eq!(res.messages.len(), 1);

    let ver_res: VerificationResultResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::VerificationResult {
                verification_id: 1,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        ver_res.result.verification_type,
        VerificationType::EmailDkim
    ));
}

#[test]
fn verify_email_empty_domain() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyEmailCredential {
        schema_id: "schema".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        email_domain: "".to_string(),
        dkim_signature: "sig".to_string(),
        email_headers: "headers".to_string(),
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyEmailDomain);
}

#[test]
fn verify_email_empty_dkim() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyEmailCredential {
        schema_id: "schema".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        email_domain: "example.com".to_string(),
        dkim_signature: "".to_string(),
        email_headers: "headers".to_string(),
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyDkimSignature);
}

#[test]
fn verify_email_empty_headers() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");
    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::VerifyEmailCredential {
        schema_id: "schema".to_string(),
        subject: user1.to_string(),
        issuer: issuer1.to_string(),
        email_domain: "example.com".to_string(),
        dkim_signature: "sig".to_string(),
        email_headers: "".to_string(),
        expires_at: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyEmailHeaders);
}

// ── Batch verification ──────────────────────────────────────────────

#[test]
fn batch_verify_success() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let user2 = addr(&deps, "user2");
    let issuer1 = addr(&deps, "issuer1");
    let issuer2 = addr(&deps, "issuer2");

    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::BatchVerify {
        verifications: vec![
            VerificationRequest::ZkProof {
                schema_id: "age".to_string(),
                subject: user1.to_string(),
                issuer: issuer1.to_string(),
                proof: "proof1".to_string(),
                public_inputs: vec!["input1".to_string()],
                expires_at: None,
            },
            VerificationRequest::EmailProof {
                schema_id: "employment".to_string(),
                subject: user2.to_string(),
                issuer: issuer2.to_string(),
                email_domain: "company.com".to_string(),
                dkim_signature: "sig".to_string(),
                email_headers: "headers".to_string(),
                expires_at: None,
            },
        ],
    };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(res.messages.len(), 2);

    let batch_res: BatchVerifyResponse = from_json(res.data.unwrap()).unwrap();
    assert_eq!(batch_res.results.len(), 2);
    assert!(batch_res.results[0].verified);
    assert!(batch_res.results[1].verified);
    assert_eq!(batch_res.results[0].verification_id, 1);
    assert_eq!(batch_res.results[1].verification_id, 2);
}

#[test]
fn batch_verify_with_invalid_items() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user2 = addr(&deps, "user2");
    let issuer2 = addr(&deps, "issuer2");

    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::BatchVerify {
        verifications: vec![
            // This one has empty fields → should fail gracefully
            VerificationRequest::ZkProof {
                schema_id: "".to_string(),
                subject: "user1".to_string(),
                issuer: "issuer1".to_string(),
                proof: "".to_string(),
                public_inputs: vec![],
                expires_at: None,
            },
            // This one is valid
            VerificationRequest::ZkProof {
                schema_id: "age".to_string(),
                subject: user2.to_string(),
                issuer: issuer2.to_string(),
                proof: "valid_proof".to_string(),
                public_inputs: vec!["input".to_string()],
                expires_at: None,
            },
        ],
    };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    let batch_res: BatchVerifyResponse = from_json(res.data.unwrap()).unwrap();

    assert!(!batch_res.results[0].verified);
    assert!(batch_res.results[1].verified);

    assert_eq!(res.messages.len(), 1);
}

#[test]
fn batch_verify_empty() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let info = message_info(&anyone, &[]);
    let msg = ExecuteMsg::BatchVerify {
        verifications: vec![],
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::EmptyBatch);
}

#[test]
fn batch_verify_too_large() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let info = message_info(&anyone, &[]);

    let verifications: Vec<VerificationRequest> = (0..21)
        .map(|i| {
            let user = deps.api.addr_make(&format!("user_{}", i));
            let issuer = deps.api.addr_make(&format!("issuer_{}", i));
            VerificationRequest::ZkProof {
                schema_id: format!("schema_{}", i),
                subject: user.to_string(),
                issuer: issuer.to_string(),
                proof: format!("proof_{}", i),
                public_inputs: vec![format!("input_{}", i)],
                expires_at: None,
            }
        })
        .collect();

    let msg = ExecuteMsg::BatchVerify { verifications };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::BatchTooLarge { max: 20 });
}

// ── Query tests ─────────────────────────────────────────────────────

#[test]
fn query_verifications_by_subject() {
    let (mut deps, env) = setup_contract();
    let anyone = addr(&deps, "anyone");
    let user1 = addr(&deps, "user1");
    let issuer1 = addr(&deps, "issuer1");

    for i in 0..3 {
        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::VerifyCredential {
            schema_id: format!("schema_{}", i),
            subject: user1.to_string(),
            issuer: issuer1.to_string(),
            proof: format!("proof_{}", i),
            public_inputs: vec![format!("input_{}", i)],
            expires_at: None,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    let res: VerificationResultsResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::VerificationsBySubject {
                subject: user1.to_string(),
                start_after: None,
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.results.len(), 2);

    let res: VerificationResultsResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::VerificationsBySubject {
                subject: user1.to_string(),
                start_after: Some(2),
                limit: Some(2),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.results.len(), 1);
    assert_eq!(res.results[0].id, 3);
}

// ── Admin tests ─────────────────────────────────────────────────────

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

    let res: ConfigResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(res.config.admin, new_admin);
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

#[test]
fn update_registry_success() {
    let (mut deps, env) = setup_contract();
    let admin = addr(&deps, "admin");
    let new_registry = addr(&deps, "new_registry");
    let info = message_info(&admin, &[]);
    let msg = ExecuteMsg::UpdateRegistry {
        credential_registry: new_registry.to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let res: ConfigResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(res.config.credential_registry, new_registry);
}

#[test]
fn update_registry_unauthorized() {
    let (mut deps, env) = setup_contract();
    let not_admin = addr(&deps, "not_admin");
    let new_registry = addr(&deps, "new_registry");
    let info = message_info(&not_admin, &[]);
    let msg = ExecuteMsg::UpdateRegistry {
        credential_registry: new_registry.to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}
