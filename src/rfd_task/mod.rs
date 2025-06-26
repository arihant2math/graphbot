use std::sync::Arc;

use mwbot::Bot;
use tokio::sync::RwLock;

use crate::config::Config;

pub async fn rfd_task(
    commons_bot: Arc<Bot>,
    wiki_bot: Arc<Bot>,
    config: Arc<RwLock<Config>>,
) -> anyhow::Result<()> {
    Ok(())
}
