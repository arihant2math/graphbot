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
pub struct Server {
    pub port: u16,
}

impl Default for Server {
    fn default() -> Self {
        Server { port: 8081 }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphTask {
    pub search_category: String,
    pub num_workers: Option<usize>,
}

impl Default for GraphTask {
    fn default() -> Self {
        GraphTask {
            search_category: "Category:Graphs_to_Port".to_string(),
            num_workers: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Main {
    pub username: String,
    pub wiki: String,
    #[serde(default)]
    pub rpc: Rpc,
    #[serde(default)]
    pub server: Server,
    #[serde(default)]
    pub graph_task: GraphTask,
}

impl Default for Main {
    fn default() -> Self {
        Main {
            username: "GraphBot".to_string(),
            wiki: "https://en.wikipedia.org/".to_string(),
            rpc: Rpc::default(),
            server: Server::default(),
            graph_task: GraphTask::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub access_token: String,
    pub client_secret: String,
    pub client_id: String,
    pub username: String,
    pub wiki: String,
    pub rpc: Rpc,
    pub server: Server,
    pub graph_task: GraphTask,
    pub shutdown_graph_task: bool,
    pub pause_graph_task: bool,
}

impl Config {
    fn from_parts(secret: Secret, main: Main) -> Self {
        Config {
            access_token: secret.access_token,
            client_secret: secret.client_secret,
            client_id: secret.client_id,
            username: main.username,
            wiki: main.wiki,
            rpc: main.rpc,
            server: main.server,
            graph_task: main.graph_task,
            shutdown_graph_task: false,
            pause_graph_task: false,
        }
    }

    fn into_parts(self) -> (Secret, Main) {
        (
            Secret {
                access_token: self.access_token,
                client_secret: self.client_secret,
                client_id: self.client_id,
            },
            Main {
                username: self.username,
                wiki: self.wiki,
                rpc: self.rpc,
                server: self.server,
                graph_task: self.graph_task,
            },
        )
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
