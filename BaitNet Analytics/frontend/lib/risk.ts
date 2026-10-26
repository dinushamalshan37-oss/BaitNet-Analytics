import type { Verdict } from "./types";

export function riskColor(verdict: Verdict): string {
  switch (verdict) {
    case "Malicious":
      return "#fb7185";
    case "Suspicious":
      return "#fbbf24";
    case "Safe":
      return "#67e8c1";
    default:
      return "#94a3b8";
  }
}
