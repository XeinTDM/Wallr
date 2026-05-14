-- 1. Threaded replies, pinning, and hiding
ALTER TABLE wallpaper_comments ADD COLUMN parent_id TEXT REFERENCES wallpaper_comments(id) ON DELETE CASCADE;
ALTER TABLE wallpaper_comments ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE wallpaper_comments ADD COLUMN is_hidden BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE wallpaper_comments ADD COLUMN is_edited BOOLEAN NOT NULL DEFAULT false;

-- 2. Edit History
CREATE TABLE IF NOT EXISTS comment_edit_history (
    id TEXT PRIMARY KEY,
    comment_id TEXT NOT NULL REFERENCES wallpaper_comments(id) ON DELETE CASCADE,
    previous_content TEXT NOT NULL,
    edited_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Comment Reporting
CREATE TABLE IF NOT EXISTS reported_comments (
    id TEXT PRIMARY KEY,
    comment_id TEXT NOT NULL REFERENCES wallpaper_comments(id) ON DELETE CASCADE,
    reporter_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reporter_name TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, resolved, dismissed
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Per-wallpaper comment disable setting
ALTER TABLE wallpapers ADD COLUMN comments_disabled BOOLEAN NOT NULL DEFAULT false;
