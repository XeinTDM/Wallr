CREATE TABLE IF NOT EXISTS wallpaper_comments (
    id TEXT PRIMARY KEY,
    wallpaper_id TEXT NOT NULL REFERENCES wallpapers(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS wallpaper_comments_wallpaper_id_idx ON wallpaper_comments(wallpaper_id);
CREATE INDEX IF NOT EXISTS wallpaper_comments_created_at_idx ON wallpaper_comments(created_at DESC);
