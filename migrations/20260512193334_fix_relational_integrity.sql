-- Drop the search_vector column that depends on `author`
ALTER TABLE wallpapers DROP COLUMN search_vector;

-- Add author_id column
ALTER TABLE wallpapers ADD COLUMN author_id TEXT REFERENCES users(id);

-- Backfill author_id by matching name
UPDATE wallpapers w
SET author_id = u.id
FROM users u
WHERE w.author = u.name;

-- Delete orphaned wallpapers
DELETE FROM wallpapers WHERE author_id IS NULL;

-- Make author_id NOT NULL
ALTER TABLE wallpapers ALTER COLUMN author_id SET NOT NULL;

-- Drop the old author column
ALTER TABLE wallpapers DROP COLUMN author;

-- Re-create search_vector without author dependency
ALTER TABLE wallpapers ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (
    setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
    setweight(to_tsvector('english', coalesce(tags::text, '')), 'C')
) STORED;

-- The old index was dropped with the column, recreate it
CREATE INDEX IF NOT EXISTS wallpapers_search_idx ON wallpapers USING GIN (search_vector);
