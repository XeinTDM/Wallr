ALTER TABLE wallpapers ADD COLUMN IF NOT EXISTS phash BYTEA;

CREATE TABLE IF NOT EXISTS banned_hashes (
    id TEXT PRIMARY KEY,
    phash BYTEA NOT NULL,
    reason TEXT,
    added_by TEXT REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_banned_hashes_phash ON banned_hashes(phash);
