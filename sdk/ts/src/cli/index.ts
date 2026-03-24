#!/usr/bin/env node

import { readFileSync } from "fs";
import { resolve } from "path";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { SigningProofKit } from "../proofkit";
import { ProofKit } from "../proofkit";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { splitList, parseNum } from "./parse";

// ── Output ─────────────────────────────────────────────────────────

let jsonMode = false;

function output(data: Record<string, unknown>): void {
  if (jsonMode) {
    console.log(JSON.stringify(data));
  } else {
    for (const [key, value] of Object.entries(data)) {
      if (typeof value === "object" && value !== null && !Array.isArray(value)) {
        console.log(`${key}:`);
        for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
          console.log(`  ${k}: ${v}`);
        }
      } else if (Array.isArray(value)) {
        if (value.length === 0) {
          console.log(`No ${key} found.`);
        } else {
          for (const item of value) {
            if (typeof item === "object" && item !== null) {
              const parts = Object.entries(item).map(([k, v]) => `${k}=${v}`).join(", ");
              console.log(`  ${parts}`);
            } else {
              console.log(`  ${item}`);
            }
          }
        }
      } else {
        console.log(`${key}: ${value}`);
      }
    }
  }
}

function outputError(error: string, code?: string): void {
  if (jsonMode) {
    console.error(JSON.stringify({ error, code: code ?? "ERROR" }));
  } else {
    console.error(`Error: ${error}`);
  }
}

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
  --json              Output results as JSON (for scripting)

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
  outputError(`Missing required: ${flag}${envKey ? ` (or ${envKey} env)` : ""}`, "MISSING_ARG");
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
  if (idx === -1 || !args[idx + 1]) return undefined;
  try {
    return parseNum(args[idx + 1]);
  } catch {
    outputError(`Invalid number for ${flag}: "${args[idx + 1]}"`, "INVALID_INPUT");
    process.exit(1);
  }
}

function loadMnemonic(args: string[]): string {
  const keyfile = optionalArg(args, "--keyfile", "PROOFKIT_KEYFILE");
  if (keyfile) {
    return readFileSync(resolve(keyfile), "utf-8").trim();
  }
  if (process.env.PROOFKIT_MNEMONIC) {
    return process.env.PROOFKIT_MNEMONIC;
  }
  outputError("Missing mnemonic: set --keyfile <path>, PROOFKIT_KEYFILE, or PROOFKIT_MNEMONIC env", "MISSING_AUTH");
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

  if (!jsonMode) {
    console.log("Uploading and instantiating contracts...");
    console.log(`  Deployer: ${sender}`);
  }

  const wasm = {
    credentialRegistry: new Uint8Array(readFileSync(resolve(regPath))),
    verifier: new Uint8Array(readFileSync(resolve(verPath))),
    issuerRegistry: new Uint8Array(readFileSync(resolve(issPath))),
  };

  const { addresses } = await SigningProofKit.deploy(client, sender, wasm);

  output({
    deployer: sender,
    credential_registry: addresses.credentialRegistry,
    verifier: addresses.verifier,
    issuer_registry: addresses.issuerRegistry,
  });
}

async function registerSchema(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const schemaId = requireArg(args, "--schema-id");
  const name = requireArg(args, "--name");
  const description = requireArg(args, "--description");
  const verifierContract = optionalArg(args, "--verifier-contract") ?? addresses.verifier;
  const types = splitList(requireArg(args, "--credential-types"));

  if (!jsonMode) console.log(`Registering schema "${schemaId}"...`);
  const result = await pk.registry.registerSchema(schemaId, name, description, verifierContract, types);
  output({ schema_id: schemaId, tx: result.transactionHash });
}

async function registerIssuer(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const issuer = requireArg(args, "--issuer");
  const name = requireArg(args, "--name");
  const description = requireArg(args, "--description");
  const types = splitList(requireArg(args, "--credential-types"));

  if (!jsonMode) console.log(`Registering issuer "${name}"...`);
  const result = await pk.issuerRegistry.registerIssuer(issuer, name, description, types);
  output({ issuer, name, tx: result.transactionHash });
}

async function revokeIssuer(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const issuer = requireArg(args, "--issuer");
  const reason = requireArg(args, "--reason");

  if (!jsonMode) console.log(`Revoking issuer ${issuer}...`);
  const result = await pk.issuerRegistry.revokeIssuer(issuer, reason);
  output({ issuer, reason, tx: result.transactionHash });
}

async function verify(args: string[]) {
  const { client, sender } = await getSigningClient(args);
  const addresses = getAddresses(args);
  const pk = new SigningProofKit(client, sender, addresses);

  const schemaId = requireArg(args, "--schema-id");
  const subject = requireArg(args, "--subject");
  const issuer = requireArg(args, "--issuer");
  const proof = requireArg(args, "--proof");
  const publicInputs = splitList(requireArg(args, "--public-inputs"));
  const expiresAt = optionalNum(args, "--expires-at");

  if (!jsonMode) console.log(`Submitting ZK verification for ${subject}...`);
  const result = await pk.verifier.verifyCredential(schemaId, subject, issuer, proof, publicInputs, expiresAt);
  output({ schema_id: schemaId, subject, issuer, tx: result.transactionHash });
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

  if (!jsonMode) console.log(`Submitting email/DKIM verification for ${subject}...`);
  const result = await pk.verifier.verifyEmailCredential(
    schemaId, subject, issuer, emailDomain, dkimSignature, emailHeaders, expiresAt,
  );
  output({ schema_id: schemaId, subject, issuer, email_domain: emailDomain, tx: result.transactionHash });
}

async function isVerified(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const subject = requireArg(args, "--subject");
  const schemaId = requireArg(args, "--schema-id");

  const result = await pk.registry.isVerified(subject, schemaId);
  output({
    verified: result.verified,
    proof_id: result.proof_id,
    expires_at: result.expires_at,
  });
}

async function isAuthorized(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const issuer = requireArg(args, "--issuer");
  const credType = requireArg(args, "--credential-type");

  const result = await pk.issuerRegistry.isAuthorized(issuer, credType);
  output({
    authorized: result.authorized,
    issuer_name: result.issuer_name,
  });
}

async function listSchemas(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const startAfter = optionalArg(args, "--start-after");
  const limit = optionalNum(args, "--limit");

  const result = await pk.registry.listSchemas(startAfter, limit);
  output({ schemas: result.schemas });
}

async function listIssuers(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const startAfter = optionalArg(args, "--start-after");
  const limit = optionalNum(args, "--limit");

  const result = await pk.issuerRegistry.listIssuers(startAfter, limit);
  output({ issuers: result.issuers });
}

async function showConfig(args: string[]) {
  const client = await getQueryClient(args);
  const addresses = getAddresses(args);
  const pk = new ProofKit(client, addresses);

  const result = await pk.verifier.getConfig();
  output({
    admin: result.config.admin,
    credential_registry: result.config.credential_registry,
    issuer_registry: result.config.issuer_registry,
  });
}

// ── Main ───────────────────────────────────────────────────────────

async function main() {
  const args = process.argv.slice(2);

  // Check for --json flag before anything else
  jsonMode = args.includes("--json");

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
        outputError(`Unknown command: ${command}`, "UNKNOWN_COMMAND");
        if (!jsonMode) usage();
        process.exit(1);
    }
  } catch (err: any) {
    outputError(err.message, "EXECUTION_ERROR");
    process.exit(1);
  }
}

main();
