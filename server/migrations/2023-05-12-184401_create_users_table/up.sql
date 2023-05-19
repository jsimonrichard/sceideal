CREATE TYPE PERMISSION_LEVEL AS ENUM ('student', 'teacher', 'admin');

CREATE TABLE users (
    -- Primary info
    id SERIAL PRIMARY KEY,
    email EMAIL UNIQUE NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    phone_number PHONE_NUMBER,

    -- User details
    fname TEXT NOT NULL,
    lname TEXT NOT NULL,
    bio TEXT,
    profile_image TEXT, -- path to an image

    permission_level PERMISSION_LEVEL NOT NULL DEFAULT 'student',

    -- Time stuff
    joined_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    last_login TIMESTAMP
);

SELECT diesel_manage_updated_at('users'::regclass);