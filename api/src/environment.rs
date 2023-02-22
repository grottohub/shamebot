use log::warn;
use std::env;

#[derive(Debug)]
pub struct Env {
    pub client_id: u64,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Env {
    pub fn new() -> Self {
        let client_id = env::var("SHAMEBOT_CLIENT_ID")
            .map_err(|_| warn!("environment variable SHAMEBOT_CLIENT_ID not set"))
            .unwrap_or_default()
            .parse::<u64>()
            .map_err(|e| warn!("error parsing SHAMEBOT_CLIENT_ID as u64: {}", e))
            .unwrap_or_default();
        let client_secret = env::var("SHAMEBOT_CLIENT_SECRET")
            .map_err(|_| warn!("environment variable SHAMEBOT_CLIENT_SECRET not set"))
            .unwrap_or_default();
        let redirect_uri = env::var("SHAMEBOT_REDIRECT_URI")
            .map_err(|_| warn!("environment variable SHAMEBOT_REDIRECT_URI not set"))
            .unwrap_or_default();

        Env {
            client_id,
            client_secret,
            redirect_uri,
        }
    }
}
