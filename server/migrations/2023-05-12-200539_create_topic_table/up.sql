CREATE TABLE topics (
    user_id INT REFERENCES users NOT NULL,
    public BOOLEAN NOT NULL DEFAULT true,

    name TEXT NOT NULL,
    description TEXT,

    -- Lock out sign ups for specific topics sooner
    lockout INTERVAL,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, name)
);

SELECT diesel_manage_updated_at('topics'::regclass);