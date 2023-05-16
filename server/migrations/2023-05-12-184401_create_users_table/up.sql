CREATE TABLE users (
    -- Primary info
    id SERIAL PRIMARY KEY,
    email EMAIL UNIQUE NOT NULL,
    phone_number PHONE_NUMBER,

    -- User details
    fname TEXT NOT NULL,
    lname TEXT NOT NULL,
    bio TEXT,
    profile_image TEXT, -- path to an image

    -- Time stuff
    joined_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    last_login TIMESTAMP
);

SELECT diesel_manage_updated_at('users'::regclass);