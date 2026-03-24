import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import {
  VerificationResultResponse,
  VerificationResultsResponse,
  ConfigResponse,
  VerificationRequest,
} from "./types";

export class VerifierClient {
  constructor(
    private readonly client: CosmWasmClient,
    public readonly contractAddress: string,
  ) {}

  // ── Queries ──────────────────────────────────────────────────────

  async getVerificationResult(verificationId: number): Promise<VerificationResultResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      verification_result: { verification_id: verificationId },
    });
  }

  async getVerificationsBySubject(
    subject: string,
    startAfter?: number,
    limit?: number,
  ): Promise<VerificationResultsResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      verifications_by_subject: { subject, start_after: startAfter, limit },
    });
  }

  async getConfig(): Promise<ConfigResponse> {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
}

export class VerifierExecuteClient extends VerifierClient {
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

  async verifyCredential(
    schemaId: string,
    subject: string,
    issuer: string,
    proof: string,
    publicInputs: string[],
    expiresAt?: number,
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      {
        verify_credential: {
          schema_id: schemaId,
          subject,
          issuer,
          proof,
          public_inputs: publicInputs,
          expires_at: expiresAt,
        },
      },
      fee,
    );
  }

  async verifyEmailCredential(
    schemaId: string,
    subject: string,
    issuer: string,
    emailDomain: string,
    dkimSignature: string,
    emailHeaders: string,
    expiresAt?: number,
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      {
        verify_email_credential: {
          schema_id: schemaId,
          subject,
          issuer,
          email_domain: emailDomain,
          dkim_signature: dkimSignature,
          email_headers: emailHeaders,
          expires_at: expiresAt,
        },
      },
      fee,
    );
  }

  async batchVerify(
    verifications: VerificationRequest[],
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      { batch_verify: { verifications } },
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

  async updateRegistry(
    credentialRegistry: string,
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      { update_registry: { credential_registry: credentialRegistry } },
      fee,
    );
  }
}
