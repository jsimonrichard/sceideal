use cfg_if::cfg_if;
use color_eyre::Result;
use leptos::*;
use serde::{Deserialize, Serialize};


#[cfg(feature = "ssr")]
use diesel::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
pub struct User {
    pub id: i32,
    pub username: String,
    hash: String,
    pub email: String,
}

cfg_if! {if #[cfg(feature = "ssr")]{
    use crate::schema::users;
    use crate::PooledPgConnection;
    use diesel::*;

    #[derive(Insertable)]
    #[diesel(table_name=users)]
    struct NewUser<'a> {
        username: &'a str,
        hash: &'a str,
        email: &'a str,
    }

    impl User {
        pub fn get(name_query: &str, connection: &mut PgConnection) -> Result<Self, ServerFnError> {
            use crate::schema::users::dsl::*;
            users
                .filter(username.eq(&name_query))
                .or_filter(email.eq(&name_query))
                .first(connection)
                .map_err(to_server_error)
        }

        pub fn verify(&self, password: String) -> Result<bool, ServerFnError> {
            bcrypt::verify(password, &self.hash)
                .map_err(to_server_error)
        }

        pub fn create(
            username: &str,
            email: &str,
            password: &str,
            connection: &mut PooledPgConnection
        ) -> Result<(), ServerFnError> {
            insert_into(users::table).values(&NewUser {
                username,
                email,
                hash: &bcrypt::hash(password, bcrypt::DEFAULT_COST)
                    .map_err(to_server_error)?
            })
                .execute(connection)
                .map_err(to_server_error)?;
            Ok(())
        }
    }
}}

#[server(Signup, "/api")]
pub async fn sign_up(
    cx: Scope,
    username: String,
    email: String,
    password: String,
) -> Result<(), ServerFnError> {
    use crate::{get_connection, get_session};

    let mut connection = get_connection(cx)?;
    let mut session = get_session(cx)?;
    User::create(&username, &email, &password, &mut connection)?;

    let user = User::get(&username, &mut connection)?;
    // session.insert()

    Ok(())
}

fn to_server_error(e: impl ToString) -> ServerFnError {
    ServerFnError::ServerError(e.to_string())
}
