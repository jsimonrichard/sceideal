impl User {
    pub fn verify(&self, password: String) -> Result<bool, UserError> {
        bcrypt::verify(password, &self.hash).map_err(|_| UserError::UserNotFound)
    }
}

#[typeshare]
#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    email: String,
    password: String,
}
#[axum_macros::debug_handler(state = AppState)]
async fn sign_up(
    State(pool): State<PgPool>,
    State(session): State<SessionStore>,
    State(rng): State<StdRng>,
    jar: CookieJar,
    Json(create_user): Json<CreateUser>,
) -> Result<(CookieJar, User), UserError> {
    let conn = &mut pool
        .get()
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

    let CreateUser {
        username,
        email,
        password,
    } = create_user;
    User::create(&username, &email, &password, conn)?;

    let user = User::find(&username, conn)?;

    // Create user session
    Ok((start_session(user.id, jar, rng, session).await, user))
}

#[typeshare]
#[derive(Deserialize)]
pub struct LoginData {
    name_query: String,
    password: String,
}
#[axum_macros::debug_handler(state = AppState)]
async fn login(
    State(pool): State<PgPool>,
    State(session): State<SessionStore>,
    State(rng): State<StdRng>,
    jar: CookieJar,
    Json(login_data): Json<LoginData>,
) -> Result<(CookieJar, User), UserError> {
    let conn = &mut pool
        .get()
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

    let LoginData {
        name_query,
        password,
    } = login_data;
    let user = User::find(&name_query, conn)?;

    trace!("Found possible match for {name_query}: {user:?}");

    let result = user.verify(password)?;
    trace!("Verify result: {result}");

    if result {
        trace!("password verified!");
        Ok((start_session(user.id, jar, rng, session).await, user))
    } else {
        Err(UserError::UserNotFound)
    }
}
