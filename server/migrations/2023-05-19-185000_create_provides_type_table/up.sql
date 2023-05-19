CREATE TABLE provides_type (
    user_id INT REFERENCES users,
    appointment_type_id INT REFERENCES appointment_types,

    PRIMARY KEY (user_id, appointment_type_id)
);