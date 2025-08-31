use crate::domain::UserStore;
use axum::{Router, routing::post, serve::Serve};
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
