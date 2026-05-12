-- Drop the generated column
ALTER TABLE wallpapers DROP COLUMN search_vector;

-- Add it back as a regular column
ALTER TABLE wallpapers ADD COLUMN search_vector tsvector;

-- Function to update a wallpaper's search vector
CREATE OR REPLACE FUNCTION update_wallpaper_search_vector()
RETURNS TRIGGER AS $$
DECLARE
    author_name TEXT;
BEGIN
    SELECT name INTO author_name FROM users WHERE id = NEW.author_id;
    
    NEW.search_vector :=
        setweight(to_tsvector('english', coalesce(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(author_name, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(NEW.tags::text, '')), 'C');
        
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger on wallpapers insert/update
CREATE TRIGGER trigger_update_wallpaper_search_vector
BEFORE INSERT OR UPDATE OF title, tags, author_id
ON wallpapers
FOR EACH ROW
EXECUTE FUNCTION update_wallpaper_search_vector();

-- Function to update all wallpapers for a user when their name changes
CREATE OR REPLACE FUNCTION update_user_wallpapers_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.name IS DISTINCT FROM NEW.name THEN
        UPDATE wallpapers
        SET search_vector = 
            setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
            setweight(to_tsvector('english', coalesce(NEW.name, '')), 'B') ||
            setweight(to_tsvector('english', coalesce(tags::text, '')), 'C')
        WHERE author_id = NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger on users update
CREATE TRIGGER trigger_update_user_wallpapers_search_vector
AFTER UPDATE OF name
ON users
FOR EACH ROW
EXECUTE FUNCTION update_user_wallpapers_search_vector();

-- Backfill existing data
UPDATE wallpapers w
SET search_vector = 
    setweight(to_tsvector('english', coalesce(w.title, '')), 'A') ||
    setweight(to_tsvector('english', coalesce(u.name, '')), 'B') ||
    setweight(to_tsvector('english', coalesce(w.tags::text, '')), 'C')
FROM users u
WHERE w.author_id = u.id;

-- Recreate index
CREATE INDEX IF NOT EXISTS wallpapers_search_idx ON wallpapers USING GIN (search_vector);
