ALTER TABLE wallpapers ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
CREATE INDEX IF NOT EXISTS wallpapers_created_at_idx ON wallpapers (created_at DESC);
