use log::warn;
use std::env;

#[derive(Debug)]
pub struct Env {
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_host: String,
    pub postgres_port: String,
}

impl Env {
    pub async fn new() -> Self {
        let postgres_user = env::var("SHAMEBOT_POSTGRES_USER")
            .map_err(|_| warn!("environment variable SHAMEBOT_POSTGRES_USER not set"))
            .unwrap_or_default();
        let postgres_password = env::var("SHAMEBOT_POSTGRES_PASSWORD")
            .map_err(|_| warn!("environment variable SHAMEBOT_POSTGRES_PASSWORD not set"))
            .unwrap_or_default();
        let postgres_host = env::var("SHAMEBOT_POSTGRES_HOST")
            .map_err(|_| warn!("environment variable SHAMEBOT_POSTGRES_HOST not set"))
            .unwrap_or_default();
        let postgres_port = env::var("SHAMEBOT_POSTGRES_PORT")
            .map_err(|_| warn!("environment variable SHAMEBOT_POSTGRES_PORT not set"))
            .unwrap_or_default();

        Env {
            postgres_user,
            postgres_password,
            postgres_host,
            postgres_port,
        }
    }
}
