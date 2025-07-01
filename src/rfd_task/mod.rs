use std::sync::Arc;
use std::time::Duration;
use mwbot::Bot;
use tokio::sync::RwLock;
use tracing::info;
use crate::config::Config;
use crate::parser;

const RFD_PAGE: &str = "Wikipedia:Redirects for discussion";

pub async fn rfd_task(
    wiki_bot: Arc<Bot>,
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
        let page = wiki_bot.page(RFD_PAGE)?;
        let parsed = parser::call_parser(&page.wikitext().await?, &config).await?;
        let mut rfd_pages = Vec::with_capacity(10);
        for template in parsed.parsed.templates {
            if template.name_str().starts_with("Wikipedia:Redirects for discussion/Log") {
                rfd_pages.push(template.name_str().to_string());
            }
        }
        for rfd_page in rfd_pages {
            info!("Processing RFD page: {}", rfd_page);
            let rfd_page_obj = wiki_bot.page(&rfd_page)?;
            let rfd_text = rfd_page_obj.wikitext().await?;
            let parsed_rfd = parser::call_parser(&rfd_text, &config).await?;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
