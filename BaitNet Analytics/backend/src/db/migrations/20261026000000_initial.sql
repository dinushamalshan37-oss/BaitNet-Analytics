CREATE TABLE indicators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind TEXT NOT NULL,
    value TEXT NOT NULL,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    risk_score SMALLINT NOT NULL DEFAULT 0 CHECK (risk_score BETWEEN 0 AND 100),
    verdict TEXT NOT NULL DEFAULT 'Unknown',
    UNIQUE (kind, value)
);

CREATE TABLE lookups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    indicator_id UUID NOT NULL REFERENCES indicators(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    raw_json JSONB NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX lookups_indicator_fetched_idx ON lookups(indicator_id, fetched_at DESC);

CREATE TABLE relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_indicator UUID NOT NULL REFERENCES indicators(id) ON DELETE CASCADE,
    to_indicator UUID NOT NULL REFERENCES indicators(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,
    UNIQUE(from_indicator, to_indicator, relation_type),
    CHECK(from_indicator <> to_indicator)
);

CREATE TABLE user_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    indicator_id UUID NOT NULL REFERENCES indicators(id) ON DELETE CASCADE,
    category TEXT NOT NULL,
    description TEXT NOT NULL CHECK (length(description) <= 2000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX user_reports_created_idx ON user_reports(created_at DESC);
