use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    pub remote_url: String,
    pub uuid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub client: ClientConfig,
}
