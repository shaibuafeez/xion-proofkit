#!/usr/bin/env node

import { readFileSync } from "fs";
import { resolve } from "path";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { SigningProofKit } from "../proofkit";
import { ProofKit } from "../proofkit";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

// ── Helpers ────────────────────────────────────────────────────────

function usage(): never {
  console.log(`
proofkit-cli — ProofKit contract management CLI

USAGE:
  proofkit <command> [options]

COMMANDS:
  deploy              Upload & instantiate all 3 contracts
  register-schema     Register a credential schema
  register-issuer     Register a trusted issuer
  revoke-issuer       Revoke a trusted issuer
  verify              Submit a ZK credential verification
  verify-email        Submit an email/DKIM verification
  is-verified         Check if a subject is verified for a schema
  is-authorized       Check if an issuer is authorized
  list-schemas        List registered schemas
  list-issuers        List registered issuers
  config              Show verifier config

GLOBAL OPTIONS:
  --rpc <url>         RPC endpoint (or PROOFKIT_RPC env)
  --keyfile <path>    Path to file containing mnemonic (or PROOFKIT_KEYFILE env)
  --gas-price <p>     Gas price (default: 0.025uxion)
  --prefix <p>        Bech32 prefix (default: xion)

AUTHENTICATION:
  The signer mnemonic is read from (in order):
    1. --keyfile <path>  (or PROOFKIT_KEYFILE env)
    2. PROOFKIT_MNEMONIC env variable
  Never pass mnemonics as CLI arguments — they leak into shell history.

CONTRACT ADDRESSES (required for non-deploy commands):
  --registry <addr>   Credential registry address (or PROOFKIT_REGISTRY env)
  --verifier <addr>   Verifier address (or PROOFKIT_VERIFIER env)
  --issuers <addr>    Issuer registry address (or PROOFKIT_ISSUERS env)
`);
  process.exit(1);
}

function requireArg(args: string[], flag: string, envKey?: string): string {
  const idx = args.indexOf(flag);
  if (idx !== -1 && args[idx + 1]) return args[idx + 1];
  if (envKey && process.env[envKey]) return process.env[envKey]!;
  console.error(`Missing required: ${flag}${envKey ? ` (or ${envKey} env)` : ""}`);
  process.exit(1);
}

function optionalArg(args: string[], flag: string, envKey?: string, fallback?: string): string | undefined {
  const idx = args.indexOf(flag);
  if (idx !== -1 && args[idx + 1]) return args[idx + 1];
  if (envKey && process.env[envKey]) return process.env[envKey]!;
  return fallback;
}

function optionalNum(args: string[], flag: string): number | undefined {
  const idx = args.indexOf(flag);
  if (idx !== -1 && args[idx + 1]) return parseInt(args[idx + 1], 10);
  return undefined;
}

function loadMnemonic(args: string[]): string {
  // 1. Keyfile (flag or env)
  const keyfile = optionalArg(args, "--keyfile", "PROOFKIT_KEYFILE");
  if (keyfile) {
    return readFileSync(resolve(keyfile), "utf-8").trim();
  }
  // 2. Environment variable
  if (process.env.PROOFKIT_MNEMONIC) {
    return process.env.PROOFKIT_MNEMONIC;
  }
  console.error("Missing mnemonic: set --keyfile <path>, PROOFKIT_KEYFILE, or PROOFKIT_MNEMONIC env");
  process.exit(1);
}

async function getSigningClient(args: string[]) {
  const rpc = requireArg(args, "--rpc", "PROOFKIT_RPC");
  const mnemonic = loadMnemonic(args);
  const gasPrice = optionalArg(args, "--gas-price", undefined, "0.025uxion");
  const prefix = optionalArg(args, "--prefix", undefined, "xion");

  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, { prefix: prefix! });
  const [account] = await wallet.getAccounts();
  const client = await SigningCosmWasmClient.connectWithSigner(rpc, wallet, {
    gasPrice: GasPrice.fromString(gasPrice!),
  });
  return { client, sender: account.address };
}

async function getQueryClient(args: string[]) {
  const rpc = requireArg(args, "--rpc", "PROOFKIT_RPC");
  return CosmWasmClient.connect(rpc);
}

function getAddresses(args: string[]) {
  return {
    credentialRegistry: requireArg(args, "--registry", "PROOFKIT_REGISTRY"),
    verifier: requireArg(args, "--verifier", "PROOFKIT_VERIFIER"),
    issuerRegistry: requireArg(args, "--issuers", "PROOFKIT_ISSUERS"),
  };
}

// ── Commands ───────────────────────────────────────────────────────

async function deploy(args: string[]) {
  const { client, sender } = await getSigningClient(args);

  const regPath = requireArg(args, "--wasm-registry");
  const verPath = requireArg(args, "--wasm-verifier");
  const issPath = requireArg(args, "--wasm-issuers");

  console.log("Uploading and instantiating contracts...");
  console.log(`  Deployer: ${sender}`);

  const wasm = {
    credentialRegistry: new Uint8Array(readFileSync(resolve(regPath))),
    verifier: new Uint8Array(readFileSync(resolve(verPath))),
    issuerRegistry: new Uint8Array(readFileSync(resolve(issPath))),
  };

  const { addresses } = await SigningProofKit.deploy(client, sender, wasm);

  console.log("\nDeployed successfully!");
  console.log(`  Credential Registry: ${addresses.credentialRegistry}`);
  console.log(`  Verifier:            ${addresses.verifier}`);
  console.log(`  Issuer Registry:     ${addresses.issuerRegistry}`);
  console.log("\nExport for future commands:");
  console.log(`  export PROOFKIT_REGISTRY=${addresses.credentialRegistry}`);
  console.log(`  export PROOFKIT_VERIFIER=${addresses.verifier}`);
  console.log(`  export PROOFKIT_ISSUERS=${addresses.issuerRegistry}`);
}

async function registerSchema(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const schemaId = requireArg(args, "--schema-id");
  const name = requireArg(args, "--name");
  const description = requireArg(args, "--description");
  const verifierContract = optionalArg(args, "--verifier-contract") ?? addresses.verifier;
  const types = requireArg(args, "--credential-types").split(",");

  console.log(`Registering schema "${schemaId}"...`);
  const result = await pk.registry.registerSchema(schemaId, name, description, verifierContract, types);
  console.log(`Done. TX: ${result.transactionHash}`);
}

async function registerIssuer(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const issuer = requireArg(args, "--issuer");
  const name = requireArg(args, "--name");
  const description = requireArg(args, "--description");
  const types = requireArg(args, "--credential-types").split(",");

  console.log(`Registering issuer "${name}"...`);
  const result = await pk.issuerRegistry.registerIssuer(issuer, name, description, types);
  console.log(`Done. TX: ${result.transactionHash}`);
}

async function revokeIssuer(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const issuer = requireArg(args, "--issuer");
  const reason = requireArg(args, "--reason");

  console.log(`Revoking issuer ${issuer}...`);
  const result = await pk.issuerRegistry.revokeIssuer(issuer, reason);
  console.log(`Done. TX: ${result.transactionHash}`);
}

async function verify(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const schemaId = requireArg(args, "--schema-id");
  const subject = requireArg(args, "--subject");
  const issuer = requireArg(args, "--issuer");
  const proof = requireArg(args, "--proof");
  const publicInputs = requireArg(args, "--public-inputs").split(",");
  const expiresAt = optionalNum(args, "--expires-at");

  console.log(`Submitting ZK verification for ${subject}...`);
  const result = await pk.verifier.verifyCredential(schemaId, subject, issuer, proof, publicInputs, expiresAt);
  console.log(`Done. TX: ${result.transactionHash}`);
}

async function verifyEmail(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const schemaId = requireArg(args, "--schema-id");
  const subject = requireArg(args, "--subject");
  const issuer = requireArg(args, "--issuer");
  const emailDomain = requireArg(args, "--email-domain");
  const dkimSignature = requireArg(args, "--dkim-signature");
  const emailHeaders = requireArg(args, "--email-headers");
  const expiresAt = optionalNum(args, "--expires-at");

  console.log(`Submitting email/DKIM verification for ${subject}...`);
  const result = await pk.verifier.verifyEmailCredential(
    schemaId, subject, issuer, emailDomain, dkimSignature, emailHeaders, expiresAt,
  );
  console.log(`Done. TX: ${result.transactionHash}`);
}

async function isVerified(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const subject = requireArg(args, "--subject");
  const schemaId = requireArg(args, "--schema-id");

  const verified = await pk.isVerified(subject, schemaId);
  console.log(`Verified: ${verified}`);
}

async function isAuthorized(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const issuer = requireArg(args, "--issuer");
  const credType = requireArg(args, "--credential-type");

  const authorized = await pk.isIssuerAuthorized(issuer, credType);
  console.log(`Authorized: ${authorized}`);
}

async function listSchemas(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const startAfter = optionalArg(args, "--start-after");
  const limit = optionalNum(args, "--limit");

  const result = await pk.registry.listSchemas(startAfter, limit);
  if (result.schemas.length === 0) {
    console.log("No schemas registered.");
    return;
  }
  for (const s of result.schemas) {
    console.log(`  ${s.schema_id}: ${s.name} (types: ${s.credential_types.join(", ")}, active: ${s.active})`);
  }
}

async function listIssuers(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const startAfter = optionalArg(args, "--start-after");
  const limit = optionalNum(args, "--limit");

  const result = await pk.issuerRegistry.listIssuers(startAfter, limit);
  if (result.issuers.length === 0) {
    console.log("No issuers registered.");
    return;
  }
  for (const iss of result.issuers) {
    console.log(`  ${iss.address}: ${iss.name} (types: ${iss.credential_types.join(", ")}, active: ${iss.active})`);
  }
}

async function showConfig(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const result = await pk.verifier.getConfig();
  console.log("Verifier Config:");
  console.log(`  Admin:               ${result.config.admin}`);
  console.log(`  Credential Registry: ${result.config.credential_registry}`);
  console.log(`  Issuer Registry:     ${result.config.issuer_registry}`);
}

// ── Main ───────────────────────────────────────────────────────────

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command || command === "--help" || command === "-h") usage();

  try {
    switch (command) {
      case "deploy":           await deploy(args); break;
      case "register-schema":  await registerSchema(args); break;
      case "register-issuer":  await registerIssuer(args); break;
      case "revoke-issuer":    await revokeIssuer(args); break;
      case "verify":           await verify(args); break;
      case "verify-email":     await verifyEmail(args); break;
      case "is-verified":      await isVerified(args); break;
      case "is-authorized":    await isAuthorized(args); break;
      case "list-schemas":     await listSchemas(args); break;
      case "list-issuers":     await listIssuers(args); break;
      case "config":           await showConfig(args); break;
      default:
        console.error(`Unknown command: ${command}`);
        usage();
    }
  } catch (err: any) {
    console.error(`Error: ${err.message}`);
    process.exit(1);
  }
}

main();
