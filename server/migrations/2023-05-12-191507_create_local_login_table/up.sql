CREATE TABLE local_login (
    id INT REFERENCES users PRIMARY KEY,
    hash CHAR(60) NOT NULL,
    last_login TIMESTAMP
);