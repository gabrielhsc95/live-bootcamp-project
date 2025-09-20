use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DROPLET_IP_ENV_VAR: &str = "DROPLET_IP";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

fn set_constant(env_key: &str, default: Option<&str>) -> String {
    dotenv().ok();
    match std_env::var(env_key) {
        Ok(secret) => secret,
        Err(_) => {
            if default.is_some() {
                default.unwrap().to_string()
            } else {
                panic!("{} must be set.", env_key);
            }
        }
    }
}

lazy_static! {
    pub static ref JWT_SECRET: String = set_constant(env::JWT_SECRET_ENV_VAR, None);
    pub static ref DROPLET_IP: String = set_constant(env::DROPLET_IP_ENV_VAR, None);
    pub static ref DATABASE_URL: String = set_constant(env::DATABASE_URL_ENV_VAR, None);
    pub static ref REDIS_HOST_NAME: String =
        set_constant(env::REDIS_HOST_NAME_ENV_VAR, Some(DEFAULT_REDIS_HOSTNAME));
}
