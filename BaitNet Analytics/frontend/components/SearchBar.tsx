"use client";

import { FormEvent, useState } from "react";
import { useRouter } from "next/navigation";
import { detectIndicatorType } from "@/lib/detect";

export function SearchBar({ initialValue = "" }: { initialValue?: string }) {
  const [value, setValue] = useState(initialValue);
  const [error, setError] = useState("");
  const router = useRouter();
  const kind = detectIndicatorType(value);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalized = value.trim();
    const detected = detectIndicatorType(normalized);
    if (!detected) {
      setError("Enter a valid IP, domain, URL, email, international phone number, or crypto wallet.");
      return;
    }
    setError("");
    router.push(`/result/${detected}/${encodeURIComponent(normalized)}`);
  }

  return (
    <form onSubmit={submit} className="w-full">
      <label htmlFor="indicator-search" className="sr-only">Search a suspicious indicator</label>
      <div className="search-shell flex items-center gap-3 rounded-2xl border border-line bg-panel p-2 pl-5 shadow-xl shadow-black/20 focus-within:border-mint/70">
        <span aria-hidden="true" className="text-xl text-slate-400">⌕</span>
        <input
          id="indicator-search"
          autoComplete="off"
          value={value}
          onChange={(event) => setValue(event.target.value)}
          placeholder="Search IP, domain, URL, email, phone or wallet"
          className="min-w-0 flex-1 bg-transparent py-3 text-sm text-white outline-none placeholder:text-slate-500 sm:text-base"
        />
        {kind && <span className="hidden rounded-full bg-mint/10 px-3 py-1 text-xs font-medium capitalize text-mint sm:inline">{kind}</span>}
        <button className="rounded-xl bg-mint px-5 py-3 text-sm font-semibold text-ink transition hover:bg-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-mint">
          Analyze
        </button>
      </div>
      {error && <p role="alert" className="mt-3 text-sm text-rose-300">{error}</p>}
    </form>
  );
}
