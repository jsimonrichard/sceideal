CREATE TABLE can_teach_in (
    user_id INT NOT NULL REFERENCES users,
    class_id INT NOT NULL REFERENCES class,

    started_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, class_id)
);