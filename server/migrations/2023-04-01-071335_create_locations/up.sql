CREATE TABLE locations (
  user_id INTEGER REFERENCES users,
  name VARCHAR NOT NULL,
  link VARCHAR,
  PRIMARY KEY(user_id, name)
);
