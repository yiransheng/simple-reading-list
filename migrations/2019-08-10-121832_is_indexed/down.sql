-- Drop column
ALTER TABLE bookmarks
DROP COLUMN toshi_index;

-- Drop index
DROP INDEX created_idx;
