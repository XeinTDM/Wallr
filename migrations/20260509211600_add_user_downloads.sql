CREATE TABLE IF NOT EXISTS user_downloads (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallpaper_id TEXT NOT NULL REFERENCES wallpapers(id) ON DELETE CASCADE,
    downloaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, wallpaper_id)
);

CREATE INDEX IF NOT EXISTS idx_user_downloads_user_id ON user_downloads(user_id);
