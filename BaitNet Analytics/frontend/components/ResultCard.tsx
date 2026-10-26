import type { ProviderResult } from "@/lib/types";

function summarize(provider: ProviderResult): string[] {
  const data = provider.data;
  if (provider.provider === "virustotal") {
    const stats = data.data && typeof data.data === "object" ? (data.data as Record<string, unknown>).attributes : undefined;
    const analysis = stats && typeof stats === "object" ? (stats as Record<string, unknown>).last_analysis_stats : undefined;
    const values = analysis && typeof analysis === "object" ? (analysis as Record<string, unknown>) : {};
    return [`Malicious detections: ${String(values.malicious ?? 0)}`, `Suspicious detections: ${String(values.suspicious ?? 0)}`];
  }
  if (provider.provider === "dns") {
    return ["addresses", "mx", "nameservers"].map((key) => {
      const value = data[key];
      return `${key}: ${Array.isArray(value) && value.length ? value.join(", ") : "none found"}`;
    });
  }
  if (provider.provider === "page_scraper") {
    return [
      `Linked domains: ${Array.isArray(data.linked_domains) ? data.linked_domains.length : 0}`,
      `Contact email(s) found: ${String(data.emails_found ?? 0)}`,
      `Phone number(s) found: ${String(data.phones_found ?? 0)}`,
      `Login/payment forms: ${Array.isArray(data.forms) ? data.forms.filter((form) => form && typeof form === "object" && Array.isArray((form as Record<string, unknown>).indicators) && ((form as Record<string, unknown>).indicators as string[]).length > 0).length : 0}`,
    ];
  }
  if (provider.provider === "hibp") {
    const breaches = Array.isArray(data.breaches) ? data.breaches.length : 0;
    return [`Breach records: ${breaches} (exposure information only)`];
  }
  return Object.entries(data).slice(0, 4).map(([key, value]) => `${key}: ${String(value)}`);
}

export function ResultCard({ provider }: { provider: ProviderResult }) {
  return (
    <section className="panel-card">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="font-semibold capitalize text-white">{provider.provider.replaceAll("_", " ")}</h3>
        <span className="rounded-full bg-white/5 px-2 py-1 text-[10px] uppercase tracking-wider text-slate-400">Public source</span>
      </div>
      <ul className="space-y-2 text-sm text-slate-300">
        {summarize(provider).map((line) => <li key={line} className="break-words">{line}</li>)}
      </ul>
    </section>
  );
}
