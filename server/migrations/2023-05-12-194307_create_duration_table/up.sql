CREATE TABLE durations (
    user_id INT REFERENCES users NOT NULL,
    public BOOLEAN NOT NULL DEFAULT true,

    -- Appointment timing
    duration_time INTERVAL NOT NULL,
    lockout INTERVAL NOT NULL DEFAULT '30 minutes',
    buffer INTERVAL NOT NULL DEFAULT '0 minutes',

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, duration_time)
);

SELECT diesel_manage_updated_at('durations'::regclass);