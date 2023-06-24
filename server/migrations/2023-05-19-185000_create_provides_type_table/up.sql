CREATE TABLE provides_type (
    user_id INT REFERENCES users ON DELETE CASCADE,
    appointment_type_id INT REFERENCES appointment_types ON DELETE CASCADE,

    PRIMARY KEY (user_id, appointment_type_id)
);