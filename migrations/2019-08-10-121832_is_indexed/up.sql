-- track is indexed 
ALTER TABLE bookmarks 
ADD COLUMN is_indexed BOOLEAN NOT NULL DEFAULT false;

-- index created
CREATE INDEX created_idx ON bookmarks (created);
