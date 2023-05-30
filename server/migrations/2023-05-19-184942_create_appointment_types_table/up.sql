CREATE TABLE appointment_types (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,

    public BOOLEAN NOT NULL,
    user_id INT REFERENCES users,
    CHECK (public OR user_id IS NOT NULL),

    allow_multiple_students BOOLEAN NOT NULL DEFAULT false,

    -- Appointment timing
    duration INTERVAL NOT NULL,
    lockout INTERVAL NOT NULL DEFAULT '30 minutes',
    buffer INTERVAL NOT NULL DEFAULT '0 minutes',

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('appointment_types'::regclass);