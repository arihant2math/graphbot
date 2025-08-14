use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use mwbot::{Bot, SaveOptions};
use sea_orm::{Database, EntityTrait};
use tokio::sync::RwLock;
use tracing::info;
use graphbot_db::graph_failed_conversions;
use crate::config::Config;

const GRAPH_ERRORS_WIKI_PAGE: &str = "User:GraphBot/Conversion Errors";

fn generate_wikitext(errors: Vec<graph_failed_conversions::Model>) -> String {
    let prelude = "This table lists the pages with graphs that GraphBot could not convert to charts, along with the error messages.\n\n";
    let mut text = String::from(prelude);
    // {| class="wikitable sortable collapsible" border="1"
    // |+ Sortable and collapsible table
    // |-
    // ! scope="col" | Alphabetic
    // ! scope="col" | Numeric
    // ! scope="col" | Date
    // ! scope="col" class="unsortable" | Unsortable
    // |-
    // | d || 20 || 2008-11-24 || This
    // |-
    // | b || 8 || 2004-03-01 || column
    // |-
    // | a || 6 || 1979-07-23 || cannot
    // |-
    // | c || 4 || 1492-12-08 || be
    // |-
    // | e || 0 || 1601-08-13 || sorted.
    // |}
    text.push_str("{| class=\"wikitable sortable collapsible\" border=\"1\"\n");
    text.push_str("|+ Conversion Errors\n");
    text.push_str("|-\n");
    text.push_str("! scope=\"col\" | Page Title\n");
    text.push_str("! scope=\"col\" | Revision ID\n");
    text.push_str("! scope=\"col\" | Error Message\n");
    text.push_str("! scope=\"col\" | Date (UTC)\n");
    for error in errors {
        text.push_str("|-\n");
        text.push_str(&format!("| {} || {} || <nowiki>{}</nowiki> || {}\n",
                               error.page_title,
                               error.rev_id,
                               error.error.as_deref().unwrap_or("No error message"),
                               error.date.to_rfc3339(),
        ));
    }
    text.push_str("|}\n");
    text
}

pub async fn report_graph_errors_task(wiki_bot: Arc<Bot>,
                                      config: Arc<RwLock<Config>>,
) -> anyhow::Result<()> {
    while !Path::new("db/graph.db").exists() {
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    let db_url = config.read().await.graph_task.db_url.clone();
    let db = Database::connect(&db_url).await?;
    info!("Starting Report Graph Port Errors task");
    loop {
        if config.read().await.shutdown_graph_task {
            info!("Shutdown flag is set, exiting.");
            break;
        }
        if config.read().await.pause_graph_task {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        info!("Reporting conversion errors to wiki ...");
        // TODO: Paginate this to reduce memory usage
        let errors = graphbot_db::prelude::GraphFailedConversions::find().all(&db).await?;
        let text = generate_wikitext(errors);
        let page = wiki_bot
            .page(GRAPH_ERRORS_WIKI_PAGE)
            .map_err(|e| anyhow::anyhow!("Failed to get page: {}", e))?;
        let old_text = page.wikitext().await.map_err(|e| anyhow::anyhow!("Failed to get page text: {}", e))?;
        if old_text == text {
            info!("No changes to report, skipping update.");
        } else {
            page.save(text, &SaveOptions::summary("Updating conversion errors"))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save page: {}", e))?;
            info!("Conversion errors reported successfully.");
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
    Ok(())
}