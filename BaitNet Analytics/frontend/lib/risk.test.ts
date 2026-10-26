import { describe, expect, it } from "vitest";
import { riskColor } from "./risk";

describe("riskColor", () => {
  it("maps verdicts to accessible risk colors", () => {
    expect(riskColor("Malicious")).toBe("#fb7185");
    expect(riskColor("Suspicious")).toBe("#fbbf24");
    expect(riskColor("Safe")).toBe("#67e8c1");
    expect(riskColor("Unknown")).toBe("#94a3b8");
  });
});
