-- Create posts table
CREATE TABLE Bookmarks (
  id SERIAL PRIMARY KEY,
  created TIMESTAMP NOT NULL default now(),
  title VARCHAR NOT NULL,
  url VARCHAR NOT NULL,
  body TEXT NOT NULL,
  tags jsonb NOT NULL DEFAULT '{"tags": []}'::jsonb
)
