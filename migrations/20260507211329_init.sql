CREATE TABLE IF NOT EXISTS wallpapers (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    image_url TEXT NOT NULL,
    thumbnail_url TEXT NOT NULL,
    tags JSONB NOT NULL,
    primary_colors JSONB NOT NULL,
    width INT NOT NULL,
    height INT NOT NULL,
    size_bytes BIGINT NOT NULL,
    likes INT DEFAULT 0,
    downloads INT DEFAULT 0,
    search_vector tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(author, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(tags::text, '')), 'C')
    ) STORED
);

CREATE INDEX IF NOT EXISTS wallpapers_search_idx ON wallpapers USING GIN (search_vector);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    pfp_url TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    banner_url TEXT,
    token_version INT NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    item_count INT NOT NULL DEFAULT 0,
    cover_url TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_favorites (
    user_id TEXT NOT NULL,
    wallpaper_id TEXT NOT NULL,
    PRIMARY KEY (user_id, wallpaper_id)
);
