import Link from "next/link";
import type { RelatedIndicator } from "@/lib/types";

export function RelatedList({ items }: { items: RelatedIndicator[] }) {
  return (
    <section className="panel-card">
      <h2 className="mb-4 text-lg font-semibold text-white">Related indicators</h2>
      {items.length ? (
        <ul className="divide-y divide-line">
          {items.map((item) => (
            <li key={`${item.id}-${item.relation_type}`} className="flex flex-wrap items-center justify-between gap-3 py-3">
              <div>
                <p className="break-all font-mono text-sm text-slate-200">{item.value}</p>
                <p className="mt-1 text-xs capitalize text-slate-500">{item.kind.replaceAll("_", " ")} · {item.relation_type.replaceAll("_", " ")}</p>
              </div>
              {item.kind === "email" || item.kind === "phone"
                ? <span className="text-xs text-slate-500">Personal data masked</span>
                : <Link className="text-xs font-medium text-mint hover:underline" href={`/result/${item.kind}/${encodeURIComponent(item.value)}`}>Analyze</Link>}
            </li>
          ))}
        </ul>
      ) : <p className="text-sm text-slate-400">No related public indicators found.</p>}
    </section>
  );
}
