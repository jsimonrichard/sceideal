CREATE DOMAIN EMAIL AS TEXT
  CHECK ( value ~ '^[\w\-\.]+@([\w\-]+\.)+[\w\-]{2,4}$' );
