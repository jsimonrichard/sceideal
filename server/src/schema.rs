// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "location_type"))]
    pub struct LocationType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "permission_level"))]
    pub struct PermissionLevel;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "provision"))]
    pub struct Provision;
}

diesel::table! {
    appointment_types (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
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
        id -> Int4,
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
    can_teach (user_id, topic_id) {
        user_id -> Int4,
        topic_id -> Int4,
        since -> Timestamp,
    }
}

diesel::table! {
    groups (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        public -> Bool,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    is_attending (id) {
        id -> Int4,
        appointment_id -> Int4,
        notes -> Nullable<Text>,
        user_id -> Nullable<Int4>,
        non_user_id -> Nullable<Int4>,
        canceled -> Bool,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    is_member_of (user_id, group_id) {
        user_id -> Int4,
        group_id -> Int4,
        assigned_teacher -> Nullable<Int4>,
        updated_at -> Timestamp,
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
    use diesel::sql_types::*;
    use super::sql_types::LocationType;

    locations (id, user_id) {
        id -> Int4,
        public -> Bool,
        user_id -> Int4,
        #[sql_name = "type"]
        type_ -> LocationType,
        name -> Text,
        description -> Nullable<Text>,
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
    use diesel::sql_types::*;
    use super::sql_types::Provision;

    oauth_connections (user_id, provider, provides) {
        user_id -> Int4,
        provider -> Text,
        provides -> Provision,
        access_token -> Text,
        access_token_expires -> Nullable<Timestamp>,
        refresh_token -> Nullable<Text>,
        refresh_token_expires -> Nullable<Timestamp>,
        oid_subject -> Nullable<Text>,
        created_on -> Timestamp,
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
    topics (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        public -> Bool,
        group_id -> Nullable<Int4>,
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
diesel::joinable!(can_teach -> topics (topic_id));
diesel::joinable!(can_teach -> users (user_id));
diesel::joinable!(is_attending -> appointments (appointment_id));
diesel::joinable!(is_attending -> non_users (non_user_id));
diesel::joinable!(is_attending -> users (user_id));
diesel::joinable!(is_member_of -> groups (group_id));
diesel::joinable!(local_logins -> users (user_id));
diesel::joinable!(locations -> users (user_id));
diesel::joinable!(oauth_connections -> users (user_id));
diesel::joinable!(provides_type -> appointment_types (appointment_type_id));
diesel::joinable!(provides_type -> users (user_id));
diesel::joinable!(topics -> groups (group_id));
diesel::joinable!(uploads -> is_attending (is_attending_id));

diesel::allow_tables_to_appear_in_same_query!(
    appointment_types,
    appointments,
    can_teach,
    groups,
    is_attending,
    is_member_of,
    local_logins,
    locations,
    non_users,
    oauth_connections,
    provides_type,
    topics,
    uploads,
    users,
);
