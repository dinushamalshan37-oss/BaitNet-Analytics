"use client";

import { FormEvent, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { submitReport } from "@/lib/api";

const categories = ["phishing", "fraud", "impersonation", "malware", "other"];

export function ReportModal({ indicator, onClose }: { indicator: string; onClose: () => void }) {
  const [category, setCategory] = useState(categories[0]);
  const [description, setDescription] = useState("");
  const mutation = useMutation({
    mutationFn: () => submitReport({ indicator, category, description }),
    onSuccess: onClose,
  });

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    mutation.mutate();
  }

  return (
    <div className="fixed inset-0 z-[1000] grid place-items-center bg-black/70 p-4" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
      <form onSubmit={submit} role="dialog" aria-modal="true" aria-labelledby="report-title" className="w-full max-w-lg rounded-2xl border border-line bg-panel p-6 shadow-2xl">
        <h2 id="report-title" className="text-xl font-semibold text-white">Report this indicator</h2>
        <p className="mt-2 break-all font-mono text-xs text-slate-400">{indicator}</p>
        <label className="mt-5 block text-sm text-slate-300">Category
          <select value={category} onChange={(event) => setCategory(event.target.value)} className="field mt-2">
            {categories.map((value) => <option key={value} value={value}>{value}</option>)}
          </select>
        </label>
        <label className="mt-4 block text-sm text-slate-300">What happened?
          <textarea required minLength={1} maxLength={2000} value={description} onChange={(event) => setDescription(event.target.value)} className="field mt-2 min-h-32 resize-y" placeholder="Share a brief, factual description. Avoid personal information." />
        </label>
        {mutation.isError && <p role="alert" className="mt-3 text-sm text-rose-300">{mutation.error.message}</p>}
        <div className="mt-5 flex justify-end gap-3">
          <button type="button" onClick={onClose} className="button-secondary">Cancel</button>
          <button disabled={mutation.isPending} className="button-primary">{mutation.isPending ? "Submitting…" : "Submit report"}</button>
        </div>
      </form>
    </div>
  );
}
