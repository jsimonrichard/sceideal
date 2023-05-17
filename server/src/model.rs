use chrono::NaiveDateTime;
use diesel::{data_types::PgInterval, *};
use serde::Deserialize;

use crate::schema::*;

#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub phone_number: Option<String>,
    pub fname: String,
    pub lname: String,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub joined_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_login: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub phone_number: Option<&'a str>,
    pub fname: &'a str,
    pub lname: &'a str,
    pub bio: Option<&'a str>,
    pub profile_image: Option<&'a str>,
}

#[derive(
    Clone, Debug, PartialEq, Eq, Deserialize, Queryable, Identifiable, Selectable, Associations,
)]
#[diesel(belongs_to(User))]
#[diesel(primary_key(user_id))]
pub struct LocalLogin {
    pub user_id: i32,
    pub hash: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=local_logins)]
pub struct NewLocalLogin<'a> {
    pub user_id: i32,
    pub hash: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Queryable, Identifiable, Selectable)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub is_mutable: bool,
    pub can_sign_up_for_appointments: bool,
    pub can_offer_appointments: bool,
    pub can_access_bio: bool,
    pub is_admin: bool,
    pub created_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=groups)]
pub struct NewGroup<'a> {
    pub name: &'a str,
    pub can_sign_up_for_appointments: bool,
    pub can_offer_appointments: bool,
    pub can_access_bio: bool,
    pub is_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Queryable, Selectable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Group))]
pub struct GroupMembership {
    pub user_id: i32,
    pub group_id: i32,
    pub joined_on: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=group_memberships)]
pub struct NewGroupMembership {
    pub user_id: i32,
    pub group_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Queryable)]
pub struct NonUser {
    pub id: i32,
    pub email: String,
    pub phone_number: Option<String>,
    pub fname: String,
    pub lname: String,
}

#[derive(Insertable)]
#[diesel(table_name=non_users)]
pub struct NewNonUser<'a> {
    pub email: &'a str,
    pub phone_number: Option<&'a str>,
    pub fname: &'a str,
    pub lname: &'a str,
}

#[derive(Debug, PartialEq, Eq, Queryable, Selectable, Associations)]
#[diesel(belongs_to(User))]
pub struct Duration {
    pub user_id: i32,
    pub public: bool,
    pub duration_time: PgInterval,
    pub lockout: PgInterval,
    pub buffer: PgInterval,
    pub created_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=durations)]
pub struct NewDuration<'a> {
    pub user_id: i32,
    pub public: bool,
    pub duration_time: &'a PgInterval,
    pub lockout: &'a PgInterval,
    pub buffer: &'a PgInterval,
}

#[derive(Clone, Debug, PartialEq, Eq, Queryable)]
pub struct Topic {
    pub user_id: i32,
    pub public: bool,
    pub name: String,
    pub description: Option<String>,
    pub lockout: Option<PgInterval>,
    pub created_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=topics)]
pub struct NewTopic<'a> {
    pub user_id: i32,
    pub public: bool,
    pub name: String,
    pub description: Option<&'a str>,
    pub lockout: Option<&'a PgInterval>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Queryable)]
pub struct Location {
    pub user_id: i32,
    pub public: bool,
    pub name: String,
    pub description: Option<String>,
    pub type_: Option<String>,
    pub created_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=locations)]
pub struct NewLocation<'a> {
    pub user_id: i32,
    pub public: bool,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub type_: Option<&'a str>,
}

// #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Queryable)]
// pub struct Appointment {
//     pub id: i32,
//     pub ...
// }

#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User), table_name = oauth_logins, primary_key(provider, associated_email))]
pub struct OAuthLogin {
    pub user_id: i32,
    pub provider: String,
    pub associated_email: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = oauth_logins)]
pub struct NewOAuthLogin<'a> {
    pub user_id: i32,
    pub provider: &'a str,
    pub associated_email: &'a str,
}
