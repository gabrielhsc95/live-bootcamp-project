use crate::{domain::UserStore, services::hashmap_user_store::HashmapUserStore};
use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    serve::Serve,
};
use domain::AuthAPIError;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::marker::Send;
use std::marker::Sync;
use tower_http::services::ServeDir;

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;

use crate::app_state::AppState;

use routes::*;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}

pub struct Application<T: UserStore + Clone + Send + Sync + 'static> {
    server: Serve<Router, Router>,
    pub address: String,
    app_state: AppState<T>,
}

impl<T: UserStore + Clone + Send + Sync> Application<T> {
    pub async fn build(app_state: AppState<T>, address: &str) -> Result<Self, Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/verify-2fa", post(verify_2fa))
            .route("/logout", post(logout))
            .route("/verify-token", post(verify_token))
            .with_state(app_state.clone());

        let server = axum::serve(listener, router);

        Ok(Application {
            server,
            address,
            app_state,
        })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
