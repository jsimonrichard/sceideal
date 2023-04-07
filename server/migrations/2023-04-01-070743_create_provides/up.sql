CREATE TABLE provides (
  user_id INTEGER REFERENCES users NOT NULL,
  appointment_type_id INTEGER REFERENCES appointment_type NOT NULL,
  PRIMARY KEY(user_id, appointment_type_id)
);
