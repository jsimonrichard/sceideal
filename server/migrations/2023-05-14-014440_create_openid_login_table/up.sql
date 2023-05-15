CREATE TABLE openid_logins (
    user_id SERIAL REFERENCES users NOT NULL,
    provider TEXT NOT NULL,
    provides_calendar BOOLEAN NOT NULL DEFAULT false,

    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, provider)
);

SELECT diesel_manage_updated_at('openid_logins'::regclass);