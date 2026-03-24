import { describe, it, expect } from "vitest";
import { splitList, parseNum } from "../cli/parse";

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

describe("parseNum", () => {
  it("parses valid integers", () => {
    expect(parseNum("42")).toBe(42);
  });

  it("parses zero", () => {
    expect(parseNum("0")).toBe(0);
  });

  it("parses negative numbers", () => {
    expect(parseNum("-1")).toBe(-1);
  });

  it("rejects non-numeric input", () => {
    expect(() => parseNum("abc")).toThrow('Invalid integer: "abc"');
  });

  it("rejects trailing garbage like 10abc", () => {
    expect(() => parseNum("10abc")).toThrow('Invalid integer: "10abc"');
  });

  it("rejects scientific notation like 1e3", () => {
    expect(() => parseNum("1e3")).toThrow('Invalid integer: "1e3"');
  });

  it("rejects floats", () => {
    expect(() => parseNum("3.14")).toThrow('Invalid integer: "3.14"');
  });

  it("rejects empty string", () => {
    expect(() => parseNum("")).toThrow('Invalid integer');
  });
});
