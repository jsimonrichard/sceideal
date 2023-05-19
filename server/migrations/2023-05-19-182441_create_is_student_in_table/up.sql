CREATE TABLE is_student_in (
    user_id INT NOT NULL REFERENCES users,
    class_id INT NOT NULL REFERENCES class,

    joined_on TIMESTAMP NOT NULL DEFAULT current_timestamp,

    PRIMARY KEY (user_id, class_id)
);