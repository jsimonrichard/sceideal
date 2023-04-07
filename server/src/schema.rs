// @generated automatically by Diesel CLI.

diesel::table! {
    appointment_type (id) {
        id -> Int4,
        #[sql_name = "type"]
        type_ -> Varchar,
    }
}

diesel::table! {
    appointments (id) {
        id -> Int4,
        user_id -> Int4,
        appointment_type_id -> Int4,
        location -> Nullable<Varchar>,
        time -> Nullable<Timestamp>,
        duration -> Nullable<Interval>,
        client_fname -> Varchar,
        client_lname -> Varchar,
        client_email -> Varchar,
        client_phone -> Nullable<Varchar>,
        details -> Nullable<Varchar>,
    }
}

diesel::table! {
    locations (user_id, name) {
        user_id -> Int4,
        name -> Varchar,
        link -> Nullable<Varchar>,
    }
}

diesel::table! {
    provides (user_id, appointment_type_id) {
        user_id -> Int4,
        appointment_type_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        hash -> Bpchar,
        email -> Varchar,
        joined_on -> Timestamp,
    }
}

diesel::joinable!(locations -> users (user_id));
diesel::joinable!(provides -> appointment_type (appointment_type_id));
diesel::joinable!(provides -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    appointment_type,
    appointments,
    locations,
    provides,
    users,
);