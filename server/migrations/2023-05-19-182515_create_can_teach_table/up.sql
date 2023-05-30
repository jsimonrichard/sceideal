CREATE TABLE can_teach (
    user_id INT NOT NULL REFERENCES users,
    topic_id INT NOT NULL REFERENCES topics,

    since TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, topic_id)
);