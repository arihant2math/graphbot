use serde::{Deserialize, Serialize};

const SECRET_FILE: &str = "conf/secret.toml";

#[derive(Deserialize, Serialize, Debug, Default)]
struct Secret {
    pub access_token: String,
    pub client_secret: String,
    pub client_id: String,
}

const MAIN_FILE: &str = "conf/main.toml";

#[derive(Deserialize, Serialize, Debug)]
pub struct Rpc {
    pub port: u16,
    pub path: String,
}

impl Default for Rpc {
    fn default() -> Self {
        Rpc {
            port: 8080,
            path: "api".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Main {
    pub search_category: String,
    pub username: String,
    pub wiki: String,
    pub rpc: Rpc,
}

impl Default for Main {
    fn default() -> Self {
        Main {
            search_category: "Category:Graphs_to_Port".to_string(),
            username: "GraphBot".to_string(),
            wiki: "https://en.wikipedia.org/".to_string(),
            rpc: Rpc::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub access_token: String,
    pub client_secret: String,
    pub client_id: String,
    pub search_category: String,
    pub username: String,
    pub wiki: String,
    pub rpc: Rpc,
}

impl Config {
    fn from_parts(secret: Secret, main: Main) -> Self {
        Config {
            access_token: secret.access_token,
            client_secret: secret.client_secret,
            client_id: secret.client_id,
            search_category: main.search_category,
            username: main.username,
            wiki: main.wiki,
            rpc: main.rpc,
        }
    }

    pub fn load() -> anyhow::Result<Self> {
        let secret = toml::from_str(&std::fs::read_to_string(SECRET_FILE)?)?;
        let main = toml::from_str(&std::fs::read_to_string(MAIN_FILE)?)?;
        Ok(Self::from_parts(secret, main))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_parts(Secret::default(), Main::default())
    }
}