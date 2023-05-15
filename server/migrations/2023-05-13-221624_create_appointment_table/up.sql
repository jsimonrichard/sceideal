CREATE TABLE appointments (
    id SERIAL PRIMARY KEY,

    -- Appointment details
    provider_id INT REFERENCES users NOT NULL,
    time TIMESTAMP WITH TIME ZONE NOT NULL,
    topic TEXT NOT NULL,
    location_name TEXT NOT NULL,
    duration INTERVAL NOT NULL,

    FOREIGN KEY (provider_id, topic) REFERENCES topics,
    FOREIGN KEY (provider_id, location_name) REFERENCES locations,
    FOREIGN KEY (provider_id, duration) REFERENCES durations,

    notes TEXT,

    -- Client details
    client_user_id INT REFERENCES users,
    client_non_user_id INT REFERENCES non_users,
    CHECK (
        (client_user_id IS NULL) !=
        (client_non_user_id IS NULL)
    ),

    canceled BOOLEAN NOT NULL DEFAULT false,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('appointments'::regclass);