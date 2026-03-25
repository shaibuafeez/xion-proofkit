# ProofKit

On-chain verified credentials for [XION](https://xion.burnt.com/).

ProofKit lets you verify things about your users — age, identity, employment, email ownership — and store the results on-chain where any app can query them. It uses XION's native ZK Module and DKIM Module, so proofs are validated at the protocol level, not by trusting a third party.

## Why

Web3 apps need to know things about users without seeing their data. "Is this user over 18?" "Do they work at this company?" "Do they own this email?" Today each app builds its own verification flow, or trusts a centralized oracle.

ProofKit replaces that with a shared, on-chain credential layer:

1. **Issuers** (KYC providers, employers, universities) are registered as trusted sources
2. **Users** submit zero-knowledge proofs or DKIM email proofs
3. **XION's native modules** validate the cryptography — not your contract, not a relayer
4. **Any app** can query the result with a single call: `isVerified("xion1user...", "age-verification")`

Once a credential is verified, it lives on-chain until it expires or is revoked. No re-verification, no API calls to third parties, no trust assumptions beyond the chain itself.

## How It Works

```
                                    ┌──────────────────────┐
  User submits proof                │  Credential Registry │
        │                          │                      │
        v                          │  Stores schemas and  │
  ┌─────────────┐     submsg       │  proof records.      │
  │   Verifier  │ ───────────────> │                      │
  │             │                  │  Any app can query:  │
  │ Validates   │                  │  isVerified(user, X) │
  │ via XION's  │                  └──────────────────────┘
  │ ZK + DKIM   │
  │ modules     │
  └──────┬──────┘
         │ checks issuer
         v
  ┌──────────────────┐
  │  Issuer Registry │
  │                  │
  │  Who is allowed  │
  │  to issue what?  │
  └──────────────────┘
```

**Three contracts, one flow:**

- **Verifier** — entry point. Receives proofs, validates them against XION's native ZK or DKIM module, checks that the issuer is authorized, then records the result.
- **Credential Registry** — storage. Holds credential schemas and proof records. This is what apps query.
- **Issuer Registry** — trust. Tracks which issuers are authorized for which credential types.

## Use Cases

- **Age verification** — User proves they're 18+ via a ZK proof from a KYC provider. DeFi apps, marketplaces, and social apps query the result.
- **Email ownership** — User proves they own an @company.com email via DKIM signature. DAOs and governance apps gate access by employer domain.
- **Identity credentials** — Passports, driver's licenses, national IDs verified through ZK proofs without revealing the underlying data.
- **Employment/education** — Prove employment or degree status from institutional email domains.
- **Batch onboarding** — Verify up to 20 credentials in a single transaction for bulk user onboarding.

## Quick Start

### Install the SDK

```bash
npm install xion-proofkit
```

### Deploy All Contracts

```typescript
import { SigningProofKit } from "xion-proofkit";
import { readFileSync } from "fs";

const wasm = {
  credentialRegistry: new Uint8Array(readFileSync("artifacts/credential_registry.wasm")),
  verifier: new Uint8Array(readFileSync("artifacts/verifier.wasm")),
  issuerRegistry: new Uint8Array(readFileSync("artifacts/issuer_registry.wasm")),
};

// Uploads and instantiates all 3 contracts with proper cross-references
const { proofkit, addresses } = await SigningProofKit.deploy(client, sender, wasm);
```

### Set Up a Schema and Issuer

```typescript
// Define what kind of credential you accept
await proofkit.registry.registerSchema(
  "age-verification",
  "Age Verification",
  "Proves user is 18+",
  addresses.verifier,
  ["age_proof"],
);

// Register a trusted issuer
await proofkit.issuerRegistry.registerIssuer(
  "xion1issuer...",
  "Acme Identity",
  "Licensed KYC provider",
  ["age_proof"],
);
```

### Verify a User

```typescript
// ZK proof verification
await proofkit.verifier.verifyCredential(
  "age-verification",    // schema
  "xion1user...",        // subject
  "xion1issuer...",      // issuer
  "base64-proof-data",  // proof
  ["public-input-1"],   // public inputs
);

// Email/DKIM verification
await proofkit.verifier.verifyEmailCredential(
  "email-verification",
  "xion1user...",
  "xion1issuer...",
  "company.com",
  "dkim-signature-base64",
  "raw-email-headers",
);
```

### Query From Any App

```typescript
import { ProofKit } from "xion-proofkit";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

const client = await CosmWasmClient.connect("https://rpc.xion.burnt.com");
const pk = new ProofKit(client, {
  credentialRegistry: "xion1...",
  verifier: "xion1...",
  issuerRegistry: "xion1...",
});

// One call — that's it
const verified = await pk.isVerified("xion1user...", "age-verification");
// true
```

## CLI

Manage everything from the terminal.

```bash
npx xion-proofkit --help
```

Configure via environment variables:

```bash
export PROOFKIT_RPC="https://rpc.xion.burnt.com"
export PROOFKIT_MNEMONIC="your mnemonic here"   # or use --keyfile
export PROOFKIT_REGISTRY="xion1..."
export PROOFKIT_VERIFIER="xion1..."
export PROOFKIT_ISSUERS="xion1..."
```

```bash
# Deploy
proofkit deploy --wasm-registry ./artifacts/credential_registry.wasm \
                --wasm-verifier ./artifacts/verifier.wasm \
                --wasm-issuers  ./artifacts/issuer_registry.wasm

# Manage
proofkit register-schema --schema-id age-verification --name "Age Check" \
                         --description "18+ proof" --credential-types age_proof
proofkit register-issuer --issuer xion1... --name "Acme" \
                         --description "KYC provider" --credential-types age_proof

# Verify
proofkit verify --schema-id age-verification --subject xion1... \
                --issuer xion1... --proof <base64> --public-inputs "input1,input2"

# Query
proofkit is-verified --subject xion1... --schema-id age-verification
proofkit is-authorized --issuer xion1... --credential-type age_proof
proofkit list-schemas
proofkit list-issuers
proofkit config
```

## Project Structure

```
proofkit/
├── contracts/
│   ├── credential-registry/   # Schema + proof storage, IsVerified queries
│   ├── verifier/              # ZK & DKIM verification engine
│   └── issuer-registry/       # Trusted issuer management
├── packages/
│   └── proofkit-types/        # Shared Rust types across all contracts
├── sdk/ts/                    # TypeScript SDK + CLI (npm: @proofkit/sdk)
├── tests/integration/         # Cross-contract integration tests
└── artifacts/                 # Optimized wasm binaries (~220KB each)
```

## Building From Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Node.js](https://nodejs.org/) 18+
- [binaryen](https://github.com/WebAssembly/binaryen) (for `wasm-opt`)

### Build

```bash
# Run all tests (72 unit + 8 integration + 21 SDK)
cargo test --workspace
cd sdk/ts && npm test

# Build optimized wasm
cargo wasm
mkdir -p artifacts
for f in target/wasm32-unknown-unknown/release/*.wasm; do
  wasm-opt -Oz --signext-lowering "$f" -o "artifacts/$(basename $f)"
done

# Build TypeScript SDK
cd sdk/ts && npm install && npm run build
```

### Test Individual Contracts

```bash
cargo test -p credential-registry   # 22 tests
cargo test -p verifier              # 19 tests
cargo test -p issuer-registry       # 23 tests
cargo test -p proofkit-integration-tests  # 8 cross-contract tests
```

## JSON Schemas

Auto-generated API schemas for each contract live in `contracts/*/schema/`. Regenerate with:

```bash
cd contracts/credential-registry && cargo run --example schema
cd contracts/issuer-registry && cargo run --example schema
cd contracts/verifier && cargo run --example schema
```

## License

[MIT](LICENSE)
