CREATE TYPE PROVISION AS ENUM ('auth', 'location', 'calendar');

CREATE TABLE oauth_connections (
    user_id INT REFERENCES users NOT NULL,
    
    provider TEXT NOT NULL,
    provides PROVISION NOT NULL,

    access_token TEXT NOT NULL,
    access_token_expires TIMESTAMP,
    refresh_token TEXT,
    refresh_token_expires TIMESTAMP,

    oid_subject TEXT CHECK (provides::text != 'auth' || oid_subject is not null),

    created_on TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, provider, provides)
);

SELECT diesel_manage_updated_at('oauth_connections'::regclass);

CREATE UNIQUE INDEX oid_subject_idx ON oauth_connections (provider, oid_subject);