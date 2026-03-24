import { describe, it, expect, vi, beforeEach } from "vitest";

// We test the helper functions by importing the CLI module internals.
// Since they're not exported, we replicate them here to test the logic directly.

function splitList(raw: string): string[] {
  const items = raw.split(",").map(s => s.trim()).filter(Boolean);
  if (items.length === 0) {
    throw new Error("Expected at least one value in comma-separated list");
  }
  return items;
}

function parseOptionalNum(value: string | undefined): number | undefined {
  if (value === undefined) return undefined;
  const n = parseInt(value, 10);
  if (isNaN(n)) {
    throw new Error(`Invalid number: "${value}"`);
  }
  return n;
}

describe("splitList", () => {
  it("splits simple comma-separated values", () => {
    expect(splitList("age,employment")).toEqual(["age", "employment"]);
  });

  it("trims whitespace around values", () => {
    expect(splitList("age, employment, identity")).toEqual(["age", "employment", "identity"]);
  });

  it("filters empty entries from double commas", () => {
    expect(splitList("age,,employment")).toEqual(["age", "employment"]);
  });

  it("handles trailing comma", () => {
    expect(splitList("age,")).toEqual(["age"]);
  });

  it("handles leading comma", () => {
    expect(splitList(",age")).toEqual(["age"]);
  });

  it("throws on empty string", () => {
    expect(() => splitList("")).toThrow("at least one value");
  });

  it("throws on only commas", () => {
    expect(() => splitList(",,")).toThrow("at least one value");
  });

  it("preserves single value", () => {
    expect(splitList("age_proof")).toEqual(["age_proof"]);
  });
});

describe("parseOptionalNum", () => {
  it("parses valid integers", () => {
    expect(parseOptionalNum("42")).toBe(42);
  });

  it("returns undefined for undefined input", () => {
    expect(parseOptionalNum(undefined)).toBeUndefined();
  });

  it("parses zero", () => {
    expect(parseOptionalNum("0")).toBe(0);
  });

  it("throws on non-numeric input", () => {
    expect(() => parseOptionalNum("abc")).toThrow('Invalid number: "abc"');
  });

  it("throws on empty string", () => {
    expect(() => parseOptionalNum("")).toThrow("Invalid number");
  });

  it("parses negative numbers", () => {
    expect(parseOptionalNum("-1")).toBe(-1);
  });
});
