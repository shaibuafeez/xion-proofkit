/**
 * Parse a comma-separated string into a trimmed, non-empty list.
 * Throws if the result is empty.
 */
export function splitList(raw: string): string[] {
  const items = raw.split(",").map(s => s.trim()).filter(Boolean);
  if (items.length === 0) {
    throw new Error("Expected at least one value in comma-separated list");
  }
  return items;
}

/**
 * Strictly parse a string as an integer.
 * Rejects floats, scientific notation, trailing garbage (e.g. "10abc"), and NaN.
 */
export function parseNum(value: string): number {
  if (!/^-?\d+$/.test(value)) {
    throw new Error(`Invalid integer: "${value}"`);
  }
  return Number(value);
}
