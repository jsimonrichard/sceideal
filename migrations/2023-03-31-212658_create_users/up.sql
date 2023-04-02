CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL,
  hash CHAR(64) NOT NULL,
  email email NOT NULL
);
