ALTER TABLE wallpapers ADD COLUMN moderation_status VARCHAR NOT NULL DEFAULT 'active';
ALTER TABLE reported_wallpapers ADD COLUMN notes TEXT;
ALTER TABLE dmca_claims ADD COLUMN notes TEXT;
ALTER TABLE dmca_claims ADD COLUMN evidence_url TEXT;
ALTER TABLE dmca_claims ADD COLUMN duplicate_of_id TEXT REFERENCES dmca_claims(id);

CREATE TABLE IF NOT EXISTS dmca_counter_notices (
    id TEXT PRIMARY KEY,
    claim_id TEXT NOT NULL REFERENCES dmca_claims(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
