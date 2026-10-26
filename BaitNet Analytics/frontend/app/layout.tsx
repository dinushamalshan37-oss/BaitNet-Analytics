import type { Metadata } from "next";
import Link from "next/link";
import "leaflet/dist/leaflet.css";
import "./globals.css";
import { QueryProvider } from "@/components/QueryProvider";

export const metadata: Metadata = {
  title: "SignalWatch · Public Threat Intelligence",
  description: "A defensive OSINT and anti-scam indicator tracker.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>
        <QueryProvider>
          <div className="min-h-screen">
            <header className="border-b border-line/80">
              <nav aria-label="Main navigation" className="mx-auto flex max-w-7xl items-center justify-between px-5 py-4">
                <Link href="/" className="flex items-center gap-2 text-lg font-semibold tracking-tight text-white">
                  <span className="grid h-8 w-8 place-items-center rounded-lg bg-mint text-sm font-black text-ink">S</span>
                  SignalWatch
                </Link>
                <div className="flex items-center gap-5 text-sm text-slate-400">
                  <Link href="/" className="transition hover:text-white">Search</Link>
                  <Link href="/feed" className="transition hover:text-white">Live feed</Link>
                </div>
              </nav>
            </header>
            <main>{children}</main>
            <footer className="mx-auto mt-16 max-w-7xl border-t border-line px-5 py-6 text-xs leading-relaxed text-slate-500">
              Defensive public-source intelligence only. Findings are indicators, not proof. Do not use this service to harass, identify, or doxx people.
            </footer>
          </div>
        </QueryProvider>
      </body>
    </html>
  );
}
