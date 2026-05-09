CREATE INDEX IF NOT EXISTS wallpapers_feed_idx ON wallpapers (is_private, created_at DESC);
CREATE INDEX IF NOT EXISTS wallpapers_downloads_idx ON wallpapers (downloads DESC);
CREATE INDEX IF NOT EXISTS wallpapers_likes_idx ON wallpapers (likes DESC);
CREATE INDEX IF NOT EXISTS wallpapers_author_idx ON wallpapers (author);
