CREATE TABLE IF NOT EXISTS dmca_claims (
    id TEXT PRIMARY KEY,
    wallpaper_id TEXT NOT NULL REFERENCES wallpapers(id) ON DELETE CASCADE,
    claimant_name TEXT NOT NULL,
    claimant_email TEXT NOT NULL,
    original_url TEXT,
    description TEXT NOT NULL,
    digital_signature TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
