use std::sync::Arc;
use std::time::Duration;
use mwbot::Bot;
use tokio::sync::RwLock;
use tracing::info;
use crate::config::Config;

const RFD_PAGE: &str = "Wikipedia:Redirects for discussion";

pub async fn rfd_task(
    _wiki_bot: Arc<Bot>,
    config: Arc<RwLock<Config>>,
) -> anyhow::Result<()> {
    loop {
        if config.read().await.shutdown_rfd_task {
            info!("Shutdown flag is set, exiting.");
            break;
        }
        if config.read().await.pause_rfd_task {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
