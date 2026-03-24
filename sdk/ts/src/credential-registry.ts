import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import {
  IsVerifiedResponse,
  SchemaResponse,
  SchemasResponse,
  ProofRecordResponse,
  ProofRecordsResponse,
  AdminResponse,
} from "./types";

export class CredentialRegistryClient {
  constructor(
    private readonly client: CosmWasmClient,
    public readonly contractAddress: string,
  ) {}

  // ── Queries ──────────────────────────────────────────────────────

  async isVerified(subject: string, schemaId: string): Promise<IsVerifiedResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      is_verified: { subject, schema_id: schemaId },
    });
  }

  async getProofRecord(proofId: number): Promise<ProofRecordResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      proof_record: { proof_id: proofId },
    });
  }

  async getProofsBySubject(
    subject: string,
    startAfter?: number,
    limit?: number,
  ): Promise<ProofRecordsResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      proofs_by_subject: { subject, start_after: startAfter, limit },
    });
  }

  async getSchema(schemaId: string): Promise<SchemaResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      schema: { schema_id: schemaId },
    });
  }

  async listSchemas(startAfter?: string, limit?: number): Promise<SchemasResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      list_schemas: { start_after: startAfter, limit },
    });
  }

  async getAdmin(): Promise<AdminResponse> {
    return this.client.queryContractSmart(this.contractAddress, { admin: {} });
  }
}

export class CredentialRegistryExecuteClient extends CredentialRegistryClient {
  private readonly signingClient: SigningCosmWasmClient;
  private readonly sender: string;

  constructor(
    signingClient: SigningCosmWasmClient,
    sender: string,
    contractAddress: string,
  ) {
    super(signingClient, contractAddress);
    this.signingClient = signingClient;
    this.sender = sender;
  }

  // ── Execute ────────────────────────────────────────────────────

  async registerSchema(
    schemaId: string,
    name: string,
    description: string,
    verifierContract: string,
    credentialTypes: string[],
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      {
        register_schema: {
          schema_id: schemaId,
          name,
          description,
          verifier_contract: verifierContract,
          credential_types: credentialTypes,
        },
      },
      fee,
    );
  }

  async revokeProof(
    proofId: number,
    reason: string,
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      { revoke_proof: { proof_id: proofId, reason } },
      fee,
    );
  }

  async updateAdmin(
    newAdmin: string,
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      { update_admin: { new_admin: newAdmin } },
      fee,
    );
  }
}
