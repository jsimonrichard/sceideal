CREATE TABLE locations (
    user_id INTEGER REFERENCES users,
    public BOOLEAN NOT NULL DEFAULT true,

    name TEXT NOT NULL,
    description TEXT,
    type TEXT, -- Helps client display description correctly

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY(user_id, name)
);

SELECT diesel_manage_updated_at('locations'::regclass);