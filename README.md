# ProofKit

Verified credential SDK for the [XION](https://xion.burnt.com/) blockchain. ProofKit gives developers a turnkey system for issuing, verifying, and querying zero-knowledge and DKIM-based credentials on-chain — backed by XION's native ZK Module and DKIM Module.

## What's Inside

```
proofkit/
├── contracts/                 # CosmWasm smart contracts
│   ├── credential-registry/   # Schema + proof storage
│   ├── verifier/              # ZK & DKIM verification engine
│   └── issuer-registry/       # Trusted issuer management
├── packages/
│   └── proofkit-types/        # Shared Rust types
├── sdk/
│   └── ts/                    # TypeScript SDK + CLI
├── tests/
│   └── integration/           # Cross-contract integration tests
└── artifacts/                 # Optimized wasm binaries
```

## Contracts

### Credential Registry

Stores credential schemas and proof records. Tracks verification status per subject/schema pair with expiration and revocation support.

- Register and manage credential schemas
- Store proof records (written by the verifier contract)
- Query verification status: `is_verified(subject, schema_id)`
- Revoke proofs with reason tracking
- Paginated queries for proofs and schemas

### Verifier

The verification engine. Accepts ZK proofs and DKIM email proofs, validates them against XION's native modules, and records results in the credential registry.

- ZK proof verification via XION ZK Module
- Email/DKIM verification via XION DKIM Module
- Batch verification (up to 20 per tx)
- Cross-contract issuer authorization checks
- Automatic proof recording via submessages

### Issuer Registry

Manages trusted issuers and their authorized credential types.

- Register/revoke/update issuers
- Authorization queries by issuer + credential type
- Secondary indexes for querying issuers by credential type

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.85+)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Node.js](https://nodejs.org/) (18+)

### Build Contracts

```bash
# Run all tests (72 unit + 8 integration)
cargo test --workspace

# Build optimized wasm
cargo wasm

# Further optimize with wasm-opt (install binaryen first)
mkdir -p artifacts
for f in target/wasm32-unknown-unknown/release/*.wasm; do
  wasm-opt -Oz --signext-lowering "$f" -o "artifacts/$(basename $f)"
done
```

### Install the SDK

```bash
npm install @proofkit/sdk
```

### Deploy

```typescript
import { SigningProofKit } from "@proofkit/sdk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { readFileSync } from "fs";

const wasm = {
  credentialRegistry: new Uint8Array(readFileSync("artifacts/credential_registry.wasm")),
  verifier: new Uint8Array(readFileSync("artifacts/verifier.wasm")),
  issuerRegistry: new Uint8Array(readFileSync("artifacts/issuer_registry.wasm")),
};

const { proofkit, addresses } = await SigningProofKit.deploy(client, sender, wasm);

console.log(addresses);
// { credentialRegistry: "xion1...", verifier: "xion1...", issuerRegistry: "xion1..." }
```

### Register a Schema and Issuer

```typescript
await proofkit.registry.registerSchema(
  "age-verification",
  "Age Verification",
  "Proves user is 18+",
  addresses.verifier,
  ["age_proof"],
);

await proofkit.issuerRegistry.registerIssuer(
  "xion1issuer...",
  "Acme Identity",
  "KYC provider",
  ["age_proof"],
);
```

### Verify a Credential

```typescript
// ZK proof
await proofkit.verifier.verifyCredential(
  "age-verification",
  "xion1subject...",
  "xion1issuer...",
  "base64-encoded-proof",
  ["public-input-1"],
);

// Email/DKIM proof
await proofkit.verifier.verifyEmailCredential(
  "email-verification",
  "xion1subject...",
  "xion1issuer...",
  "example.com",
  "dkim-signature-base64",
  "raw-email-headers",
);

// Check result
const verified = await proofkit.isVerified("xion1subject...", "age-verification");
// true
```

### Query-Only Client

```typescript
import { ProofKit } from "@proofkit/sdk";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

const client = await CosmWasmClient.connect("https://rpc.xion.burnt.com");
const pk = new ProofKit(client, {
  credentialRegistry: "xion1...",
  verifier: "xion1...",
  issuerRegistry: "xion1...",
});

const verified = await pk.isVerified("xion1subject...", "age-verification");
const authorized = await pk.isIssuerAuthorized("xion1issuer...", "age_proof");
```

## CLI

The SDK ships with a `proofkit` CLI for managing contracts from the terminal.

```bash
npx @proofkit/sdk --help
```

Set connection details via environment variables:

```bash
export PROOFKIT_RPC="https://rpc.xion.burnt.com"
export PROOFKIT_MNEMONIC="your mnemonic here"
export PROOFKIT_REGISTRY="xion1..."
export PROOFKIT_VERIFIER="xion1..."
export PROOFKIT_ISSUERS="xion1..."
```

Commands:

```bash
proofkit deploy --wasm-registry ./artifacts/credential_registry.wasm \
                --wasm-verifier ./artifacts/verifier.wasm \
                --wasm-issuers ./artifacts/issuer_registry.wasm

proofkit register-schema --schema-id age-verification --name "Age Check" \
                         --description "18+ proof" \
                         --verifier-contract xion1... \
                         --credential-types age_proof

proofkit register-issuer --issuer xion1... --name "Acme" \
                         --description "KYC provider" \
                         --credential-types age_proof

proofkit verify --schema-id age-verification --subject xion1... \
                --issuer xion1... --proof <base64> --public-inputs "input1,input2"

proofkit is-verified --subject xion1... --schema-id age-verification
proofkit is-authorized --issuer xion1... --credential-type age_proof
proofkit list-schemas
proofkit list-issuers
proofkit config
```

## Architecture

```
┌─────────────┐     submsg      ┌──────────────────────┐
│   Verifier  │ ──────────────> │  Credential Registry │
│             │                 │                      │
│  ZK Module  │                 │  Schemas + Proofs    │
│ DKIM Module │                 │  IsVerified queries  │
└──────┬──────┘                 └──────────────────────┘
       │ query
       v
┌──────────────────┐
│  Issuer Registry │
│                  │
│  Trusted issuers │
│  Authorization   │
└──────────────────┘
```

The **Verifier** is the entry point for all verification requests. It:

1. Validates the proof against XION's native ZK or DKIM module
2. Checks issuer authorization via the Issuer Registry
3. Records the result in the Credential Registry via submessage

Applications only need to query the **Credential Registry** to check if a subject is verified.

## JSON Schemas

Auto-generated API schemas for each contract are in `contracts/*/schema/`. Regenerate with:

```bash
cd contracts/credential-registry && cargo run --example schema
cd contracts/issuer-registry && cargo run --example schema
cd contracts/verifier && cargo run --example schema
```

## Testing

```bash
# All tests
cargo test --workspace

# Single contract
cargo test -p credential-registry
cargo test -p verifier
cargo test -p issuer-registry

# Integration tests only
cargo test -p proofkit-integration-tests
```

## License

[MIT](LICENSE)
