CREATE TABLE uploads (
    id SERIAL PRIMARY KEY,
    appointment_id INT REFERENCES appointments,

    file_name TEXT NOT NULL,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('uploads'::regclass);