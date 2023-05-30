CREATE TABLE is_member_of (
    user_id INT NOT NULL REFERENCES users,
    group_id INT NOT NULL REFERENCES groups,

    joined_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, group_id)
);