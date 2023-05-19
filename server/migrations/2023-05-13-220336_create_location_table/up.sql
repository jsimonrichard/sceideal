CREATE TABLE locations (
    id SERIAL,

    -- Owner
    user_id INTEGER REFERENCES users,

    type TEXT, -- Helps web client display description correctly
    name TEXT NOT NULL,
    description TEXT,
    requirements TEXT,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (id, user_id)
);

SELECT diesel_manage_updated_at('locations'::regclass);