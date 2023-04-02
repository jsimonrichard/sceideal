CREATE TABLE durations (
  user_id INTEGER REFERENCES users,
  duration INTERVAL,
  PRIMARY KEY (user_id, duration)
)
