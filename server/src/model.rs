use chrono::NaiveDateTime;
use diesel::{data_types::PgInterval, *};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::schema::*;

#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::PermissionLevel"]
pub enum PermissionLevel {
    Student,
    Teacher,
    Admin,
}

#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub fname: String,
    pub lname: String,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub permission_level: PermissionLevel,
    pub joined_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_login: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub email_verified: bool,
    pub phone_number: Option<&'a str>,
    pub fname: &'a str,
    pub lname: &'a str,
    pub bio: Option<&'a str>,
    pub profile_image: Option<&'a str>,
    pub permission_level: PermissionLevel,
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

#[derive(Clone, Debug, PartialEq, Eq, Queryable)]
pub struct Topic {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub lockout: Option<PgInterval>,
    pub created_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=topics)]
pub struct NewTopic<'a> {
    pub id: i32,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub requirements: Option<&'a str>,
    pub lockout: Option<&'a PgInterval>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(belongs_to(User), primary_key(id, user_id))]
pub struct Location {
    pub id: i32,
    pub user_id: i32,
    pub type_: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub created_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=locations)]
pub struct NewLocation<'a> {
    pub id: i32,
    pub user_id: i32,
    pub type_: Option<&'a str>,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub requirements: Option<&'a str>,
}

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
