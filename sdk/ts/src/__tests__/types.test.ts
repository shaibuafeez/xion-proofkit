import { describe, it, expectTypeOf } from "vitest";
import type {
  ProofRecord,
  IsVerifiedResponse,
  IssuerRecord,
  IsAuthorizedResponse,
} from "../types";

describe("nullable type correctness", () => {
  it("ProofRecord nullable fields accept null", () => {
    const record: ProofRecord = {
      id: 1,
      schema_id: "test",
      subject: "xion1abc",
      proof_hash: "hash",
      issuer: "xion1def",
      verified_at: 1000,
      expires_at: null,
      revoked: false,
      revoked_at: null,
      revocation_reason: null,
    };
    expectTypeOf(record.expires_at).toEqualTypeOf<number | null>();
    expectTypeOf(record.revoked_at).toEqualTypeOf<number | null>();
    expectTypeOf(record.revocation_reason).toEqualTypeOf<string | null>();
  });

  it("ProofRecord nullable fields accept values", () => {
    const record: ProofRecord = {
      id: 1,
      schema_id: "test",
      subject: "xion1abc",
      proof_hash: "hash",
      issuer: "xion1def",
      verified_at: 1000,
      expires_at: 2000,
      revoked: true,
      revoked_at: 1500,
      revocation_reason: "expired",
    };
    expectTypeOf(record.expires_at).toEqualTypeOf<number | null>();
  });

  it("IsVerifiedResponse nullable fields", () => {
    const resp: IsVerifiedResponse = {
      verified: false,
      proof_id: null,
      expires_at: null,
    };
    expectTypeOf(resp.proof_id).toEqualTypeOf<number | null>();
    expectTypeOf(resp.expires_at).toEqualTypeOf<number | null>();
  });

  it("IssuerRecord nullable fields", () => {
    const issuer: IssuerRecord = {
      address: "xion1abc",
      name: "Test",
      description: "Test issuer",
      credential_types: ["age"],
      registered_at: 1000,
      active: true,
      revoked_at: null,
      revocation_reason: null,
    };
    expectTypeOf(issuer.revoked_at).toEqualTypeOf<number | null>();
    expectTypeOf(issuer.revocation_reason).toEqualTypeOf<string | null>();
  });

  it("IsAuthorizedResponse nullable fields", () => {
    const resp: IsAuthorizedResponse = {
      authorized: true,
      issuer_name: null,
    };
    expectTypeOf(resp.issuer_name).toEqualTypeOf<string | null>();
  });
});
