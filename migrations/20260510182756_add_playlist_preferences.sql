ALTER TABLE users ADD COLUMN active_playlist_id TEXT REFERENCES user_collections(id) ON DELETE SET NULL;
ALTER TABLE users ADD COLUMN playlist_interval_secs INT DEFAULT 3600;
