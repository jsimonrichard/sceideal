// Modified from https://fasterthanli.me/series/updating-fasterthanli-me-for-2022/part-2#the-opinions-of-axum-also-nice-error-handling

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use color_eyre::Report;
use tracing::error;

#[derive(Debug)]
pub enum HttpError {
    WithCode { code: StatusCode, msg: &'static str },
    Internal { err: String },
}

macro_rules! constructor {
    ($name:ident, $code:expr) => {
        pub fn $name(msg: &'static str) -> Self {
            HttpError::WithCode { code: $code, msg }
        }
    };
}

impl HttpError {
    constructor!(internal, StatusCode::INTERNAL_SERVER_ERROR);
    constructor!(forbidden, StatusCode::FORBIDDEN);
    // constructor!(bad_request, StatusCode::BAD_REQUEST);
    constructor!(not_found, StatusCode::NOT_FOUND);

    fn from_report(err: Report) -> Self {
        error!("HTTP handler error: {}", err);
        HttpError::Internal {
            err: err.to_string(),
        }
    }
}

macro_rules! impl_from {
    ($from:ty) => {
        impl From<$from> for HttpError {
            fn from(err: $from) -> Self {
                Self::from_report(err.into())
            }
        }
    };
}

impl_from!(std::io::Error);
impl_from!(color_eyre::Report);
impl_from!(diesel::result::Error);
impl_from!(diesel_async::pooled_connection::bb8::RunError);
impl_from!(bcrypt::BcryptError);

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            HttpError::WithCode { code, msg } => (code, msg).into_response(),
            HttpError::Internal { err } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                err,
            )
                .into_response(),
        }
    }
}
