-- Create posts table
CREATE TABLE posts (
  id SERIAL PRIMARY KEY,
  title VARCHAR NOT NULL,
  url VARCHAR NOT NULL,
  body TEXT NOT NULL,
  tags jsonb NOT NULL DEFAULT '[]'::jsonb
)
