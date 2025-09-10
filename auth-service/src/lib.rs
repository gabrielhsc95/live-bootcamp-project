use crate::domain::{BannedTokenStore, TwoFACodeStore, UserStore};
use axum::{Router, http::Method, routing::post, serve::Serve};
use std::error::Error;
use tower_http::{cors::CorsLayer, services::ServeDir};
use utils::constants::DROPLET_IP;

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

use crate::app_state::AppState;

use routes::*;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build<
        T: UserStore + 'static,
        U: BannedTokenStore + 'static,
        V: TwoFACodeStore + 'static,
    >(
        app_state: AppState<T, U, V>,
        address: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();

        let allowed_origins = [
            "http://localhost:8000".parse()?,
            format!("http://{}:8000", DROPLET_IP.as_str()).parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/verify-2fa", post(verify_2fa))
            .route("/logout", post(logout))
            .route("/verify-token", post(verify_token))
            .with_state(app_state.clone())
            .layer(cors);

        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
