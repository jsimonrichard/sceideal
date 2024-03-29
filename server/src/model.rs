use chrono::NaiveDateTime;
use diesel::{data_types::PgInterval, *};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typeshare::typeshare;

use crate::schema::*;
use crate::utils::some_option;

#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
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
    pub permission_level: Option<PermissionLevel>,
}

#[typeshare]
#[derive(AsChangeset, Deserialize)]
#[skip_serializing_none]
#[diesel(table_name = users)]
pub struct AdminUpdateUser {
    pub email_verified: Option<bool>,
    #[serde(default, deserialize_with = "some_option")]
    pub phone_number: Option<Option<String>>,
    pub fname: Option<String>,
    pub lname: Option<String>,
    #[serde(default, deserialize_with = "some_option")]
    pub bio: Option<Option<String>>,
    pub profile_image: Option<String>,
    pub permission_level: Option<PermissionLevel>,
}

#[typeshare]
#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = users)]
#[skip_serializing_none]
pub struct UpdateUser {
    pub phone_number: Option<Option<String>>,
    pub fname: Option<String>,
    pub lname: Option<String>,
    pub bio: Option<Option<String>>,
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
    pub lockout: Option<&'a PgInterval>,
}

#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::LocationType"]
pub enum LocationType {
    Address,
    Link,
    Other,
}

#[typeshare]
#[derive(
    Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Queryable, Selectable, Identifiable,
)]
#[diesel(belongs_to(User), primary_key(id, user_id))]
pub struct Location {
    pub id: i32,
    pub public: bool,
    pub user_id: i32,
    pub type_: LocationType,
    pub name: String,
    pub description: Option<String>,
    #[typeshare(serialized_as = "String")]
    pub created_on: NaiveDateTime,
    #[typeshare(serialized_as = "String")]
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name=locations)]
pub struct NewLocation<'a> {
    pub user_id: i32,
    pub type_: LocationType,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

#[typeshare]
#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = locations)]
pub struct UpdateLocation {
    pub type_: Option<LocationType>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Hash, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::Provision"]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvision {
    Auth,
    Location,
    Calendar,
}

#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Associations, Selectable)]
#[diesel(belongs_to(User), table_name = oauth_connections, primary_key(user_id, provider, provides))]
pub struct OAuthConnection {
    pub user_id: i32,
    pub provider: String,
    pub provides: OAuthProvision,
    pub access_token: String,
    pub access_token_expires: Option<NaiveDateTime>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires: Option<NaiveDateTime>,
    pub oid_subject: Option<String>,
    pub created_on: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = oauth_connections)]
pub struct NewOAuthConnection<'a> {
    pub user_id: i32,
    pub provider: &'a str,
    pub provides: &'a OAuthProvision,
    pub access_token: &'a str,
    pub access_token_expires: Option<&'a NaiveDateTime>,
    pub refresh_token: Option<&'a str>,
    pub refresh_token_expires: Option<&'a NaiveDateTime>,
    pub oid_subject: Option<&'a str>,
}

#[typeshare]
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Serialize)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub public: bool,
    #[typeshare(serialized_as = "String")]
    pub created_on: NaiveDateTime,
    #[typeshare(serialized_as = "String")]
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = groups)]
pub struct NewGroup<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub public: bool,
}

#[typeshare]
#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = groups)]
pub struct UpdateGroup {
    pub name: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
}

#[typeshare]
#[derive(Deserialize, Insertable)]
#[diesel(table_name = groups)]
pub struct CreateGroup {
    pub name: String,
    pub description: Option<String>,
    pub public: bool,
}

#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Associations, Serialize)]
#[diesel(
    table_name = is_member_of,
    belongs_to(User),
    belongs_to(Group),
    primary_key(user_id, group_id)
)]
pub struct IsMemberOf {
    user_id: i32,
    group_id: i32,
    assigned_teacher: Option<i32>,
    updated_at: NaiveDateTime,
    joined_on: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = is_member_of)]
pub struct CreateIsMemberOf {
    pub user_id: i32,
    pub group_id: i32,
    pub assigned_teacher: Option<i32>,
}

#[typeshare]
#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = is_member_of)]
pub struct UpdateIsMemberOf {
    pub assigned_teacher: Option<i32>,
}
