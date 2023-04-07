CREATE TABLE appointments (
  id SERIAL PRIMARY KEY,
  
  user_id INTEGER NOT NULL,
  appointment_type_id INTEGER NOT NULL,
  location VARCHAR,

  time TIMESTAMP,
  duration INTERVAL,

  client_fname VARCHAR NOT NULL,
  client_lname VARCHAR NOT NULL,
  client_email email NOT NULL,
  client_phone phone_number,

  details VARCHAR,

  FOREIGN KEY(user_id, appointment_type_id) REFERENCES provides,
  FOREIGN KEY(user_id, location) REFERENCES locations(user_id, name)
);
