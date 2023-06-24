use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use diesel::{delete, insert_into, prelude::*, update};
use diesel_async::RunQueryDsl;

use crate::{
    http_error::HttpError,
    model::{CreateGroup, Group, IsMemberOf, UpdateGroup, User},
    user::{session::SessionStore, AdminFromParts, UserFromParts},
    AppState, PgPool,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_groups))
        .route("/:id", get(get_group))
        .route("/a", get(get_all_groups_admin).post(create_group))
        .route(
            "/a/:id",
            get(get_group_admin).put(update_group).delete(delete_group),
        )
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_all_groups(
    State(pool): State<PgPool>,
    State(session): State<SessionStore>,
    mut jar: CookieJar,
) -> Result<(CookieJar, Json<Vec<Group>>), HttpError> {
    use crate::schema::groups::dsl::*;

    let conn = &mut pool.get().await?;

    let group_dsl = groups.filter(public.eq(true)).select(Group::as_select());

    let groups_ = if let Some(user) = User::from_jar(&jar, &session, conn).await? {
        jar = session.reup(jar).await;
        group_dsl
            .union(
                IsMemberOf::belonging_to(&user)
                    .inner_join(groups)
                    .select(Group::as_select()),
            )
            .load(conn)
            .await?
    } else {
        group_dsl.load(conn).await?
    };

    Ok((jar, Json(groups_)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_group(
    State(pool): State<PgPool>,
    State(session): State<SessionStore>,
    mut jar: CookieJar,
    Path(id_): Path<i32>,
) -> Result<(CookieJar, Json<Group>), HttpError> {
    use crate::schema::groups::dsl::*;
    use crate::schema::is_member_of::dsl::*;

    let conn = &mut pool.get().await?;

    let group = groups
        .find(id_)
        .select(Group::as_select())
        .get_result(conn)
        .await
        .optional()?
        .ok_or(HttpError::not_found("Group not found"))?;

    if group.public {
        return Ok((jar, Json(group)));
    }

    let user = User::from_jar(&jar, &session, conn)
        .await?
        .ok_or(HttpError::forbidden("No user found"))?;
    jar = session.reup(jar).await;

    // Check membership
    is_member_of
        .find((user.id, id_))
        .select(IsMemberOf::as_select())
        .first(conn)
        .await
        .optional()?
        .ok_or(HttpError::forbidden("You are not a member of this group"))?;

    Ok((jar, Json(group)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_all_groups_admin(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<Vec<Group>>), HttpError> {
    use crate::schema::groups::dsl::*;

    let conn = &mut pool.get().await?;

    let groups_ = groups.select(Group::as_select()).load(conn).await?;

    Ok((jar, Json(groups_)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_group_admin(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path(id_): Path<i32>,
) -> Result<(CookieJar, Json<Group>), HttpError> {
    use crate::schema::groups::dsl::*;

    let conn = &mut pool.get().await?;

    let group = groups
        .find(id_)
        .select(Group::as_select())
        .first(conn)
        .await
        .optional()?
        .ok_or(HttpError::not_found("Group not found"))?;

    Ok((jar, Json(group)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn create_group(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Json(group_data): Json<CreateGroup>,
) -> Result<(CookieJar, String), HttpError> {
    use crate::schema::groups::dsl::*;

    let conn = &mut pool.get().await?;

    let id_: i32 = insert_into(groups)
        .values(group_data)
        .returning(id)
        .get_result(conn)
        .await?;

    Ok((jar, id_.to_string()))
}

#[axum_macros::debug_handler(state = AppState)]
async fn update_group(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path(id_): Path<i32>,
    Json(group_data): Json<UpdateGroup>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::groups::dsl::*;

    let conn = &mut pool.get().await?;

    update(groups.find(id_))
        .set(group_data)
        .execute(conn)
        .await?;

    Ok(jar)
}

#[axum_macros::debug_handler(state = AppState)]
async fn delete_group(
    AdminFromParts(UserFromParts { jar, .. }): AdminFromParts,
    State(pool): State<PgPool>,
    Path(id_): Path<i32>,
) -> Result<CookieJar, HttpError> {
    use crate::schema::groups::dsl::*;

    let conn = &mut pool.get().await?;

    delete(groups.find(id_)).execute(conn).await?;

    Ok(jar)
}
