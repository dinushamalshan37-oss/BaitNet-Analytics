"use client";

import { useQuery } from "@tanstack/react-query";
import { Charts } from "@/components/Charts";
import { LiveFeed } from "@/components/LiveFeed";
import { getFeed, getStats } from "@/lib/api";

export default function FeedPage() {
  const feed = useQuery({ queryKey: ["feed"], queryFn: () => getFeed(50), refetchInterval: 10_000 });
  const stats = useQuery({ queryKey: ["stats"], queryFn: getStats, refetchInterval: 60_000 });

  return (
    <div className="mx-auto max-w-7xl px-5 py-10">
      <div className="mb-8 flex flex-wrap items-end justify-between gap-4">
        <div><p className="text-xs font-semibold uppercase tracking-widest text-mint">Community signals</p><h1 className="mt-2 text-3xl font-semibold text-white">Live intelligence feed</h1><p className="mt-2 text-sm text-slate-400">High-risk indicators and recent reports, with personal contact details masked.</p></div>
        <span className="rounded-full border border-line px-3 py-2 text-xs text-slate-400">Updates every 10 seconds</span>
      </div>

      {(feed.isError || stats.isError) && <p role="alert" className="panel-card mb-5 text-sm text-rose-200">{feed.error?.message ?? stats.error?.message}</p>}
      {stats.data && <div className="mb-8"><Charts stats={stats.data} /></div>}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-white">Recent activity</h2>
        {feed.isPending ? <div className="skeleton h-80" aria-label="Loading recent activity" /> : feed.data && <LiveFeed items={feed.data.items} />}
      </section>
    </div>
  );
}
