import type { FeedItem, SearchResult, Stats } from "./types";

const API_URL = (process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080").replace(/\/$/, "");

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${API_URL}/api${path}`, {
      ...init,
      headers: { "Content-Type": "application/json", ...init?.headers },
    });
  } catch {
    throw new Error("The intelligence API could not be reached. Check that the backend is running.");
  }

  if (!response.ok) {
    const body: unknown = await response.json().catch(() => null);
    const message =
      typeof body === "object" && body !== null && "message" in body && typeof body.message === "string"
        ? body.message
        : `Request failed (${response.status})`;
    throw new Error(message);
  }
  return (await response.json()) as T;
}

export function searchIndicator(value: string): Promise<SearchResult> {
  return request<SearchResult>(`/search?q=${encodeURIComponent(value)}`);
}

export function getFeed(limit = 30): Promise<{ items: FeedItem[] }> {
  return request<{ items: FeedItem[] }>(`/feed?limit=${limit}`);
}

export function getStats(): Promise<Stats> {
  return request<Stats>("/stats");
}

export function submitReport(payload: {
  indicator: string;
  category: string;
  description: string;
}): Promise<{ id: string; status: string }> {
  return request<{ id: string; status: string }>("/report", { method: "POST", body: JSON.stringify(payload) });
}
