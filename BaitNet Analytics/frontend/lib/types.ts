export type IndicatorKind =
  | "ipv4"
  | "ipv6"
  | "domain"
  | "url"
  | "email"
  | "phone"
  | "bitcoin"
  | "ethereum";

export type Verdict = "Safe" | "Suspicious" | "Malicious" | "Unknown";

export interface Indicator {
  id: string | null;
  kind: IndicatorKind;
  value: string;
}

export interface ScoreReason {
  signal: string;
  weight: number;
  detail: string;
}

export interface ProviderResult {
  provider: string;
  data: Record<string, unknown>;
}

export interface RelatedIndicator {
  id: string;
  kind: string;
  value: string;
  relation_type: string;
}

export interface SearchResult {
  indicator: Indicator;
  score: number;
  verdict: Verdict;
  reasons: ScoreReason[];
  providers: ProviderResult[];
  related: RelatedIndicator[];
}

export interface FeedItem {
  id: string;
  type: "indicator" | "report";
  kind: IndicatorKind;
  value: string;
  score?: number;
  verdict?: Verdict;
  category?: string;
  description?: string;
  created_at: string;
}

export interface Stats {
  by_type: { kind: string; count: number }[];
  by_verdict: { verdict: string; count: number }[];
  by_day: { day: string; count: number }[];
}
