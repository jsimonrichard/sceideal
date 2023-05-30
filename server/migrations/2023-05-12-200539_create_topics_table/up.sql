CREATE TABLE topics (
    id SERIAL PRIMARY KEY,

    name TEXT NOT NULL,
    description TEXT,

    public BOOLEAN NOT NULL,
    group_id INT REFERENCES groups,
    CHECK (public OR group_id IS NOT NULL),

    -- Lock out sign ups for specific topics sooner
    lockout INTERVAL,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('topics'::regclass);