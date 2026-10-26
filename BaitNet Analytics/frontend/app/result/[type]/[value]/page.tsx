"use client";

import dynamic from "next/dynamic";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { ResultCard } from "@/components/ResultCard";
import { RelatedList } from "@/components/RelatedList";
import { RiskGauge } from "@/components/RiskGauge";
import { ReportModal } from "@/components/ReportModal";
import { searchIndicator } from "@/lib/api";
import type { ProviderResult } from "@/lib/types";

const GeoMap = dynamic(() => import("@/components/GeoMap").then((module) => module.GeoMap), {
  ssr: false,
  loading: () => <div className="skeleton h-72" aria-label="Loading map" />,
});

function geoCoordinates(provider?: ProviderResult): { latitude: number; longitude: number; label: string } | null {
  if (!provider) return null;
  const { latitude, longitude } = provider.data;
  if (typeof latitude !== "number" || typeof longitude !== "number") return null;
  const country = typeof provider.data.country === "string" ? provider.data.country : "Approximate location";
  const city = typeof provider.data.city === "string" ? provider.data.city : "";
  return { latitude, longitude, label: [city, country].filter(Boolean).join(", ") };
}

export default function ResultPage() {
  const params = useParams<{ type: string; value: string }>();
  const value = params.value;
  const [reportOpen, setReportOpen] = useState(false);
  const query = useQuery({ queryKey: ["search", value], queryFn: () => searchIndicator(value) });
  const result = query.data;
  const geo = geoCoordinates(result?.providers.find((provider) => provider.provider === "geoip"));

  useEffect(() => {
    if (!result) return;
    if (result.indicator.kind === "email" || result.indicator.kind === "phone") return;
    try {
      const existing: unknown = JSON.parse(localStorage.getItem("signalwatch-recent") ?? "[]");
      const items = Array.isArray(existing) ? existing.filter((item) => item && typeof item.value === "string") : [];
      const next = [{ value: result.indicator.value, kind: result.indicator.kind, timestamp: Date.now() }, ...items.filter((item) => item.value !== result.indicator.value)].slice(0, 5);
      localStorage.setItem("signalwatch-recent", JSON.stringify(next));
    } catch {
      localStorage.removeItem("signalwatch-recent");
    }
  }, [result]);

  return (
    <div className="mx-auto max-w-7xl px-5 py-9">
      <div className="mb-7 flex flex-wrap items-center justify-between gap-3">
        <Link href="/" className="text-sm text-slate-400 hover:text-white">← New search</Link>
        <button onClick={() => setReportOpen(true)} disabled={!result} className="button-secondary">Report this indicator</button>
      </div>

      {query.isPending && <div className="grid gap-5 lg:grid-cols-[270px_1fr]"><div className="skeleton h-72" /><div className="skeleton h-72" /></div>}
      {query.isError && <section role="alert" className="panel-card border-rose-400/30"><h1 className="font-semibold text-rose-200">Couldn&apos;t analyze this indicator</h1><p className="mt-2 text-sm text-slate-300">{query.error.message}</p><Link href="/" className="mt-4 inline-block text-sm text-mint">Try another search</Link></section>}

      {result && (
        <>
          <section className="panel-card mb-6 grid items-center gap-7 lg:grid-cols-[250px_1fr]">
            <RiskGauge score={result.score} verdict={result.verdict} />
            <div className="min-w-0">
              <span className="text-xs font-semibold uppercase tracking-widest text-slate-500">{result.indicator.kind}</span>
              <h1 className="mt-2 break-all font-mono text-xl font-semibold text-white sm:text-2xl">{result.indicator.value}</h1>
              <p className="mt-3 max-w-2xl text-sm leading-6 text-slate-400">
                This result is an evidence-based indicator, not a definitive conclusion. Source coverage can be incomplete or outdated.
              </p>
              <div className="mt-4 flex flex-wrap gap-2 text-xs text-slate-400">
                <span className="rounded-full bg-white/5 px-3 py-1.5">{result.providers.length} public source(s) responded</span>
                <span className="rounded-full bg-white/5 px-3 py-1.5">{result.related.length} related indicator(s)</span>
              </div>
            </div>
          </section>

          <div className="grid items-start gap-6 lg:grid-cols-[1fr_330px]">
            <div className="space-y-6">
              <section className="panel-card">
                <h2 className="text-lg font-semibold text-white">Why this score</h2>
                {result.reasons.length ? (
                  <ul className="mt-4 space-y-3">
                    {result.reasons.map((reason) => <li key={reason.signal} className="flex gap-3 rounded-xl bg-ink/70 p-3"><span className={`mt-0.5 min-w-10 text-sm font-semibold ${reason.weight > 0 ? "text-amber-300" : "text-slate-500"}`}>{reason.weight > 0 ? `+${reason.weight}` : "Info"}</span><div><p className="text-sm font-medium text-slate-200">{reason.detail}</p><p className="mt-1 text-xs text-slate-500">{reason.signal.replaceAll("_", " ")}</p></div></li>)}
                  </ul>
                ) : <p className="mt-3 text-sm text-slate-400">No actionable reputation signals were found in the available sources.</p>}
              </section>
              <RelatedList items={result.related} />
              {geo && <section className="panel-card"><h2 className="mb-4 text-lg font-semibold text-white">Approximate network location</h2><p className="mb-3 text-xs text-slate-400">{geo.label}. IP geolocation is approximate and does not identify a person.</p><GeoMap {...geo} /></section>}
              <section><h2 className="mb-3 text-lg font-semibold text-white">Provider findings</h2>{result.providers.length ? <div className="grid gap-3 sm:grid-cols-2">{result.providers.map((provider) => <ResultCard key={provider.provider} provider={provider} />)}</div> : <p className="panel-card text-sm text-slate-400">No public providers returned results.</p>}</section>
            </div>
            <aside className="panel-card text-sm leading-6 text-slate-400">
              <h2 className="font-semibold text-white">Use responsibly</h2>
              <p className="mt-2">This tool checks public indicators only. A high score is not proof of wrongdoing; a low score does not guarantee safety.</p>
              <p className="mt-3">Do not use these results for harassment, doxxing, or other harmful action. Avoid submitting personal or sensitive information.</p>
            </aside>
          </div>
        </>
      )}
      {reportOpen && result && <ReportModal indicator={result.indicator.value} onClose={() => setReportOpen(false)} />}
    </div>
  );
}
