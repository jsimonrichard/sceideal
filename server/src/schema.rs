// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "permission_level"))]
    pub struct PermissionLevel;
}

diesel::table! {
    appointment_types (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        requirements -> Nullable<Text>,
        public -> Bool,
        user_id -> Nullable<Int4>,
        allow_multiple_students -> Bool,
        duration -> Interval,
        lockout -> Interval,
        buffer -> Interval,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    appointments (id) {
        id -> Uuid,
        user_id -> Int4,
        time -> Timestamptz,
        topic_id -> Int4,
        appointment_type_id -> Int4,
        location_id -> Int4,
        canceled -> Bool,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    can_teach_in (user_id, class_id) {
        user_id -> Int4,
        class_id -> Int4,
        started_on -> Timestamp,
    }
}

diesel::table! {
    class (id) {
        id -> Int4,
        name -> Text,
        instructor_email -> Nullable<Text>,
        description -> Nullable<Text>,
        public -> Bool,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    is_attending (id) {
        id -> Int4,
        appointment_id -> Uuid,
        user_id -> Nullable<Int4>,
        non_user_id -> Nullable<Int4>,
        canceled -> Bool,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    is_student_in (user_id, class_id) {
        user_id -> Int4,
        class_id -> Int4,
        joined_on -> Timestamp,
    }
}

diesel::table! {
    local_logins (user_id) {
        user_id -> Int4,
        hash -> Bpchar,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    locations (id, user_id) {
        id -> Int4,
        user_id -> Int4,
        #[sql_name = "type"]
        type_ -> Nullable<Text>,
        name -> Text,
        description -> Nullable<Text>,
        requirements -> Nullable<Text>,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    non_users (id) {
        id -> Int4,
        email -> Text,
        phone_number -> Nullable<Text>,
        fname -> Text,
        lname -> Text,
    }
}

diesel::table! {
    oauth_logins (provider, associated_email) {
        user_id -> Int4,
        provider -> Text,
        associated_email -> Text,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    provides_type (user_id, appointment_type_id) {
        user_id -> Int4,
        appointment_type_id -> Int4,
    }
}

diesel::table! {
    teaches (user_id, topic_id) {
        user_id -> Int4,
        topic_id -> Int4,
        since -> Timestamp,
    }
}

diesel::table! {
    topics (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        requirements -> Nullable<Text>,
        lockout -> Nullable<Interval>,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    uploads (id) {
        id -> Int4,
        is_attending_id -> Nullable<Int4>,
        file_name -> Text,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PermissionLevel;

    users (id) {
        id -> Int4,
        email -> Text,
        email_verified -> Bool,
        phone_number -> Nullable<Text>,
        fname -> Text,
        lname -> Text,
        bio -> Nullable<Text>,
        profile_image -> Nullable<Text>,
        permission_level -> PermissionLevel,
        joined_on -> Timestamp,
        updated_at -> Timestamp,
        last_login -> Nullable<Timestamp>,
    }
}

diesel::joinable!(appointment_types -> users (user_id));
diesel::joinable!(appointments -> appointment_types (appointment_type_id));
diesel::joinable!(appointments -> topics (topic_id));
diesel::joinable!(appointments -> users (user_id));
diesel::joinable!(can_teach_in -> class (class_id));
diesel::joinable!(can_teach_in -> users (user_id));
diesel::joinable!(is_attending -> appointments (appointment_id));
diesel::joinable!(is_attending -> non_users (non_user_id));
diesel::joinable!(is_attending -> users (user_id));
diesel::joinable!(is_student_in -> class (class_id));
diesel::joinable!(is_student_in -> users (user_id));
diesel::joinable!(local_logins -> users (user_id));
diesel::joinable!(locations -> users (user_id));
diesel::joinable!(oauth_logins -> users (user_id));
diesel::joinable!(provides_type -> appointment_types (appointment_type_id));
diesel::joinable!(provides_type -> users (user_id));
diesel::joinable!(teaches -> topics (topic_id));
diesel::joinable!(teaches -> users (user_id));
diesel::joinable!(uploads -> is_attending (is_attending_id));

diesel::allow_tables_to_appear_in_same_query!(
    appointment_types,
    appointments,
    can_teach_in,
    class,
    is_attending,
    is_student_in,
    local_logins,
    locations,
    non_users,
    oauth_logins,
    provides_type,
    teaches,
    topics,
    uploads,
    users,
);
