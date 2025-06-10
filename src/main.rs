mod config;
mod convert;
mod page_handler;
mod parser;
pub mod schema;
mod rev_info;

use std::{sync::Arc, time::Duration};
use anyhow::Context;
use bincode::{Decode, Encode};
use dashmap::DashMap;
use mwbot::{
    generators::{CategoryMemberSort, CategoryMembers, Generator}, Bot,
    Page,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc, mpsc::Receiver, oneshot, oneshot::Sender, Mutex},
    task,
    task::JoinHandle,
    time::sleep,
};
use tracing::{error, info, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer};
use rev_info::RevInfo;
use crate::config::Config;
use std::io::Write;

#[derive(Default, Serialize, Deserialize, Encode, Decode)]
#[repr(transparent)]
pub struct FailedRevs(#[bincode(with_serde)] DashMap<RevInfo, String>);

impl FailedRevs {
    pub fn from_file() -> anyhow::Result<Self> {
        let file = std::fs::File::open("failed_revisions.bin")
            .context("Failed to open failed revisions file")?;
        let reader = std::io::BufReader::new(file);
        let deserialized: Self = bincode::decode_from_reader(reader, bincode::config::standard())
            .context("Failed to deserialize failed revisions")?;
        Ok(deserialized)
    }

    pub fn load() -> Self {
        Self::from_file().unwrap_or_else(|e| {
            error!("Failed to load failed revisions: {e}");
            Self::default()
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let file = std::fs::File::create("failed_revisions.bin")
            .context("Failed to create failed revisions file")?;
        let mut writer = std::io::BufWriter::new(file);
        bincode::encode_into_std_write(&self, &mut writer, bincode::config::standard())
            .context("Failed to serialize failed revisions")?;
        writer.flush().context("Failed to flush failed revisions file")?;
        Ok(())
    }

    pub fn insert(&self, rev_info: RevInfo, error: anyhow::Error) {
        self.0.insert(rev_info, error.to_string());
    }

    pub fn contains_key(&self, rev_info: &RevInfo) -> bool {
        self.0.contains_key(rev_info)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}


pub const TAB_EXT: &str = ".tab";
pub const CHART_EXT: &str = ".chart";

async fn check_shutdown(bot: &Bot, wiki: &str, config: &Config) -> anyhow::Result<bool> {
    if bot
        .page(&format!("User:{}/Shutdown", config.username))?
        .exists()
        .await?
    {
        error!(
            "This bot has been shut down on {}. Please do not use it.",
            wiki
        );
        return Ok(true);
    }
    Ok(false)
}

fn init_logging(_config: &Config) -> non_blocking::WorkerGuard {
    let file_appender = rolling::hourly("logs/", "graphport.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::Layer::new()
        .with_ansi(true)
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::new("info"));

    let file_layer = fmt::Layer::new()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(
            EnvFilter::new("trace")
                .add_directive("hyper=info".parse().unwrap())
                .add_directive("h2=info".parse().unwrap())
                .add_directive("mwbot=debug".parse().unwrap())
                .add_directive("parsoid=debug".parse().unwrap()),
        );

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
    guard
}

const COMMONS_API_URL: &str = "https://commons.wikimedia.org/w/api.php";
const COMMONS_REST_URL: &str = "https://commons.wikimedia.org/api/rest_v1";

const USER_AGENT: &str = "GraphPort/1";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let url = url::Url::parse(&config.wiki)?;
    let api_url = url.join("w/api.php")?;
    let rest_url = url.join("api/rest_v1")?;
    // Read the config file
    let _guard = init_logging(&config);
    // check for parser load
    if let Err(e) = parser::call_parser("", &config) {
        error!("Parser failed to parse empty string, are you sure it is running? {e}");
        return Err(e.into());
    }
    let token = config.access_token.clone();
    let init_bots_span = tracing::debug_span!("init_bots").entered();
    let wiki_bot = Bot::builder(api_url.to_string(), rest_url.to_string())
        .set_user_agent(USER_AGENT.to_string())
        .set_mark_as_bot(true)
        .set_oauth2_token(config.username.clone(), token.clone())
        .build()
        .await?;

    let commons_bot = Bot::builder(COMMONS_API_URL.to_string(), COMMONS_REST_URL.to_string())
        .set_user_agent(USER_AGENT.to_string())
        .set_mark_as_bot(true)
        .set_oauth2_token(config.username.clone(), token)
        .build()
        .await?;
    init_bots_span.exit();

    let wiki_bot = Arc::new(wiki_bot);
    let commons_bot = Arc::new(commons_bot);
    let config = Arc::new(config);

    let failed_revs = Arc::new(FailedRevs::load());

    info!("Starting GraphPort bot");

    // Create workers
    let (page_sender, page_reciever) = mpsc::channel(100);
    let rx = Arc::new(Mutex::new(page_reciever));
    let workers = spawn_workers(&wiki_bot, &commons_bot, &config, &rx);

    loop {
        // Get the list of articles to process
        let generator =
            CategoryMembers::new(&config.search_category).sort(CategoryMemberSort::Timestamp);
        let mut output = generator.generate(&wiki_bot);
        while let Some(Ok(o)) = output.recv().await {
            let revid = get_revid(&o, &wiki_bot).await;
            let page_title = o.title().to_string();
            let rev_info = revid.map(|id| RevInfo::new(id, page_title));
            if let Some(ref rev_info) = rev_info {
                if failed_revs.contains_key(rev_info) {
                    warn!(
                        "Skipping page {} with revision ID {} due to previous failure",
                        rev_info.page_title, rev_info.id
                    );
                    continue;
                }
            }
            if check_shutdown(&commons_bot, "https://commons.wikimedia.org/", &config).await? {
                info!("Shutdown detected, exiting.");
                break;
            }
            if check_shutdown(&wiki_bot, &config.wiki, &config).await? {
                info!("Shutdown detected, exiting.");
                break;
            }

            let (send, rec) = oneshot::channel();
            if let Err(e) = page_sender.send((o, send)).await {
                error!("Failed to send page to handler: {e}");
                continue;
            }
            task::spawn({
                let failed_revs = Arc::clone(&failed_revs);
                async move {
                    match rec.await {
                        Ok(result) => {
                            if let Err(e) = result {
                                error!("Error processing page: {e}");
                                if let Some(rev_info) = rev_info {
                                    failed_revs.insert(rev_info, e);
                                }
                            }
                        }
                        Err(e) => error!("Failed to receive result: {e}"),
                    }
                }
            });
        }
        if check_shutdown(&commons_bot, "https://commons.wikimedia.org/", &config).await? {
            info!("Shutdown detected, exiting.");
            break;
        }
        if check_shutdown(&wiki_bot, &config.wiki, &config).await? {
            info!("Shutdown detected, exiting.");
            break;
        }
        info!("No more articles found in {}", config.search_category);
        // Write failed revisions to file
        if !failed_revs.is_empty() {
            if let Err(e) = failed_revs.save() {
                error!("Failed to save failed revisions: {e}");
            } else {
                info!("Failed revisions saved");
            }
        } else {
            info!("No failed revisions to write.");
        }
        sleep(Duration::from_secs(10)).await;
    }
    drop(workers); // Ensure all worker handles are dropped right before exiting
    Ok(())
}

fn spawn_workers(
    wiki_bot: &Arc<Bot>,
    commons_bot: &Arc<Bot>,
    config: &Arc<Config>,
    rx: &Arc<Mutex<Receiver<(Page, Sender<anyhow::Result<()>>)>>>,
) -> Vec<JoinHandle<()>> {
    const NUM_WORKERS: usize = 4;

    let mut workers = Vec::with_capacity(NUM_WORKERS);
    for _ in 0..NUM_WORKERS {
        let wiki_bot = Arc::clone(&wiki_bot);
        let commons_bot = Arc::clone(&commons_bot);
        let config = Arc::clone(&config);
        let rx = Arc::clone(&rx);
        let worker = tokio::spawn(async move {
            page_handler::page_handler(rx, commons_bot, wiki_bot, config).await;
        });
        workers.push(worker);
    }
    workers
}

async fn get_revid(page: &Page, bot: &Bot) -> Option<u64> {
    let resp = bot.parsoid().get(page.title()).await.ok()?;
    resp.revision_id()
}
