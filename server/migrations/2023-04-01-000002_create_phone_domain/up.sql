CREATE DOMAIN PHONE_NUMBER AS TEXT CHECK(VALUE ~ '^[\+]?[(]?[0-9]{3}[)]?[-\s\.]?[0-9]{3}[-\s\.]?[0-9]{4,6}$');
