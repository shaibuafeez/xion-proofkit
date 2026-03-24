use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use proofkit_types::credential_registry::{
    ExecuteMsg as RegistryExecuteMsg, InstantiateMsg as RegistryInstantiateMsg,
    IsVerifiedResponse, QueryMsg as RegistryQueryMsg, SchemaResponse,
};
use proofkit_types::issuer_registry::{
    ExecuteMsg as IssuerExecuteMsg, InstantiateMsg as IssuerInstantiateMsg,
    IsAuthorizedResponse, QueryMsg as IssuerQueryMsg,
};
use proofkit_types::verifier::{
    ConfigResponse, ExecuteMsg as VerifierExecuteMsg, InstantiateMsg as VerifierInstantiateMsg,
    QueryMsg as VerifierQueryMsg, VerificationResultResponse, VerificationType,
    VerificationRequest,
};

// ── Contract wrappers ────────────────────────────────────────────────

fn credential_registry_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        credential_registry::contract::execute,
        credential_registry::contract::instantiate,
        credential_registry::contract::query,
    );
    Box::new(contract)
}

fn verifier_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        verifier::contract::execute,
        verifier::contract::instantiate,
        verifier::contract::query,
    );
    Box::new(contract)
}

fn issuer_registry_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        issuer_registry::contract::execute,
        issuer_registry::contract::instantiate,
        issuer_registry::contract::query,
    );
    Box::new(contract)
}

// ── Test setup ───────────────────────────────────────────────────────

struct TestSuite {
    app: App,
    admin: Addr,
    registry_addr: Addr,
    verifier_addr: Addr,
    issuer_reg_addr: Addr,
}

fn setup() -> TestSuite {
    let mut app = App::default();
    let admin = app.api().addr_make("admin");

    // Store contract codes
    let registry_code = app.store_code(credential_registry_contract());
    let verifier_code = app.store_code(verifier_contract());
    let issuer_code = app.store_code(issuer_registry_contract());

    // Instantiate issuer registry first
    let issuer_reg_addr = app
        .instantiate_contract(
            issuer_code,
            admin.clone(),
            &IssuerInstantiateMsg { admin: None },
            &[],
            "issuer-registry",
            None,
        )
        .unwrap();

    // Instantiate credential registry
    let registry_addr = app
        .instantiate_contract(
            registry_code,
            admin.clone(),
            &RegistryInstantiateMsg { admin: None },
            &[],
            "credential-registry",
            None,
        )
        .unwrap();

    // Instantiate verifier with references to both registries
    let verifier_addr = app
        .instantiate_contract(
            verifier_code,
            admin.clone(),
            &VerifierInstantiateMsg {
                admin: None,
                credential_registry: registry_addr.to_string(),
                issuer_registry: issuer_reg_addr.to_string(),
            },
            &[],
            "verifier",
            None,
        )
        .unwrap();

    TestSuite {
        app,
        admin,
        registry_addr,
        verifier_addr,
        issuer_reg_addr,
    }
}

// ── Full verification flow tests ─────────────────────────────────────

#[test]
fn full_zk_verification_flow() {
    let mut suite = setup();
    let user = suite.app.api().addr_make("user1");
    let issuer = suite.app.api().addr_make("issuer1");

    // Step 1: Register a schema in the credential registry.
    //         The verifier contract is set as the authorized verifier.
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.registry_addr.clone(),
            &RegistryExecuteMsg::RegisterSchema {
                schema_id: "age_verification".to_string(),
                name: "Age Verification".to_string(),
                description: "Proves user is over 18".to_string(),
                verifier_contract: suite.verifier_addr.to_string(),
                credential_types: vec!["age".to_string()],
            },
            &[],
        )
        .unwrap();

    // Step 2: Register an issuer in the issuer registry.
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.issuer_reg_addr.clone(),
            &IssuerExecuteMsg::RegisterIssuer {
                issuer: issuer.to_string(),
                name: "Gov ID Issuer".to_string(),
                description: "Government identity provider".to_string(),
                credential_types: vec!["age".to_string(), "identity".to_string()],
            },
            &[],
        )
        .unwrap();

    // Step 3: Verify a credential through the verifier contract.
    //         This triggers a RecordProof submessage to the credential registry.
    suite
        .app
        .execute_contract(
            user.clone(),
            suite.verifier_addr.clone(),
            &VerifierExecuteMsg::VerifyCredential {
                schema_id: "age_verification".to_string(),
                subject: user.to_string(),
                issuer: issuer.to_string(),
                proof: "zk_proof_data_base64".to_string(),
                public_inputs: vec!["age_over_18=true".to_string()],
                expires_at: None,
            },
            &[],
        )
        .unwrap();

    // Step 4: Verify the proof was recorded in the credential registry.
    let is_verified: IsVerifiedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.registry_addr,
            &RegistryQueryMsg::IsVerified {
                subject: user.to_string(),
                schema_id: "age_verification".to_string(),
            },
        )
        .unwrap();

    assert!(is_verified.verified);
    assert_eq!(is_verified.proof_id, Some(1));

    // Step 5: Verify the verification record exists in the verifier.
    let ver_result: VerificationResultResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.verifier_addr,
            &VerifierQueryMsg::VerificationResult {
                verification_id: 1,
            },
        )
        .unwrap();

    assert!(ver_result.result.verified);
    assert_eq!(ver_result.result.subject, user);
    assert!(matches!(
        ver_result.result.verification_type,
        VerificationType::ZkProof
    ));
}

#[test]
fn full_email_verification_flow() {
    let mut suite = setup();
    let user = suite.app.api().addr_make("employee1");
    let issuer = suite.app.api().addr_make("corp_issuer");

    // Register schema
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.registry_addr.clone(),
            &RegistryExecuteMsg::RegisterSchema {
                schema_id: "employment_verification".to_string(),
                name: "Employment Verification".to_string(),
                description: "Proves employment at a company".to_string(),
                verifier_contract: suite.verifier_addr.to_string(),
                credential_types: vec!["employment".to_string()],
            },
            &[],
        )
        .unwrap();

    // Verify email credential
    suite
        .app
        .execute_contract(
            user.clone(),
            suite.verifier_addr.clone(),
            &VerifierExecuteMsg::VerifyEmailCredential {
                schema_id: "employment_verification".to_string(),
                subject: user.to_string(),
                issuer: issuer.to_string(),
                email_domain: "megacorp.com".to_string(),
                dkim_signature: "dkim_sig_base64".to_string(),
                email_headers: "From: hr@megacorp.com\nSubject: Employment Confirmation".to_string(),
                expires_at: Some(9999999999),
            },
            &[],
        )
        .unwrap();

    // Check it's recorded
    let is_verified: IsVerifiedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.registry_addr,
            &RegistryQueryMsg::IsVerified {
                subject: user.to_string(),
                schema_id: "employment_verification".to_string(),
            },
        )
        .unwrap();

    assert!(is_verified.verified);
    assert_eq!(is_verified.expires_at, Some(9999999999));
}

#[test]
fn batch_verify_records_multiple_proofs() {
    let mut suite = setup();
    let user1 = suite.app.api().addr_make("user1");
    let user2 = suite.app.api().addr_make("user2");
    let issuer = suite.app.api().addr_make("issuer1");

    // Register two schemas
    for (id, name, ctype) in [
        ("age_check", "Age Check", "age"),
        ("id_check", "ID Check", "identity"),
    ] {
        suite
            .app
            .execute_contract(
                suite.admin.clone(),
                suite.registry_addr.clone(),
                &RegistryExecuteMsg::RegisterSchema {
                    schema_id: id.to_string(),
                    name: name.to_string(),
                    description: "desc".to_string(),
                    verifier_contract: suite.verifier_addr.to_string(),
                    credential_types: vec![ctype.to_string()],
                },
                &[],
            )
            .unwrap();
    }

    // Batch verify two different users/schemas
    suite
        .app
        .execute_contract(
            user1.clone(),
            suite.verifier_addr.clone(),
            &VerifierExecuteMsg::BatchVerify {
                verifications: vec![
                    VerificationRequest::ZkProof {
                        schema_id: "age_check".to_string(),
                        subject: user1.to_string(),
                        issuer: issuer.to_string(),
                        proof: "proof_1".to_string(),
                        public_inputs: vec!["input_1".to_string()],
                        expires_at: None,
                    },
                    VerificationRequest::ZkProof {
                        schema_id: "id_check".to_string(),
                        subject: user2.to_string(),
                        issuer: issuer.to_string(),
                        proof: "proof_2".to_string(),
                        public_inputs: vec!["input_2".to_string()],
                        expires_at: None,
                    },
                ],
            },
            &[],
        )
        .unwrap();

    // Both should be verified in the registry
    let v1: IsVerifiedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.registry_addr,
            &RegistryQueryMsg::IsVerified {
                subject: user1.to_string(),
                schema_id: "age_check".to_string(),
            },
        )
        .unwrap();
    assert!(v1.verified);

    let v2: IsVerifiedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.registry_addr,
            &RegistryQueryMsg::IsVerified {
                subject: user2.to_string(),
                schema_id: "id_check".to_string(),
            },
        )
        .unwrap();
    assert!(v2.verified);
}

#[test]
fn revoke_proof_after_verification() {
    let mut suite = setup();
    let user = suite.app.api().addr_make("user1");
    let issuer = suite.app.api().addr_make("issuer1");

    // Setup: register schema and verify
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.registry_addr.clone(),
            &RegistryExecuteMsg::RegisterSchema {
                schema_id: "age".to_string(),
                name: "Age".to_string(),
                description: "desc".to_string(),
                verifier_contract: suite.verifier_addr.to_string(),
                credential_types: vec!["age".to_string()],
            },
            &[],
        )
        .unwrap();

    suite
        .app
        .execute_contract(
            user.clone(),
            suite.verifier_addr.clone(),
            &VerifierExecuteMsg::VerifyCredential {
                schema_id: "age".to_string(),
                subject: user.to_string(),
                issuer: issuer.to_string(),
                proof: "proof".to_string(),
                public_inputs: vec!["input".to_string()],
                expires_at: None,
            },
            &[],
        )
        .unwrap();

    // Verified at this point
    let v: IsVerifiedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.registry_addr,
            &RegistryQueryMsg::IsVerified {
                subject: user.to_string(),
                schema_id: "age".to_string(),
            },
        )
        .unwrap();
    assert!(v.verified);

    // Revoke the proof
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.registry_addr.clone(),
            &RegistryExecuteMsg::RevokeProof {
                proof_id: 1,
                reason: "Fraudulent".to_string(),
            },
            &[],
        )
        .unwrap();

    // No longer verified
    let v: IsVerifiedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.registry_addr,
            &RegistryQueryMsg::IsVerified {
                subject: user.to_string(),
                schema_id: "age".to_string(),
            },
        )
        .unwrap();
    assert!(!v.verified);
}

#[test]
fn issuer_authorization_query_cross_contract() {
    let mut suite = setup();
    let issuer = suite.app.api().addr_make("trusted_issuer");

    // Register issuer for "age" credentials
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.issuer_reg_addr.clone(),
            &IssuerExecuteMsg::RegisterIssuer {
                issuer: issuer.to_string(),
                name: "Trusted Age Verifier".to_string(),
                description: "Official age verification provider".to_string(),
                credential_types: vec!["age".to_string()],
            },
            &[],
        )
        .unwrap();

    // Query from the verifier's perspective (cross-contract query)
    let authorized: IsAuthorizedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.issuer_reg_addr,
            &IssuerQueryMsg::IsAuthorized {
                issuer: issuer.to_string(),
                credential_type: "age".to_string(),
            },
        )
        .unwrap();
    assert!(authorized.authorized);

    // Not authorized for employment
    let not_authorized: IsAuthorizedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.issuer_reg_addr,
            &IssuerQueryMsg::IsAuthorized {
                issuer: issuer.to_string(),
                credential_type: "employment".to_string(),
            },
        )
        .unwrap();
    assert!(!not_authorized.authorized);

    // Revoke issuer
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.issuer_reg_addr.clone(),
            &IssuerExecuteMsg::RevokeIssuer {
                issuer: issuer.to_string(),
                reason: "Keys compromised".to_string(),
            },
            &[],
        )
        .unwrap();

    // No longer authorized even for age
    let revoked: IsAuthorizedResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.issuer_reg_addr,
            &IssuerQueryMsg::IsAuthorized {
                issuer: issuer.to_string(),
                credential_type: "age".to_string(),
            },
        )
        .unwrap();
    assert!(!revoked.authorized);
}

#[test]
fn verifier_config_points_to_correct_contracts() {
    let suite = setup();

    let config: ConfigResponse = suite
        .app
        .wrap()
        .query_wasm_smart(&suite.verifier_addr, &VerifierQueryMsg::Config {})
        .unwrap();

    assert_eq!(config.config.admin, suite.admin);
    assert_eq!(config.config.credential_registry, suite.registry_addr);
    assert_eq!(config.config.issuer_registry, suite.issuer_reg_addr);
}

#[test]
fn unauthorized_recorder_rejected_by_registry() {
    let mut suite = setup();
    let random = suite.app.api().addr_make("random_user");
    let issuer = suite.app.api().addr_make("issuer1");

    // Register a schema with the verifier as authorized recorder
    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.registry_addr.clone(),
            &RegistryExecuteMsg::RegisterSchema {
                schema_id: "test".to_string(),
                name: "Test".to_string(),
                description: "desc".to_string(),
                verifier_contract: suite.verifier_addr.to_string(),
                credential_types: vec!["test".to_string()],
            },
            &[],
        )
        .unwrap();

    let someone = suite.app.api().addr_make("someone");

    // Try to record a proof directly (not through the verifier) — should fail
    let err = suite
        .app
        .execute_contract(
            random,
            suite.registry_addr.clone(),
            &RegistryExecuteMsg::RecordProof {
                schema_id: "test".to_string(),
                subject: someone.to_string(),
                proof_hash: "hash".to_string(),
                issuer: issuer.to_string(),
                expires_at: None,
            },
            &[],
        )
        .unwrap_err();

    // The error should indicate unauthorized recorder
    assert!(
        err.root_cause().to_string().contains("verifier contract or admin"),
        "Expected unauthorized recorder error, got: {}",
        err.root_cause()
    );
}

#[test]
fn schema_query_after_registration() {
    let mut suite = setup();

    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.registry_addr.clone(),
            &RegistryExecuteMsg::RegisterSchema {
                schema_id: "purchase_proof".to_string(),
                name: "Purchase Proof".to_string(),
                description: "Proves a purchase was made".to_string(),
                verifier_contract: suite.verifier_addr.to_string(),
                credential_types: vec!["purchase".to_string(), "receipt".to_string()],
            },
            &[],
        )
        .unwrap();

    let schema: SchemaResponse = suite
        .app
        .wrap()
        .query_wasm_smart(
            &suite.registry_addr,
            &RegistryQueryMsg::Schema {
                schema_id: "purchase_proof".to_string(),
            },
        )
        .unwrap();

    assert_eq!(schema.schema.name, "Purchase Proof");
    assert_eq!(
        schema.schema.credential_types,
        vec!["purchase", "receipt"]
    );
    assert_eq!(schema.schema.verifier_contract, suite.verifier_addr);
    assert!(schema.schema.active);
}
