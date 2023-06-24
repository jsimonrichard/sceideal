CREATE TABLE local_logins (
    user_id INT PRIMARY KEY REFERENCES users ON DELETE CASCADE,
    hash CHAR(60) NOT NULL,

    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('local_logins'::regclass);