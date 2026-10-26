"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { SearchBar } from "@/components/SearchBar";
import { detectIndicatorType } from "@/lib/detect";

type Recent = { value: string; kind: string; timestamp: number };

export default function HomePage() {
  const [recent, setRecent] = useState<Recent[]>([]);
  useEffect(() => {
    try {
      const parsed: unknown = JSON.parse(localStorage.getItem("signalwatch-recent") ?? "[]");
      if (Array.isArray(parsed)) setRecent(parsed.filter((item): item is Recent => typeof item?.value === "string" && typeof item?.kind === "string").slice(0, 5));
    } catch {
      localStorage.removeItem("signalwatch-recent");
    }
  }, []);

  return (
    <div className="mx-auto max-w-7xl px-5">
      <section className="mx-auto max-w-4xl pb-10 pt-20 text-center sm:pt-28">
        <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-mint/20 bg-mint/5 px-3 py-1.5 text-xs font-medium text-mint">
          <span className="h-1.5 w-1.5 rounded-full bg-mint" /> Public-source threat intelligence
        </div>
        <h1 className="text-4xl font-semibold tracking-tight text-white sm:text-6xl">
          Know what you&apos;re <span className="text-mint">looking at.</span>
        </h1>
        <p className="mx-auto mt-5 max-w-2xl text-base leading-7 text-slate-400 sm:text-lg">
          Check suspicious IPs, domains, links, emails, phone numbers, and crypto wallets against public intelligence sources.
        </p>
        <div className="mt-9 text-left"><SearchBar /></div>
        <p className="mt-4 text-xs text-slate-500">No scanning or account access. Searches query public reputation and registration data.</p>
      </section>

      <section className="mx-auto grid max-w-4xl gap-4 pb-16 sm:grid-cols-3">
        {[
          ["Correlate", "Connect public DNS, domain, and reported-indicator signals."],
          ["Understand", "See a transparent score and the signals behind it."],
          ["Report", "Share a concise, factual report to help protect others."],
        ].map(([title, text], index) => (
          <article key={title} className="panel-card">
            <span className="text-xs font-semibold tracking-widest text-mint">0{index + 1}</span>
            <h2 className="mt-3 font-semibold text-white">{title}</h2>
            <p className="mt-2 text-sm leading-6 text-slate-400">{text}</p>
          </article>
        ))}
      </section>

      {recent.length > 0 && (
        <section className="mx-auto max-w-4xl pb-12">
          <h2 className="mb-3 text-sm font-semibold text-slate-300">Recent searches on this device</h2>
          <ul className="flex flex-wrap gap-2">
            {recent.map((item) => {
              const kind = detectIndicatorType(item.value) ?? item.kind;
              return <li key={`${item.value}-${item.timestamp}`}><Link className="inline-flex max-w-full items-center gap-2 rounded-full border border-line px-3 py-2 text-xs text-slate-300 hover:border-mint/50" href={`/result/${kind}/${encodeURIComponent(item.value)}`}><span className="truncate">{item.value}</span><span className="text-slate-500">{kind}</span></Link></li>;
            })}
          </ul>
        </section>
      )}
    </div>
  );
}
