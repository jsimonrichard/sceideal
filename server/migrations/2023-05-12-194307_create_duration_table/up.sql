CREATE TABLE duration (
    user_id INT REFERENCES users NOT NULL,
    public BOOLEAN NOT NULL DEFAULT true,

    -- Appointment timing
    duration INTERVAL NOT NULL,
    lockout INTERVAL NOT NULL DEFAULT '30 minutes',
    buffer INTERVAL NOT NULL DEFAULT '0 minutes',

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, duration)
);

SELECT diesel_manage_updated_at('duration'::regclass);