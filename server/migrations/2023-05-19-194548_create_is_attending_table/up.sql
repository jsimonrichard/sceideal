CREATE TABLE is_attending (
    id SERIAL PRIMARY KEY,

    appointment_id INT NOT NULL REFERENCES appointments ON DELETE CASCADE,

    notes TEXT,
    
    -- Client details
    user_id INT REFERENCES users ON DELETE CASCADE,
    non_user_id INT REFERENCES non_users ON DELETE CASCADE,
    CHECK (
        (user_id IS NULL) !=
        (non_user_id IS NULL)
    ),

    canceled BOOLEAN NOT NULL DEFAULT false,
    
    -- Time stuff
    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('is_attending'::regclass);