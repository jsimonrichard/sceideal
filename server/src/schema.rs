// @generated automatically by Diesel CLI.

diesel::table! {
    appointments (id) {
        id -> Int4,
        provider_id -> Int4,
        time -> Timestamptz,
        topic -> Text,
        location_name -> Text,
        duration -> Interval,
        notes -> Nullable<Text>,
        client_user_id -> Nullable<Int4>,
        client_non_user_id -> Nullable<Int4>,
        canceled -> Bool,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    durations (user_id, duration_time) {
        user_id -> Int4,
        public -> Bool,
        duration_time -> Interval,
        lockout -> Interval,
        buffer -> Interval,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    group_memberships (user_id, group_id) {
        user_id -> Int4,
        group_id -> Int4,
        joined_on -> Timestamp,
    }
}

diesel::table! {
    groups (id) {
        id -> Int4,
        name -> Text,
        is_mutable -> Bool,
        can_sign_up_for_appointments -> Bool,
        can_offer_appointments -> Bool,
        can_access_bio -> Bool,
        is_admin -> Bool,
        created_on -> Timestamp,
        updated_at -> Timestamp,
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
    locations (user_id, name) {
        user_id -> Int4,
        public -> Bool,
        name -> Text,
        description -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Nullable<Text>,
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
    topics (user_id, name) {
        user_id -> Int4,
        public -> Bool,
        name -> Text,
        description -> Nullable<Text>,
        lockout -> Nullable<Interval>,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    uploads (id) {
        id -> Int4,
        appointment_id -> Nullable<Int4>,
        file_name -> Text,
        created_on -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Text,
        phone_number -> Nullable<Text>,
        fname -> Text,
        lname -> Text,
        bio -> Nullable<Text>,
        profile_image -> Nullable<Text>,
        joined_on -> Timestamp,
        updated_at -> Timestamp,
        last_login -> Nullable<Timestamp>,
    }
}

diesel::joinable!(appointments -> non_users (client_non_user_id));
diesel::joinable!(durations -> users (user_id));
diesel::joinable!(group_memberships -> groups (group_id));
diesel::joinable!(group_memberships -> users (user_id));
diesel::joinable!(local_logins -> users (user_id));
diesel::joinable!(locations -> users (user_id));
diesel::joinable!(oauth_logins -> users (user_id));
diesel::joinable!(topics -> users (user_id));
diesel::joinable!(uploads -> appointments (appointment_id));

diesel::allow_tables_to_appear_in_same_query!(
    appointments,
    durations,
    group_memberships,
    groups,
    local_logins,
    locations,
    non_users,
    oauth_logins,
    topics,
    uploads,
    users,
);
