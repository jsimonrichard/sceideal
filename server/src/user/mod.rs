use std::collections::HashMap;

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::NaiveDateTime;
use diesel::{delete, insert_into, prelude::*, update};
use diesel_async::RunQueryDsl;
use futures::TryStreamExt;
use itertools::izip;
use openidconnect::{url::Url, LogoutRequest, PostLogoutRedirectUrl};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

mod local;
pub mod openid_connect;
pub mod session;

use crate::{
    config::StatefulConfig,
    http_error::HttpError,
    model::{
        AdminUpdateUser, CreateIsMemberOf, Group, IsMemberOf, LocalLogin, NewLocalLogin, NewUser,
        OAuthConnection, OAuthProvision, PermissionLevel, UpdateIsMemberOf, UpdateUser, User,
    },
    AppState, PgConn, PgPool, SessionStore,
};

use self::openid_connect::OpenIdClients;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/local", local::router())
        .nest("/openid", openid_connect::router())
        .route("/", get(get_me).post(add_user).put(update_me))
        .route("/groups", get(get_my_groups))
        .route("/groups/:group_id", get(get_my_group_details))
        .route("/logout", post(logout))
        .route("/:id", get(get_user))
        .route("/a", get(get_all_users))
        .route(
            "/a/:id",
            get(get_user_admin).put(update_user).delete(delete_user),
        )
        .route("/a/:id/groups", get(get_user_groups))
        .route(
            "/a/:id/groups/:group_id",
            post(add_user_to_group)
                .delete(remove_user_from_group)
                .put(update_group_membership),
        )
}

impl User {
    pub async fn find(
        name_query: &str,
        connection: &mut PgConn<'_>,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use crate::schema::users::dsl::*;
        users
            .filter(email.eq(&name_query))
            .first(connection)
            .await
            .optional()
    }

    pub async fn get(
        id_: i32,
        connection: &mut PgConn<'_>,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use crate::schema::users::dsl::*;
        users.find(id_).get_result(connection).await.optional()
    }

    pub async fn from_jar(
        jar: &CookieJar,
        session: &SessionStore,
        connection: &mut PgConn<'_>,
    ) -> Result<Option<Self>, diesel::result::Error> {
        let user_id_ = if let Some(session_data) = session.get(jar).await {
            session_data.user_id
        } else {
            return Ok(None);
        };

        User::get(user_id_, connection).await
    }

    pub async fn get_user_data(
        &self,
        conn: &mut PgConn<'_>,
    ) -> Result<UserData, diesel::result::Error> {
        let local_login_opt: Option<LocalLoginData> = LocalLogin::belonging_to(&self)
            .first::<LocalLogin>(conn)
            .await
            .optional()?
            .map(|l| l.into());
        let oauth_providers = OAuthConnection::belonging_to(&self)
            .load_stream::<OAuthConnection>(conn)
            .await?
            .try_fold(
                HashMap::<OAuthProvision, Vec<OAuthConnectionData>>::new(),
                |mut acc, item| {
                    acc.entry(item.provides).or_default().push(item.into());
                    futures::future::ready(Ok(acc))
                },
            )
            .await?;
        Ok(UserData {
            id: self.id,
            email: self.email.clone(),
            email_verified: self.email_verified,
            phone_number: self.phone_number.clone(),
            fname: self.fname.clone(),
            lname: self.lname.clone(),
            bio: self.bio.clone(),
            profile_image: self.profile_image.clone(),
            permission_level: self.permission_level,
            joined_on: self.joined_on,
            updated_at: self.updated_at,
            last_login: self.last_login,
            local_login: local_login_opt,
            oauth_providers,
        })
    }

    pub fn get_public_user_data(&self) -> PublicUserData {
        PublicUserData {
            id: self.id,
            email: self.email.clone(),
            phone_number: self.phone_number.clone(),
            fname: self.fname.clone(),
            lname: self.lname.clone(),
            bio: self.bio.clone(),
            profile_image: self.profile_image.clone(),
            permission_level: self.permission_level,
            joined_on: self.joined_on,
        }
    }
}

pub struct UserFromParts {
    pub user: User,
    pub jar: CookieJar,
}

#[async_trait]
impl FromRequestParts<AppState> for UserFromParts {
    type Rejection = HttpError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Get cookie value
        let mut jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        // Get the user
        let conn = &mut state.pool.get().await?;
        // Gets user and updates session TTL
        let user = User::from_jar(&jar, &state.session_store, conn)
            .await?
            .ok_or(HttpError::WithCode {
                code: StatusCode::FORBIDDEN,
                msg: "No user found",
            })?;

        // Update cookie TTL
        jar = state.session_store.reup(jar).await;

        Ok(UserFromParts { user, jar })
    }
}

pub struct TeacherFromParts(pub UserFromParts);

#[async_trait]
impl FromRequestParts<AppState> for TeacherFromParts {
    type Rejection = HttpError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_from_parts = UserFromParts::from_request_parts(parts, state).await?;
        match user_from_parts.user.permission_level {
            PermissionLevel::Teacher | PermissionLevel::Admin => {
                Ok(TeacherFromParts(user_from_parts))
            }
            _ => Err(HttpError::forbidden("insufficient permissions")),
        }
    }
}

pub struct AdminFromParts(pub UserFromParts);

#[async_trait]
impl FromRequestParts<AppState> for AdminFromParts {
    type Rejection = HttpError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_from_parts = UserFromParts::from_request_parts(parts, state).await?;
        match user_from_parts.user.permission_level {
            PermissionLevel::Admin => Ok(AdminFromParts(user_from_parts)),
            _ => Err(HttpError::forbidden("insufficient permissions")),
        }
    }
}

#[typeshare]
#[derive(Serialize)]
pub struct UserData {
    pub id: i32,
    pub email: String,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub fname: String,
    pub lname: String,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub permission_level: PermissionLevel,
    #[typeshare(serialized_as = "String")]
    pub joined_on: NaiveDateTime,
    #[typeshare(serialized_as = "String")]
    pub updated_at: NaiveDateTime,
    #[typeshare(serialized_as = "Option<String>")]
    pub last_login: Option<NaiveDateTime>,
    pub local_login: Option<LocalLoginData>,
    pub oauth_providers: HashMap<OAuthProvision, Vec<OAuthConnectionData>>,
}

#[typeshare]
#[derive(Serialize)]
pub struct LocalLoginData {
    #[typeshare(serialized_as = "String")]
    updated_at: NaiveDateTime,
}

impl From<LocalLogin> for LocalLoginData {
    fn from(value: LocalLogin) -> Self {
        LocalLoginData {
            updated_at: value.updated_at,
        }
    }
}

#[typeshare]
#[derive(Serialize)]
pub struct OAuthConnectionData {
    pub provider: String,
    #[typeshare(serialized_as = "String")]
    pub created_on: NaiveDateTime,
    #[typeshare(serialized_as = "String")]
    pub updated_at: NaiveDateTime,
}

impl From<OAuthConnection> for OAuthConnectionData {
    fn from(value: OAuthConnection) -> Self {
        Self {
            provider: value.provider,
            created_on: value.created_on,
            updated_at: value.updated_at,
        }
    }
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_me(
    UserFromParts { user, jar }: UserFromParts,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<UserData>), HttpError> {
    let mut conn = pool.get().await?;
    let user_data = user.get_user_data(&mut conn).await?;
    Ok((jar, Json(user_data)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn logout(
    State(session): State<SessionStore>,
    State(oid_clients): State<OpenIdClients>,
    State(config): State<StatefulConfig>,
    jar: CookieJar,
) -> (CookieJar, Json<Option<Url>>) {
    let (session_data, jar) = session.remove(jar).await;

    let mut logout_url = None;
    if let Some(provider) = session_data.and_then(|data| {
        data.rp_logout_providers_with_open_sessions
            .into_iter()
            .next()
    }) {
        if let Some(client_record) = oid_clients.get(&provider).await {
            if let Ok(redirect_url) =
                PostLogoutRedirectUrl::new(config.read().await.base_url.clone())
            {
                logout_url = client_record.end_session_endpoint.as_ref().map(|url| {
                    LogoutRequest::from(url.clone())
                        .set_post_logout_redirect_uri(redirect_url)
                        .http_get_url()
                });
            }
        }
    }

    (jar, Json(logout_url))
}

#[axum_macros::debug_handler(state = AppState)]
async fn update_me(
    UserFromParts { user, jar }: UserFromParts,
    State(pool): State<PgPool>,
    Json(update_user): Json<UpdateUser>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::users::dsl::*;

    let conn = &mut pool.get().await?;

    update(users)
        .filter(id.eq(user.id))
        .set(update_user)
        .execute(conn)
        .await?;

    Ok(jar)
}

#[typeshare]
#[derive(Serialize)]
pub struct PublicUserData {
    pub id: i32,
    pub email: String,
    pub phone_number: Option<String>,
    pub fname: String,
    pub lname: String,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub permission_level: PermissionLevel,
    #[typeshare(serialized_as = "String")]
    pub joined_on: NaiveDateTime,
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_user(
    Path(id_): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<Json<PublicUserData>, HttpError> {
    let conn = &mut pool.get().await?;
    let user = User::get(id_, conn)
        .await?
        .ok_or(HttpError::not_found("No user found"))?;
    Ok(Json(user.get_public_user_data()))
}

#[typeshare]
#[derive(Deserialize)]
pub struct CreateLocalUser {
    pub email: String,
    pub phone_number: Option<String>,
    pub fname: String,
    pub lname: String,
    pub permission_level: Option<PermissionLevel>,
    pub password: String,
}

#[axum_macros::debug_handler(state = AppState)]
async fn add_user(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Json(user_data): Json<CreateLocalUser>,
) -> Result<(CookieJar, String), HttpError> {
    use crate::schema::local_logins::dsl::*;
    use crate::schema::users::dsl::*;

    let conn = &mut pool.get().await?;

    let new_user = NewUser {
        email: &user_data.email,
        email_verified: false,
        phone_number: user_data.phone_number.as_deref(),
        fname: &user_data.fname,
        lname: &user_data.lname,
        permission_level: user_data.permission_level,
        bio: None,
        profile_image: None,
    };

    let id_: i32 = insert_into(users)
        .values(new_user)
        .returning(id)
        .get_result(conn)
        .await?;

    let new_local_login = NewLocalLogin {
        user_id: id_,
        hash: &bcrypt::hash(user_data.password, bcrypt::DEFAULT_COST)?,
    };

    insert_into(local_logins)
        .values(new_local_login)
        .execute(conn)
        .await?;

    Ok((jar, id_.to_string()))
}

#[axum_macros::debug_handler(state = AppState)]
async fn update_user(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    Path(id_): Path<i32>,
    State(pool): State<PgPool>,
    Json(update_user): Json<AdminUpdateUser>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::users::dsl::*;

    let conn = &mut pool.get().await?;

    update(users)
        .filter(id.eq(id_))
        .set(update_user)
        .execute(conn)
        .await?;

    Ok(jar)
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_all_users(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<Vec<UserData>>), HttpError> {
    use crate::schema::users::dsl::*;

    let conn = &mut pool.get().await?;

    let users_ = users.select(User::as_select()).load(conn).await?;

    let local_logins_ = LocalLogin::belonging_to(&users_)
        .select(LocalLogin::as_select())
        .load(conn)
        .await?
        .grouped_by(&users_);

    let oauth_providers_ = OAuthConnection::belonging_to(&users_)
        .select(OAuthConnection::as_select())
        .load(conn)
        .await?
        .grouped_by(&users_);

    let data = izip!(users_, local_logins_, oauth_providers_)
        .map(|(user, mut local_login, providers)| {
            let mut provider_map: HashMap<OAuthProvision, Vec<OAuthConnectionData>> =
                HashMap::new();
            for provider in providers {
                provider_map
                    .entry(provider.provides)
                    .or_default()
                    .push(provider.into());
            }
            UserData {
                id: user.id,
                email: user.email,
                email_verified: user.email_verified,
                phone_number: user.phone_number,
                fname: user.fname,
                lname: user.lname,
                bio: user.bio,
                profile_image: user.profile_image,
                permission_level: user.permission_level,
                joined_on: user.joined_on,
                updated_at: user.updated_at,
                last_login: user.last_login,
                local_login: local_login.pop().map(|l| l.into()),
                oauth_providers: provider_map,
            }
        })
        .collect::<Vec<_>>();

    Ok((jar, Json(data)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_user_admin(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path(id_): Path<i32>,
) -> Result<(CookieJar, Json<UserData>), HttpError> {
    let conn = &mut pool.get().await?;

    let user = User::get(id_, conn)
        .await?
        .ok_or(HttpError::not_found("user not found"))?;
    let user_data = user.get_user_data(conn).await?;

    Ok((jar, Json(user_data)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn delete_user(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path(id_): Path<i32>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::users::dsl::*;

    let conn = &mut pool.get().await?;

    delete(users).filter(id.eq(id_)).execute(conn).await?;

    Ok(jar)
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_my_groups(
    UserFromParts { jar, user }: UserFromParts,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<Vec<(Group, Option<PublicUserData>)>>), HttpError> {
    use crate::schema::groups;
    use crate::schema::is_member_of::dsl::*;
    use crate::schema::users::dsl::*;

    let conn = &mut pool.get().await?;

    let member_records: Vec<(IsMemberOf, Group, Option<User>)> = is_member_of
        .filter(user_id.eq(user.id))
        .inner_join(groups::table)
        .left_join(users.on(id.nullable().eq(assigned_teacher)))
        .load(conn)
        .await?;

    let groups = member_records
        .into_iter()
        .map(|record| {
            let (_, group, teacher) = record;
            (group, teacher.map(|u| u.get_public_user_data()))
        })
        .collect();

    Ok((jar, Json(groups)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_my_group_details(
    UserFromParts { jar, user }: UserFromParts,
    Path(group_id_): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<IsMemberOf>), HttpError> {
    use crate::schema::is_member_of::dsl::*;

    let conn = &mut pool.get().await?;

    let member_record: IsMemberOf = is_member_of.find((user.id, group_id_)).first(conn).await?;

    Ok((jar, Json(member_record)))
}

#[typeshare]
#[derive(Serialize)]
struct MembershipData {
    group: Group,
    assigned_teacher: Option<PublicUserData>,
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_user_groups(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    Path(id_): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<Vec<MembershipData>>), HttpError> {
    use crate::schema::groups;
    use crate::schema::is_member_of::dsl::*;
    use crate::schema::users::dsl::*;

    let conn = &mut pool.get().await?;

    let member_records: Vec<(IsMemberOf, Group, Option<User>)> = is_member_of
        .filter(user_id.eq(id_))
        .inner_join(groups::table)
        .left_join(users.on(id.nullable().eq(assigned_teacher)))
        .load(conn)
        .await?;

    let groups = member_records
        .into_iter()
        .map(|record| {
            let (_, group, teacher) = record;
            MembershipData {
                group,
                assigned_teacher: teacher.map(|u| u.get_public_user_data()),
            }
        })
        .collect();

    Ok((jar, Json(groups)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn add_user_to_group(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path((user_id_, group_id_)): Path<(i32, i32)>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::is_member_of::dsl::*;

    let conn = &mut pool.get().await?;

    let new_member = CreateIsMemberOf {
        user_id: user_id_,
        group_id: group_id_,
        assigned_teacher: None,
    };

    insert_into(is_member_of)
        .values(new_member)
        .execute(conn)
        .await?;

    Ok(jar)
}

#[axum_macros::debug_handler(state = AppState)]
async fn remove_user_from_group(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path((user_id_, group_id_)): Path<(i32, i32)>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::is_member_of::dsl::*;

    let conn = &mut pool.get().await?;

    delete(is_member_of.find((user_id_, group_id_)))
        .execute(conn)
        .await?;

    Ok(jar)
}

#[axum_macros::debug_handler(state = AppState)]
async fn update_group_membership(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path((user_id_, group_id_)): Path<(i32, i32)>,
    Json(update_is_member_of): Json<UpdateIsMemberOf>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::is_member_of::dsl::*;

    let conn = &mut pool.get().await?;

    update(is_member_of.find((user_id_, group_id_)))
        .set(update_is_member_of)
        .execute(conn)
        .await?;

    Ok(jar)
}
