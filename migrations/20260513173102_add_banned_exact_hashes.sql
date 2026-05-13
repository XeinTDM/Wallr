CREATE TABLE IF NOT EXISTS banned_exact_hashes (
    id TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL UNIQUE,
    reason TEXT NOT NULL,
    added_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
