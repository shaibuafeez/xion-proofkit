// ── Client classes ─────────────────────────────────────────────────
export { CredentialRegistryClient, CredentialRegistryExecuteClient } from "./credential-registry";
export { VerifierClient, VerifierExecuteClient } from "./verifier";
export { IssuerRegistryClient, IssuerRegistryExecuteClient } from "./issuer-registry";
export { ProofKit, SigningProofKit } from "./proofkit";

// ── Types ──────────────────────────────────────────────────────────
export type {
  // Credential Registry
  CredentialRegistryInstantiateMsg,
  RegisterSchemaMsg,
  RecordProofMsg,
  RevokeProofMsg,
  CredentialSchema,
  ProofRecord,
  IsVerifiedResponse,
  SchemaResponse,
  SchemasResponse,
  ProofRecordResponse,
  ProofRecordsResponse,

  // Verifier
  VerifierInstantiateMsg,
  VerifyCredentialMsg,
  VerifyEmailCredentialMsg,
  VerificationRequest,
  BatchVerifyMsg,
  VerificationType,
  VerificationRecord,
  VerificationResult,
  VerificationResultResponse,
  VerificationResultsResponse,
  BatchVerifyResponse,
  VerifierConfig,
  ConfigResponse,

  // Issuer Registry
  IssuerRegistryInstantiateMsg,
  RegisterIssuerMsg,
  RevokeIssuerMsg,
  UpdateIssuerMsg,
  IssuerRecord,
  IsAuthorizedResponse,
  IssuerResponse,
  IssuersResponse,
  AdminResponse,

  // Deploy
  DeployResult,
} from "./types";
