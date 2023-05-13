CREATE TABLE group_membership (
    user_id INT REFERENCES users,
    group_id INT REFERENCES groups,

    joined_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, group_id)
);