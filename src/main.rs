mod api_utils;
mod failed_revs;
mod graph_task;
mod parser;
mod rev_info;
mod rfd_task;
mod server;

use std::sync::Arc;

use anyhow::Context;
use graphbot_config::Config;
use mwbot::Bot;
use tokio::{join, sync::RwLock, task};
use tracing::error;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt};

pub const TAB_EXT: &str = ".tab";
pub const CHART_EXT: &str = ".chart";

async fn check_shutdown(bot: &Bot, wiki: &str, config: &RwLock<Config>) -> anyhow::Result<bool> {
    let config = config.read().await;
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

fn init_logging(_config: &RwLock<Config>) -> non_blocking::WorkerGuard {
    let file_appender = rolling::hourly("logs/", "graphbot.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::Layer::new()
        .with_ansi(true)
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::new("info").add_directive("sqlx::query=warn".parse().unwrap()));

    let file_layer = fmt::Layer::new()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(
            EnvFilter::new("trace")
                .add_directive("hyper=info".parse().unwrap())
                .add_directive("h2=info".parse().unwrap())
                .add_directive("mwbot=debug".parse().unwrap())
                .add_directive("parsoid=debug".parse().unwrap())
                .add_directive("sqlx::query=info".parse().unwrap()),
        );

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
    guard
}

const COMMONS_BASE_URL: &str = "https://commons.wikimedia.org/";
const COMMONS_API_URL: &str = "https://commons.wikimedia.org/w/";

const USER_AGENT: &str = "GraphBot/1";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = RwLock::new(Config::load()?);
    let url = url::Url::parse(&config.read().await.wiki)?;
    let api_url = url.join("w/")?;
    // Read the config file
    let _guard = init_logging(&config);
    let token = config.read().await.access_token.clone();
    let init_bots_span = tracing::debug_span!("init_bots").entered();
    let wiki_bot_future = Bot::builder(api_url.to_string())
        .set_user_agent(USER_AGENT.to_string())
        .set_mark_as_bot(true)
        .set_oauth2_token(config.read().await.username.clone(), token.clone())
        .build();

    let commons_bot_future = Bot::builder(COMMONS_API_URL.to_string())
        .set_user_agent(USER_AGENT.to_string())
        .set_mark_as_bot(true)
        .set_oauth2_token(config.read().await.username.clone(), token)
        .build();
    let (wiki_bot, commons_bot) = join!(wiki_bot_future, commons_bot_future);
    let wiki_bot = wiki_bot.context("Failed to initialize wiki bot")?;
    let commons_bot = commons_bot.context("Failed to initialize commons bot")?;
    init_bots_span.exit();

    let wiki_bot = Arc::new(wiki_bot);
    let commons_bot = Arc::new(commons_bot);
    let config = Arc::new(config);

    let _server_task = task::spawn({
        let config = Arc::clone(&config);
        async move {
            if let Err(e) = server::run(config).await {
                error!("Server failed to run: {e}");
            }
        }
    });

    let graph_task = task::spawn({
        let wiki_bot = Arc::clone(&wiki_bot);
        let commons_bot = Arc::clone(&commons_bot);
        let config = Arc::clone(&config);
        async move {
            if let Err(e) = graph_task::graph_task(commons_bot, wiki_bot, config).await {
                error!("Graph task failed: {e}");
            }
        }
    });
    let rfd_task = task::spawn({
        let wiki_bot = Arc::clone(&wiki_bot);
        let config = Arc::clone(&config);
        async move {
            if let Err(e) = rfd_task::rfd_task(wiki_bot, config).await {
                error!("RFD task failed: {e}");
            }
        }
    });
    let shutdown_task: task::JoinHandle<anyhow::Result<()>> = task::spawn({
        let wiki_bot = Arc::clone(&wiki_bot);
        let commons_bot = Arc::clone(&commons_bot);
        let config = Arc::clone(&config);
        async move {
            loop {
                if check_shutdown(&wiki_bot, &config.read().await.wiki, &config).await? {
                    break;
                }
                if check_shutdown(&commons_bot, COMMONS_BASE_URL, &config).await? {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
            tracing::info!("Shutting down gracefully...");
            config.write().await.shutdown_graph_task = true;
            config.write().await.shutdown_rfd_task = true;
            Ok(())
        }
    });
    let _ = join!(
        graph_task,
        rfd_task,
        shutdown_task
    );
    Ok(())
}
