-- Create users table
CREATE TABLE Users (
  id SERIAL PRIMARY KEY,
  created TIMESTAMP NOT NULL default now(),
  email VARCHAR UNIQUE NOT NULL,
  password VARCHAR NOT NULL,
  is_admin BOOLEAN NOT NULL default false
);

-- Create index
CREATE INDEX bm_created_idx ON bookmarks (created);
