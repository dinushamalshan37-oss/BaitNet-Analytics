"use client";

import { Bar, BarChart, CartesianGrid, Cell, ResponsiveContainer, Tooltip, XAxis, YAxis, Line, LineChart } from "recharts";
import type { Stats } from "@/lib/types";

const palette = ["#67e8c1", "#fbbf24", "#fb7185", "#94a3b8", "#60a5fa", "#c084fc"];

export function Charts({ stats }: { stats: Stats }) {
  return (
    <div className="grid gap-5 lg:grid-cols-2">
      <section className="panel-card">
        <h2 className="mb-5 font-semibold text-white">Indicators by verdict</h2>
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={stats.by_verdict}>
              <CartesianGrid stroke="#293744" vertical={false} />
              <XAxis dataKey="verdict" stroke="#94a3b8" tickLine={false} axisLine={false} />
              <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} allowDecimals={false} />
              <Tooltip contentStyle={{ background: "#17232e", border: "1px solid #293744", borderRadius: 12 }} />
              <Bar dataKey="count" radius={[6, 6, 0, 0]}>{stats.by_verdict.map((item, index) => <Cell key={item.verdict} fill={palette[index % palette.length]} />)}</Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
      </section>
      <section className="panel-card">
        <h2 className="mb-5 font-semibold text-white">New indicators · 30 days</h2>
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={stats.by_day}>
              <CartesianGrid stroke="#293744" vertical={false} />
              <XAxis dataKey="day" stroke="#94a3b8" tickLine={false} axisLine={false} tickFormatter={(day: string) => day.slice(5)} />
              <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} allowDecimals={false} />
              <Tooltip contentStyle={{ background: "#17232e", border: "1px solid #293744", borderRadius: 12 }} />
              <Line type="monotone" dataKey="count" stroke="#67e8c1" strokeWidth={2} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </section>
    </div>
  );
}
