CREATE TABLE is_member_of (
    user_id INT NOT NULL REFERENCES users ON DELETE CASCADE,
    group_id INT NOT NULL REFERENCES groups ON DELETE CASCADE,

    assigned_teacher INT REFERENCES users ON DELETE SET NULL,

    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    joined_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, group_id)
);

SELECT diesel_manage_updated_at('is_member_of'::regclass);