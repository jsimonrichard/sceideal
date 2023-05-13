CREATE TABLE groups (
    id SERIAL PRIMARY KEY,
    name TEXT,
    
    -- Is mutable by the admin UI panel
    is_mutable BOOLEAN,

    -- Permissions
    can_sign_up_for_appointments BOOLEAN NOT NULL DEFAULT true,
    can_offer_appointments BOOLEAN NOT NULL DEFAULT false,
    can_access_bio BOOLEAN NOT NULL DEFAULT false,
    is_admin BOOLEAN NOT NULL DEFAULT false,

    -- Time stuff
    created_on  TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('groups'::regclass);