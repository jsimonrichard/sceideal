CREATE TABLE is_member_of (
    user_id INT NOT NULL REFERENCES users ON DELETE CASCADE,
    group_id INT NOT NULL REFERENCES groups ON DELETE CASCADE,

    joined_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, group_id)
);