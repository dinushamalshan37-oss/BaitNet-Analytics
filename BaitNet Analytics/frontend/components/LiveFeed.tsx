import type { FeedItem } from "@/lib/types";
import { riskColor } from "@/lib/risk";

export function LiveFeed({ items }: { items: FeedItem[] }) {
  if (!items.length) return <p className="panel-card text-sm text-slate-400">No recent high-risk indicators or reports.</p>;
  return (
    <ul className="divide-y divide-line overflow-hidden rounded-2xl border border-line bg-panel">
      {items.map((item) => (
        <li key={`${item.type}-${item.id}`} className="flex flex-wrap items-start justify-between gap-4 p-4 sm:p-5">
          <div className="min-w-0">
            <div className="mb-2 flex flex-wrap items-center gap-2">
              <span className="rounded-full bg-white/5 px-2 py-1 text-[10px] uppercase tracking-wider text-slate-400">{item.type === "report" ? `Report · ${item.category}` : item.verdict}</span>
              <span className="text-xs capitalize text-slate-500">{item.kind}</span>
            </div>
            <p className="break-all font-mono text-sm text-slate-100">{item.value}</p>
            {item.description && <p className="mt-2 text-sm text-slate-400">{item.description}</p>}
            <time className="mt-2 block text-xs text-slate-500" dateTime={item.created_at}>{new Date(item.created_at).toLocaleString()}</time>
          </div>
          {typeof item.score === "number" && <span className="font-semibold" style={{ color: riskColor(item.verdict ?? "Unknown") }}>{item.score}/100</span>}
        </li>
      ))}
    </ul>
  );
}
