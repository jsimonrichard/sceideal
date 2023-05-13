CREATE TABLE non_user (
    id SERIAL PRIMARY KEY,
    
    email TEXT NOT NULL,
    phone_number PHONE_NUMBER,
    fname TEXT NOT NULL,
    lname TEXT NOT NULL
);