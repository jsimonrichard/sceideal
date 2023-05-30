CREATE TYPE LOCATION_TYPE AS ENUM ('address', 'link', 'other');

CREATE TABLE locations (
    id SERIAL,

    public BOOLEAN NOT NULL,
    user_id INT REFERENCES users,
    CHECK (public OR user_id IS NOT NULL),

    type LOCATION_TYPE NOT NULL,
    name TEXT NOT NULL,
    description TEXT,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (id, user_id)
);

SELECT diesel_manage_updated_at('locations'::regclass);