-- Remove orphaned records first to prevent constraint violations
DELETE FROM user_favorites WHERE user_id NOT IN (SELECT id FROM users);
DELETE FROM user_favorites WHERE wallpaper_id NOT IN (SELECT id FROM wallpapers);

-- Add missing foreign keys to user_favorites
ALTER TABLE user_favorites
ADD CONSTRAINT user_favorites_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE user_favorites
ADD CONSTRAINT user_favorites_wallpaper_id_fkey
FOREIGN KEY (wallpaper_id) REFERENCES wallpapers(id) ON DELETE CASCADE;
