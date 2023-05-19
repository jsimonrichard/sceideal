CREATE TABLE uploads (
    id SERIAL PRIMARY KEY,
    is_attending_id INT REFERENCES is_attending,

    file_name TEXT NOT NULL,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('uploads'::regclass);