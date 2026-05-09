CREATE INDEX IF NOT EXISTS wallpapers_tags_idx ON wallpapers USING GIN (tags);
CREATE INDEX IF NOT EXISTS wallpapers_colors_idx ON wallpapers USING GIN (primary_colors);
