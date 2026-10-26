import type { IndicatorKind } from "./types";

export function detectIndicatorType(value: string): IndicatorKind | null {
  const input = value.trim();
  if (!input) return null;
  if (/^0x[a-fA-F0-9]{40}$/.test(input)) return "ethereum";
  if (/^(?:[13][a-km-zA-HJ-NP-Z1-9]{25,34}|bc1[ac-hj-np-z02-9]{11,71})$/i.test(input)) return "bitcoin";
  if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(input)) return "email";
  if (/^\+?[0-9][0-9\s().-]{6,}[0-9]$/.test(input)) return "phone";
  if (input.includes("://")) {
    try {
      const url = new URL(input);
      if (url.protocol === "http:" || url.protocol === "https:") return "url";
    } catch {
      return null;
    }
    return null;
  }
  if (/^(?:\d{1,3}\.){3}\d{1,3}$/.test(input)) return "ipv4";
  if (input.includes(":") && /^[\da-f:]+$/i.test(input)) return "ipv6";
  if (/^[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\.?$/.test(input)) return "domain";
  return null;
}
