CREATE TABLE class (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,

    instructor_email EMAIL,
    description TEXT,
    
    public BOOLEAN NOT NULL,

    -- Time stuff
    created_on  TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('class'::regclass);