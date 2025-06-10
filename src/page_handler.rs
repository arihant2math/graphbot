use std::sync::Arc;

use anyhow::{Context, bail};
use mwbot::{Bot, Page, SaveOptions};
use tokio::sync::{Mutex, mpsc, oneshot};
use tracing::{error, info, warn};

use crate::{
    CHART_EXT, TAB_EXT,
    config::Config,
    convert::gen_graph_chart,
    parser::{Template, call_parser},
};

struct Swap {
    from: String,
    to: String,
}

#[tracing::instrument(level = "trace", skip(bot, template))]
async fn create_pages(bot: &Bot, template: &Template, name: &str) -> anyhow::Result<String> {
    let file_name = name.replace(' ', "_");
    let tab_file_name = format!("Data:{file_name}{TAB_EXT}");
    let chart_file_name = format!("Data:{file_name}{CHART_EXT}");
    let mut modded_template = template.clone();
    // width is handled separately
    modded_template.params.remove("width");
    let out = gen_graph_chart(name, &modded_template.params)?;

    // Save the tab and chart files
    let tab_text = serde_json::to_string_pretty(&out.tab)?;
    let tab_file_page = bot.page(&tab_file_name)?;
    if !tab_file_page.exists().await? {
        match tab_file_page
            .save(
                tab_text,
                &SaveOptions::summary("GraphPort: Create tab file").mark_as_bot(true),
            )
            .await
        {
            Ok(_) => info!("Tab file {tab_file_name} created successfully."),
            Err(e) => {
                error!("Failed to create tab file {tab_file_name}: {e}");
                warn!("Please create the tab file manually.");
                warn!(
                    "Tab file content ({tab_file_name}):\n{}",
                    serde_json::to_string_pretty(&out.tab)?
                );
                return Err(e.into());
            }
        }
    } else {
        warn!("Tab file {tab_file_name} already exists, skipping creation.");
    }
    let chart_text = serde_json::to_string_pretty(&out.chart)?;
    let chart_file_page = bot.page(&chart_file_name)?;
    if !chart_file_page.exists().await? {
        match chart_file_page
            .save(
                chart_text,
                &SaveOptions::summary("GraphPort: Create chart file").mark_as_bot(true),
            )
            .await
        {
            Ok(_) => info!("Chart file {chart_file_name} created successfully."),
            Err(e) => {
                error!("Failed to create chart file {chart_file_name}: {e}");
                warn!("Please create the chart file manually.");
                warn!(
                    "Chart file content ({chart_file_name}):\n{}",
                    serde_json::to_string_pretty(&out.chart)?
                );
                return Err(e.into());
            }
        }
    } else {
        warn!("Chart file {chart_file_name} already exists, skipping creation.");
    }
    info!("Successfully created tab and chart files for {name}.");
    let inside = if let Some(width) = template.params.get("width").cloned().flatten() {
        format!("ChartDisplay|definition={name}{CHART_EXT}|data={name}{TAB_EXT}|Width={width}")
    } else {
        format!("ChartDisplay|definition={name}{CHART_EXT}|data={name}{TAB_EXT}")
    };
    Ok(format!("{}{inside}{}", "{{", "}}"))
}

#[tracing::instrument(skip(bot, parsed))]
async fn handle_template(bot: &Bot, parsed: Template, title: &str) -> anyhow::Result<Option<Swap>> {
    let mut parsed = parsed;
    match &*parsed.name {
        "PortGraph" => {
            let mut name = parsed.params.get("name").cloned().flatten();

            // Special handling for demographics related pages
            if name.is_none() && title.starts_with("Demographics of ") {
                // now we need to extract the name from the title
                let country = title.trim_start_matches("Demographics of").trim();
                let country = country.trim_start_matches("the").trim();
                if country.is_empty() {
                    bail!("Country name empty, unreachable");
                }
                match &*parsed.params.get("y1Title").cloned().flatten().ok_or_else(|| {
                    anyhow::anyhow!("'y1Title' parameter is required for PortGraph on demographics pages without template graph name")
                })?.to_ascii_lowercase() {
                    s if s.starts_with("population") => {
                        name = Some(format!("{country} Total Population"));
                        if !parsed.params.contains_key("title") {
                            parsed.params.insert("title".to_string(), Some(format!("{country} Population")));
                        }
                    }
                    s if s.starts_with("natural change") => {
                        name = Some(format!("{country} Population Change"));
                    }
                    "natural growth" => {
                        name = Some(format!("{country} Natural Growth"));
                    }
                    s if s.starts_with("infant mortality") => {
                        name = Some(format!("{country} Infant Mortality"));
                    }
                    "total fertility rate" | "tfr" => {
                        name = Some(format!("{country} TFR"));
                        if !parsed.params.contains_key("title") {
                            parsed.params.insert("title".to_string(), Some("Total Fertility Rate".to_string()));
                        }
                    }
                    _ => {
                        bail!("Unsupported y1Title for demographics page: {}", parsed.params.get("y1Title").cloned().flatten().unwrap_or_default());
                    }
                }
            }

            let name = name.ok_or_else(|| {
                anyhow::anyhow!("'name' parameter is required for ConvertGraphChart")
            })?;
            let swap = create_pages(bot, &parsed, &name)
                .await
                .context("Failed to generate/create pages")?;
            Ok(Some(Swap {
                from: parsed.wikitext,
                to: swap,
            }))
        }
        _ => Ok(None),
    }
}

#[tracing::instrument(skip_all)]
pub async fn run_on_page(
    page: Page,
    commons_bot: &Bot,
    _wiki_bot: &Bot,
    config: &Config,
) -> anyhow::Result<()> {
    info!("Processing page: {}", page.title());
    // Download the article
    let content_future = page.wikitext();
    // Delete in.txt and out.json if they exist
    let rm_future = async {
        if tokio::fs::remove_file("in.txt").await.is_err() {
            // File didn't exist, ignore
        }
        if tokio::fs::remove_file("out.json").await.is_err() {
            // File didn't exist, ignore
        }
    };
    let (content, _) = tokio::join!(content_future, rm_future);
    let content = content.context("Failed to get wikitext")?;
    let p = call_parser(&content, config)?;

    let mut tasks = vec![];
    for parsed in p.templates {
        tasks.push(async { handle_template(commons_bot, parsed, page.title()).await });
    }
    let task_results = futures::future::join_all(tasks).await;

    let mut swaps = vec![];
    let mut errors = vec![];
    for result in task_results {
        match result {
            Ok(s) => {
                if let Some(swap) = s {
                    swaps.push(swap);
                }
            }
            Err(error) => {
                error!("Error handling template: {error}");
                errors.push(error);
            }
        }
    }
    let mut modified_wikitext = content.clone();
    for swap in swaps {
        if modified_wikitext.contains(&swap.from) {
            modified_wikitext = modified_wikitext.replace(&swap.from, &swap.to);
        } else {
            warn!("Template {} not found in page {}", swap.from, page.title());
        }
    }
    // Save the modified wikitext back to the page
    let title = page.title().to_string();
    if modified_wikitext != content {
        let save_options = SaveOptions::summary("Port graphs to charts").mark_as_bot(true);
        match page.save(modified_wikitext, &save_options).await {
            Ok(_) => info!("Successfully updated page {title}"),
            Err(e) => {
                error!("Failed to update page {title}: {e}");
                bail!("Failed to update page {title}");
            }
        }
    } else {
        info!("No changes made to page {title}");
    }
    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Errors occurred while processing page {title}: {errors:?}"
        ));
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn page_handler(
    rx: Arc<Mutex<mpsc::Receiver<(Page, oneshot::Sender<anyhow::Result<()>>)>>>,
    commons_bot: Arc<Bot>,
    wiki_bot: Arc<Bot>,
    config: Arc<Config>,
) {
    while let Some((page, result_handler)) = rx.lock().await.recv().await {
        if tokio::fs::metadata("shutdown.txt").await.is_ok() {
            info!("Shutdown file found, exiting.");
            break;
        }
        result_handler
            .send(run_on_page(page, &commons_bot, &wiki_bot, &config).await)
            .unwrap();
    }
}
