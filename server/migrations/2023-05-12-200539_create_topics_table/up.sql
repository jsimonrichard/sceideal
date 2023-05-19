CREATE TABLE topics (
    id SERIAL PRIMARY KEY,

    name TEXT NOT NULL,
    description TEXT,
    requirements TEXT,

    -- Lock out sign ups for specific topics sooner
    lockout INTERVAL,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('topics'::regclass);