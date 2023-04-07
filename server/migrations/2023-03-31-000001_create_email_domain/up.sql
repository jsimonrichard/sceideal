CREATE DOMAIN email AS VARCHAR
  CHECK ( value ~ '^[\w\-\.]+@([\w\-]+\.)+[\w\-]{2,4}$' );
