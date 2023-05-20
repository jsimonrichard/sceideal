use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use diesel::{delete, prelude::*};
use diesel::{insert_into, update};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use typeshare::typeshare;

use crate::{
    http_error::HttpError,
    model::{Location, NewLocation, UpdateLocation},
    schema::locations,
    user::{TeacherFromParts, UserFromParts},
    AppState, PgPool,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/u", get(get_my_locations).post(create_location))
        .route(
            "/u/:location_id",
            get(get_my_location)
                .put(update_location)
                .delete(delete_location),
        )
        .route("/:user_id", get(get_locations))
        .route("/:user_id/:location_id", get(get_location))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_my_locations(
    TeacherFromParts(UserFromParts { user, jar }): TeacherFromParts,
    State(pool): State<PgPool>,
) -> Result<(CookieJar, Json<Vec<Location>>), HttpError> {
    use crate::schema::locations::dsl::*;

    let mut conn = pool.get().await?;

    let location_list = locations
        .filter(user_id.eq(user.id))
        .load(&mut conn)
        .await?;

    Ok((jar, Json(location_list)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_my_location(
    TeacherFromParts(UserFromParts { user, jar }): TeacherFromParts,
    State(pool): State<PgPool>,
    Path(location_id): Path<i32>,
) -> Result<(CookieJar, Json<Location>), HttpError> {
    use crate::schema::locations::dsl::*;

    let mut conn = pool.get().await?;

    let location = locations
        .find((location_id, user.id))
        .get_result(&mut conn)
        .await?;

    Ok((jar, Json(location)))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_locations(
    State(pool): State<PgPool>,
    Path(user_id_): Path<i32>,
) -> Result<Json<Vec<Location>>, HttpError> {
    use crate::schema::locations::dsl::*;

    let mut conn = pool.get().await?;

    let location_list = locations
        .filter(user_id.eq(user_id_))
        .load(&mut conn)
        .await?;

    Ok(Json(location_list))
}

#[axum_macros::debug_handler(state = AppState)]
async fn get_location(
    State(pool): State<PgPool>,
    Path((user_id_, location_id)): Path<(i32, i32)>,
) -> Result<Json<Location>, HttpError> {
    use crate::schema::locations::dsl::*;

    let mut conn = pool.get().await?;

    let location = locations
        .find((user_id_, location_id))
        .get_result(&mut conn)
        .await?;

    Ok(Json(location))
}

#[typeshare]
#[derive(Deserialize)]
pub struct CreateLocation {
    type_: Option<String>,
    name: String,
    description: Option<String>,
    requirements: Option<String>,
}
#[axum_macros::debug_handler(state = AppState)]
async fn create_location(
    TeacherFromParts(UserFromParts { user, jar }): TeacherFromParts,
    State(pool): State<PgPool>,
    Json(create_location): Json<CreateLocation>,
) -> Result<(CookieJar, String), HttpError> {
    let new_location_data = NewLocation {
        user_id: user.id,
        type_: create_location.type_.as_deref(),
        name: &create_location.name,
        description: create_location.description.as_deref(),
        requirements: create_location.requirements.as_deref(),
    };

    let mut conn = pool.get().await?;

    let id: i32 = insert_into(locations::table)
        .values(&new_location_data)
        .returning(locations::id)
        .get_result(&mut conn)
        .await?;

    Ok((jar, id.to_string()))
}

#[axum_macros::debug_handler(state = AppState)]
async fn update_location(
    TeacherFromParts(UserFromParts { user, jar }): TeacherFromParts,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(update_location): Json<UpdateLocation>,
) -> Result<CookieJar, HttpError> {
    use locations::dsl::locations;

    let mut conn = pool.get().await?;

    update(locations.find((id, user.id)))
        .set(&update_location)
        .execute(&mut conn)
        .await?;

    Ok(jar)
}

#[axum_macros::debug_handler(state = AppState)]
async fn delete_location(
    TeacherFromParts(UserFromParts { user, jar }): TeacherFromParts,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<CookieJar, HttpError> {
    use locations::dsl::locations;

    let mut conn = pool.get().await?;

    delete(locations.find((id, user.id)))
        .execute(&mut conn)
        .await?;

    Ok(jar)
}
