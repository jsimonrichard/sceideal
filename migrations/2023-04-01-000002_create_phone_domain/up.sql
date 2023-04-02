CREATE DOMAIN phone_number AS VARCHAR(10) CHECK(VALUE ~ '^[\+]?[(]?[0-9]{3}[)]?[-\s\.]?[0-9]{3}[-\s\.]?[0-9]{4,6}$');
