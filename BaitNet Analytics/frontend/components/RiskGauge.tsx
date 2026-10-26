"use client";

import type { Verdict } from "@/lib/types";
import { riskColor } from "@/lib/risk";

export function RiskGauge({ score, verdict }: { score: number; verdict: Verdict }) {
  const color = riskColor(verdict);
  const value = Math.max(0, Math.min(100, score));
  return (
    <div className="flex flex-col items-center gap-3" aria-label={`Risk score ${value} out of 100, ${verdict}`}>
      <div
        className="relative grid h-44 w-44 place-items-center rounded-full"
        style={{ background: `conic-gradient(${color} ${value * 3.6}deg, #293744 ${value * 3.6}deg)` }}
      >
        <div className="grid h-36 w-36 place-content-center rounded-full bg-panel text-center">
          <span className="text-5xl font-semibold tracking-tight text-white">{value}</span>
          <span className="text-xs uppercase tracking-[0.2em] text-slate-400">Risk score</span>
        </div>
      </div>
      <span className="rounded-full px-3 py-1 text-sm font-semibold" style={{ backgroundColor: `${color}20`, color }}>
        {verdict}
      </span>
    </div>
  );
}
