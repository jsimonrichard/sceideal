CREATE TABLE appointments (
    id SERIAL PRIMARY KEY,

    -- Provider
    user_id INT NOT NULL REFERENCES users ON DELETE CASCADE,

    --- Appointment details
    time TIMESTAMP WITH TIME ZONE NOT NULL,

    topic_id INT NOT NULL REFERENCES topics ON DELETE CASCADE,
    appointment_type_id INT NOT NULL REFERENCES appointment_types ON DELETE CASCADE,
    location_id INT NOT NULL,

    FOREIGN KEY (user_id, topic_id) REFERENCES can_teach ON DELETE CASCADE,
    FOREIGN KEY (user_id, appointment_type_id) REFERENCES provides_type ON DELETE CASCADE,
    FOREIGN KEY (user_id, location_id) REFERENCES locations ON DELETE CASCADE,

    canceled BOOLEAN NOT NULL DEFAULT false,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('appointments'::regclass);