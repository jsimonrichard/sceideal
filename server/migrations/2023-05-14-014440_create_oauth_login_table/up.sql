CREATE TABLE oauth_logins (
    user_id SERIAL REFERENCES users NOT NULL,
    provider TEXT NOT NULL,
    associated_email TEXT NOT NULL,

    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (provider, associated_email)
);

SELECT diesel_manage_updated_at('oauth_logins'::regclass);