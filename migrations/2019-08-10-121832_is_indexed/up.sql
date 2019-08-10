-- track is indexed 
ALTER TABLE bookmarks 
ADD COLUMN toshi_index TEXT;

-- index created
CREATE INDEX created_idx ON bookmarks (created);
