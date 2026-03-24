import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import {
  IsAuthorizedResponse,
  IssuerResponse,
  IssuersResponse,
  AdminResponse,
} from "./types";

export class IssuerRegistryClient {
  constructor(
    private readonly client: CosmWasmClient,
    public readonly contractAddress: string,
  ) {}

  // ── Queries ──────────────────────────────────────────────────────

  async isAuthorized(issuer: string, credentialType: string): Promise<IsAuthorizedResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      is_authorized: { issuer, credential_type: credentialType },
    });
  }

  async getIssuer(issuer: string): Promise<IssuerResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      issuer: { issuer },
    });
  }

  async listIssuers(startAfter?: string, limit?: number): Promise<IssuersResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      list_issuers: { start_after: startAfter, limit },
    });
  }

  async getIssuersByType(
    credentialType: string,
    startAfter?: string,
    limit?: number,
  ): Promise<IssuersResponse> {
    return this.client.queryContractSmart(this.contractAddress, {
      issuers_by_type: { credential_type: credentialType, start_after: startAfter, limit },
    });
  }

  async getAdmin(): Promise<AdminResponse> {
    return this.client.queryContractSmart(this.contractAddress, { admin: {} });
  }
}

export class IssuerRegistryExecuteClient extends IssuerRegistryClient {
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

  async registerIssuer(
    issuer: string,
    name: string,
    description: string,
    credentialTypes: string[],
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      {
        register_issuer: {
          issuer,
          name,
          description,
          credential_types: credentialTypes,
        },
      },
      fee,
    );
  }

  async revokeIssuer(
    issuer: string,
    reason: string,
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      { revoke_issuer: { issuer, reason } },
      fee,
    );
  }

  async updateIssuer(
    issuer: string,
    updates: {
      name?: string;
      description?: string;
      credential_types?: string[];
    },
    fee: "auto" | number = "auto",
  ): Promise<ExecuteResult> {
    return this.signingClient.execute(
      this.sender,
      this.contractAddress,
      {
        update_issuer: {
          issuer,
          ...updates,
        },
      },
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
