CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE wallpapers ADD COLUMN embedding vector(512);
