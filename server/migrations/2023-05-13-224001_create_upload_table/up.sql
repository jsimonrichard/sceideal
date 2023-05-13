CREATE TABLE upload (
    id SERIAL PRIMARY KEY,
    appointment_id INT REFERENCES appointment,

    file_name TEXT NOT NULL,

    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_on TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('upload'::regclass);