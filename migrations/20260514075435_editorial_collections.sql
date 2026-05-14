CREATE TABLE IF NOT EXISTS editorial_collections (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    cover_url TEXT,
    is_published BOOLEAN NOT NULL DEFAULT false,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS editorial_collection_items (
    collection_id TEXT NOT NULL REFERENCES editorial_collections(id) ON DELETE CASCADE,
    wallpaper_id TEXT NOT NULL REFERENCES wallpapers(id) ON DELETE CASCADE,
    sort_order INT NOT NULL DEFAULT 0,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (collection_id, wallpaper_id)
);