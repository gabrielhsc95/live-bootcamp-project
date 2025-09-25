use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use color_eyre::eyre::Report;
use thiserror::Error;

use serde::{Deserialize, Serialize};

fn log_error_chain(e: &(dyn std::error::Error + 'static)) {
    let mut report = format!("{:?}\n", e);
    let mut current = e.source();
    while let Some(cause) = current {
        let str = format!("Caused by:\n\n{:?}", cause);
        report = format!("{}\n{}", report, str);
        current = cause.source();
    }
    report = format!("{}", report);
    tracing::error!(name:"error", "{}", report);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Incorrect credentials")]
    IncorrectCredentials,
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        log_error_chain(&self);
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
            AuthAPIError::IncorrectCredentials => {
                (StatusCode::UNAUTHORIZED, "Authentication failed")
            }
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Invalid input"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "JWT is not valid"),
        };
        let body = Json(ErrorResponse {
            error: error_message.to_owned(),
        });
        (status, body).into_response()
    }
}
