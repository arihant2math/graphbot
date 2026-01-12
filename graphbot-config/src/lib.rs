use anyhow::Context;
use serde::{Deserialize, Serialize};

const CONF_DIR: &str = "conf/";
const SECRET_FILE: &str = "secret.toml";
const MAIN_FILE: &str = "main.toml";

#[derive(Deserialize, Serialize, Debug, Default)]
struct Secret {
    pub access_token: String,
    pub client_secret: String,
    pub client_id: String,
    pub tools_db_password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Rpc {
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl Default for Rpc {
    fn default() -> Self {
        Rpc {
            host: "localhost".to_string(),
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
    pub db_url: String,
    pub search_category: String,
    pub num_workers: Option<usize>,
}

impl Default for GraphTask {
    fn default() -> Self {
        GraphTask {
            db_url: String::new(),
            search_category: "Category:Graphs_to_Port".to_string(),
            num_workers: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct RfdTask {
    pub wiki_replica_db_url: String,
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
    #[serde(default)]
    pub rfd_task: RfdTask,
}

impl Default for Main {
    fn default() -> Self {
        Main {
            username: "GraphBot".to_string(),
            wiki: "https://en.wikipedia.org/".to_string(),
            rpc: Rpc::default(),
            server: Server::default(),
            graph_task: GraphTask::default(),
            rfd_task: RfdTask::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub access_token: String,
    pub client_secret: String,
    pub client_id: String,
    pub tools_db_password: String,
    pub username: String,
    pub wiki: String,
    pub rpc: Rpc,
    pub server: Server,
    pub graph_task: GraphTask,
    pub rfd_task: RfdTask,
    pub shutdown_graph_task: bool,
    pub pause_graph_task: bool,
    pub shutdown_rfd_task: bool,
    pub pause_rfd_task: bool,
}

impl Config {
    fn from_parts(secret: Secret, main: Main) -> Self {
        let db_password = &secret.tools_db_password;
        let mut graph_task = main.graph_task;
        let mut rfd_task = main.rfd_task;
        graph_task.db_url = graph_task.db_url.replace("{{password}}", db_password);
        rfd_task.wiki_replica_db_url = rfd_task.wiki_replica_db_url.replace("{{password}}", db_password);
        Config {
            access_token: secret.access_token,
            client_secret: secret.client_secret,
            client_id: secret.client_id,
            tools_db_password: secret.tools_db_password,
            username: main.username,
            wiki: main.wiki,
            rpc: main.rpc,
            server: main.server,
            graph_task,
            rfd_task,
            shutdown_graph_task: false,
            pause_graph_task: false,
            shutdown_rfd_task: true,
            pause_rfd_task: true,
        }
    }

    /// Load configuration from conf/secret.toml and conf/main.toml
    /// Searches for conf/ directory in current directory or parent directories
    /// # Errors
    /// - If conf/ directory is not found
    /// - If secret.toml or main.toml cannot be read or parsed
    /// # Panics
    /// - If the current directory cannot be determined
    pub fn load() -> anyhow::Result<Self> {
        // Find conf directory in current directory or parent directories
        let mut dir = std::env::current_dir().context("Failed to get current directory")?;
        loop {
            if dir.join(CONF_DIR).is_dir() {
                break;
            }
            if !dir.pop() {
                return Err(anyhow::anyhow!(
                    "Failed to find conf directory in current or parent directories"
                ));
            }
        }
        let conf_dir = dir.join(CONF_DIR);
        let secret_file = conf_dir.join(SECRET_FILE);
        let main_file = conf_dir.join(MAIN_FILE);

        let secret = toml::from_str(
            &std::fs::read_to_string(secret_file).context("Failed to open secret config file")?,
        )
        .context("Failed to parse secret config file")?;
        let main = toml::from_str(
            &std::fs::read_to_string(main_file).context("Failed to open main config file")?,
        )
        .context("Failed to parse main config file")?;
        Ok(Self::from_parts(secret, main))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_parts(Secret::default(), Main::default())
    }
}
