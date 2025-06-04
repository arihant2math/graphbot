use serde::{Deserialize, Serialize};

const SECRET_FILE: &str = "conf/secret.toml";

#[derive(Deserialize, Serialize, Debug)]
struct Secret {
    pub access_token: String,
    pub client_secret: String,
    pub client_id: String,
}

const MAIN_FILE: &str = "conf/main.toml";

#[derive(Deserialize, Serialize, Debug)]
struct Main {
    pub search_category: String,
    pub username: String,
    pub wiki: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub access_token: String,
    pub client_secret: String,
    pub client_id: String,
    pub search_category: String,
    pub username: String,
    pub wiki: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let Secret { access_token, client_secret, client_id } =
            toml::from_str(&std::fs::read_to_string(SECRET_FILE)?)?;
        let Main { search_category, username, wiki } = toml::from_str(&std::fs::read_to_string(MAIN_FILE)?)?;
        Ok(Config {
            access_token,
            client_secret,
            client_id,
            search_category,
            username,
            wiki
        })
    }
}