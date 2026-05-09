CREATE MATERIALIZED VIEW IF NOT EXISTS trending_tags AS
SELECT tag, count(*) as count
FROM wallpapers, jsonb_array_elements_text(tags) as tag
WHERE tag != 'misc'
GROUP BY tag;

CREATE UNIQUE INDEX IF NOT EXISTS trending_tags_tag_idx ON trending_tags(tag);
CREATE INDEX IF NOT EXISTS trending_tags_count_idx ON trending_tags(count DESC);
