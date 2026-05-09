CREATE INDEX IF NOT EXISTS wallpaper_comments_wallpaper_created_idx ON wallpaper_comments(wallpaper_id, created_at DESC);
DROP INDEX IF EXISTS wallpaper_comments_wallpaper_id_idx;
DROP INDEX IF EXISTS wallpaper_comments_created_at_idx;
