// ── Credential Registry Types ────────────────────────────────────────

export interface CredentialRegistryInstantiateMsg {
  admin?: string;
}

export interface RegisterSchemaMsg {
  register_schema: {
    schema_id: string;
    name: string;
    description: string;
    verifier_contract: string;
    credential_types: string[];
  };
}

export interface RecordProofMsg {
  record_proof: {
    schema_id: string;
    subject: string;
    proof_hash: string;
    issuer: string;
    expires_at?: number;
  };
}

export interface RevokeProofMsg {
  revoke_proof: {
    proof_id: number;
    reason: string;
  };
}

export interface CredentialSchema {
  schema_id: string;
  name: string;
  description: string;
  verifier_contract: string;
  credential_types: string[];
  created_at: number;
  active: boolean;
}

export interface ProofRecord {
  id: number;
  schema_id: string;
  subject: string;
  proof_hash: string;
  issuer: string;
  verified_at: number;
  expires_at?: number;
  revoked: boolean;
  revoked_at?: number;
  revocation_reason?: string;
}

export interface IsVerifiedResponse {
  verified: boolean;
  proof_id?: number;
  expires_at?: number;
}

export interface SchemaResponse {
  schema: CredentialSchema;
}

export interface SchemasResponse {
  schemas: CredentialSchema[];
}

export interface ProofRecordResponse {
  record: ProofRecord;
}

export interface ProofRecordsResponse {
  records: ProofRecord[];
}

// ── Verifier Types ──────────────────────────────────────────────────

export interface VerifierInstantiateMsg {
  admin?: string;
  credential_registry: string;
  issuer_registry: string;
}

export interface VerifyCredentialMsg {
  verify_credential: {
    schema_id: string;
    subject: string;
    issuer: string;
    proof: string;
    public_inputs: string[];
    expires_at?: number;
  };
}

export interface VerifyEmailCredentialMsg {
  verify_email_credential: {
    schema_id: string;
    subject: string;
    issuer: string;
    email_domain: string;
    dkim_signature: string;
    email_headers: string;
    expires_at?: number;
  };
}

export type VerificationRequest =
  | {
      zk_proof: {
        schema_id: string;
        subject: string;
        issuer: string;
        proof: string;
        public_inputs: string[];
        expires_at?: number;
      };
    }
  | {
      email_proof: {
        schema_id: string;
        subject: string;
        issuer: string;
        email_domain: string;
        dkim_signature: string;
        email_headers: string;
        expires_at?: number;
      };
    };

export interface BatchVerifyMsg {
  batch_verify: {
    verifications: VerificationRequest[];
  };
}

export type VerificationType = "zk_proof" | "email_dkim";

export interface VerificationRecord {
  id: number;
  schema_id: string;
  subject: string;
  issuer: string;
  verification_type: VerificationType;
  verified: boolean;
  verified_at: number;
  proof_hash: string;
}

export interface VerificationResult {
  verified: boolean;
  verification_id: number;
  schema_id: string;
  subject: string;
  verification_type: VerificationType;
  message: string;
}

export interface VerificationResultResponse {
  result: VerificationRecord;
}

export interface VerificationResultsResponse {
  results: VerificationRecord[];
}

export interface BatchVerifyResponse {
  results: VerificationResult[];
}

export interface VerifierConfig {
  admin: string;
  credential_registry: string;
  issuer_registry: string;
}

export interface ConfigResponse {
  config: VerifierConfig;
}

// ── Issuer Registry Types ───────────────────────────────────────────

export interface IssuerRegistryInstantiateMsg {
  admin?: string;
}

export interface RegisterIssuerMsg {
  register_issuer: {
    issuer: string;
    name: string;
    description: string;
    credential_types: string[];
  };
}

export interface RevokeIssuerMsg {
  revoke_issuer: {
    issuer: string;
    reason: string;
  };
}

export interface UpdateIssuerMsg {
  update_issuer: {
    issuer: string;
    name?: string;
    description?: string;
    credential_types?: string[];
  };
}

export interface IssuerRecord {
  address: string;
  name: string;
  description: string;
  credential_types: string[];
  registered_at: number;
  active: boolean;
  revoked_at?: number;
  revocation_reason?: string;
}

export interface IsAuthorizedResponse {
  authorized: boolean;
  issuer_name?: string;
}

export interface IssuerResponse {
  issuer: IssuerRecord;
}

export interface IssuersResponse {
  issuers: IssuerRecord[];
}

export interface AdminResponse {
  admin: string;
}

// ── Deploy result ───────────────────────────────────────────────────

export interface DeployResult {
  credentialRegistry: string;
  verifier: string;
  issuerRegistry: string;
}
