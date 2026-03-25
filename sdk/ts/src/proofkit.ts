import { SigningCosmWasmClient, CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { CredentialRegistryClient, CredentialRegistryExecuteClient } from "./credential-registry";
import { VerifierClient, VerifierExecuteClient } from "./verifier";
import { IssuerRegistryClient, IssuerRegistryExecuteClient } from "./issuer-registry";
import { DeployResult, IsVerifiedResponse, VerificationRequest } from "./types";

/**
 * Read-only ProofKit client for querying contract state.
 */
export class ProofKit {
  public readonly registry: CredentialRegistryClient;
  public readonly verifier: VerifierClient;
  public readonly issuerRegistry: IssuerRegistryClient;

  constructor(
    client: CosmWasmClient,
    addresses: {
      credentialRegistry: string;
      verifier: string;
      issuerRegistry: string;
    },
  ) {
    this.registry = new CredentialRegistryClient(client, addresses.credentialRegistry);
    this.verifier = new VerifierClient(client, addresses.verifier);
    this.issuerRegistry = new IssuerRegistryClient(client, addresses.issuerRegistry);
  }

  /**
   * Quick check: is this subject verified for this schema?
   */
  async isVerified(subject: string, schemaId: string): Promise<boolean> {
    const result = await this.registry.isVerified(subject, schemaId);
    return result.verified;
  }

  /**
   * Check if an issuer is authorized for a credential type.
   */
  async isIssuerAuthorized(issuer: string, credentialType: string): Promise<boolean> {
    const result = await this.issuerRegistry.isAuthorized(issuer, credentialType);
    return result.authorized;
  }
}

/**
 * Full ProofKit client with signing capabilities for executing transactions.
 */
export class SigningProofKit {
  public readonly registry: CredentialRegistryExecuteClient;
  public readonly verifier: VerifierExecuteClient;
  public readonly issuerRegistry: IssuerRegistryExecuteClient;

  constructor(
    signingClient: SigningCosmWasmClient,
    sender: string,
    addresses: {
      credentialRegistry: string;
      verifier: string;
      issuerRegistry: string;
    },
  ) {
    this.registry = new CredentialRegistryExecuteClient(
      signingClient,
      sender,
      addresses.credentialRegistry,
    );
    this.verifier = new VerifierExecuteClient(signingClient, sender, addresses.verifier);
    this.issuerRegistry = new IssuerRegistryExecuteClient(
      signingClient,
      sender,
      addresses.issuerRegistry,
    );
  }

  /**
   * Quick check: is this subject verified for this schema?
   */
  async isVerified(subject: string, schemaId: string): Promise<boolean> {
    const result = await this.registry.isVerified(subject, schemaId);
    return result.verified;
  }

  /**
   * Deploy the full ProofKit contract suite.
   *
   * Uploads and instantiates all three contracts with proper cross-references.
   *
   * @param signingClient - Connected signing client
   * @param sender - Deployer/admin address
   * @param wasmPaths - Paths or bytes for each contract wasm
   */
  static async deploy(
    signingClient: SigningCosmWasmClient,
    sender: string,
    wasm: {
      credentialRegistry: Uint8Array;
      verifier: Uint8Array;
      issuerRegistry: Uint8Array;
    },
    fee: "auto" | number = "auto",
  ): Promise<{ proofkit: SigningProofKit; addresses: DeployResult }> {
    // Upload contracts sequentially (parallel uploads cause sequence mismatch)
    const regUpload = await signingClient.upload(sender, wasm.credentialRegistry, fee);
    const verUpload = await signingClient.upload(sender, wasm.verifier, fee);
    const issUpload = await signingClient.upload(sender, wasm.issuerRegistry, fee);

    // Instantiate issuer registry first (no dependencies)
    const issuerResult = await signingClient.instantiate(
      sender,
      issUpload.codeId,
      { admin: null },
      "proofkit-issuer-registry",
      fee,
    );

    // Instantiate credential registry (no dependencies)
    const registryResult = await signingClient.instantiate(
      sender,
      regUpload.codeId,
      { admin: null },
      "proofkit-credential-registry",
      fee,
    );

    // Instantiate verifier (depends on both registries)
    const verifierResult = await signingClient.instantiate(
      sender,
      verUpload.codeId,
      {
        admin: null,
        credential_registry: registryResult.contractAddress,
        issuer_registry: issuerResult.contractAddress,
      },
      "proofkit-verifier",
      fee,
    );

    const addresses: DeployResult = {
      credentialRegistry: registryResult.contractAddress,
      verifier: verifierResult.contractAddress,
      issuerRegistry: issuerResult.contractAddress,
    };

    const proofkit = new SigningProofKit(signingClient, sender, addresses);

    return { proofkit, addresses };
  }
}
