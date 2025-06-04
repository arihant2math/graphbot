mod config;
mod convert;
mod parser;
pub mod schema;

use crate::config::Config;
use crate::convert::handle_graph_chart;
use crate::parser::call_parser;
use anyhow::{Context, bail};
use clap::Parser;
use log::{LevelFilter, debug, error, info, warn};
use mwbot::generators::{CategoryMemberSort, CategoryMembers, Generator};
use mwbot::{Bot, Page, SaveOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

pub const TAB_EXT: &str = ".tab";
pub const CHART_EXT: &str = ".chart";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Template {
    pub name: String,
    pub params: HashMap<String, Option<String>>,
    pub wikitext: String,
}

#[derive(Serialize, Deserialize)]
struct OutRoot {
    pub data: Vec<Template>,
}

async fn create_pages(bot: &Bot, template: &Template, name: &str) -> anyhow::Result<String> {
    let file_name = name.replace(" ", "_");
    let tab_file_name = format!("Data:{}{TAB_EXT}", file_name);
    let chart_file_name = format!("Data:{}{CHART_EXT}", file_name);
    let mut modded_template = template.clone();
    // width is handled separately
    modded_template.params.remove("width");
    let out = handle_graph_chart(name.to_string(), modded_template.params);

    // Save the tab and chart files
    let tab_text = serde_json::to_string_pretty(&out.tab)?;
    let tab_file_page = bot.page(&format!("{tab_file_name}"))?;
    if !tab_file_page.exists().await? {
        match tab_file_page
            .save(
                tab_text,
                &SaveOptions::summary("GraphPort: Create tab file").mark_as_bot(true),
            )
            .await
        {
            Ok(_) => info!("Tab file {} created successfully.", tab_file_name),
            Err(e) => {
                error!("Failed to create tab file {}: {}", tab_file_name, e);
                warn!("Please create the tab file manually.");
                warn!(
                    "Tab file content ({tab_file_name}):\n{}",
                    serde_json::to_string_pretty(&out.tab)?
                );
                return Err(e.into());
            }
        }
    } else {
        warn!(
            "Tab file {} already exists, skipping creation.",
            tab_file_name
        );
    }
    let chart_text = serde_json::to_string_pretty(&out.chart)?;
    let chart_file_page = bot.page(&format!("{chart_file_name}"))?;
    if !chart_file_page.exists().await? {
        match chart_file_page
            .save(
                chart_text,
                &SaveOptions::summary("GraphPort: Create chart file").mark_as_bot(true),
            )
            .await
        {
            Ok(_) => info!("Chart file {} created successfully.", chart_file_name),
            Err(e) => {
                error!("Failed to create chart file {}: {}", chart_file_name, e);
                warn!("Please create the chart file manually.");
                warn!(
                    "Chart file content ({chart_file_name}):\n{}",
                    serde_json::to_string_pretty(&out.chart)?
                );
                return Err(e.into());
            }
        }
    } else {
        warn!(
            "Chart file {} already exists, skipping creation.",
            chart_file_name
        );
    }
    info!("Successfully created tab and chart files for {}.", name);
    let inside = if let Some(width) = template.params.get("width").cloned().flatten() {
        format!("ChartDisplay|Definition={name}{CHART_EXT}|Data={name}{TAB_EXT}|Width={width}")
    } else {
        format!("ChartDisplay|Definition={name}{CHART_EXT}|Data={name}{TAB_EXT}")
    };
    Ok(format!("{}{inside}{}", "{{", "}}"))
}

struct Swap {
    from: String,
    to: String,
}

async fn handle_template(bot: &Bot, parsed: Template) -> anyhow::Result<Option<Swap>> {
    match &*parsed.name {
        "PortGraph" => {
            let name = parsed
                .params
                .get("name")
                .cloned()
                .flatten()
                .ok_or_else(|| {
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

async fn check_shutdown(bot: &Bot, wiki: &str, config: &Config) -> anyhow::Result<()> {
    if bot
        .page(&format!("User:{}/Shutdown", config.username))?
        .exists()
        .await?
    {
        error!(
            "This bot has been shut down on {}. Please do not use it.",
            wiki
        );
        std::process::exit(1);
    }
    Ok(())
}

async fn run_on_page(page: Page, commons_bot: &Bot, _wiki_bot: &Bot) -> anyhow::Result<()> {
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
    let content = content.expect("Failed to get wikitext");
    let input = call_parser(&content)?;
    let templates: OutRoot = serde_json::from_str(&input).expect("Failed to parse JSON");
    let mut swaps = vec![];
    for parsed in templates.data {
        match handle_template(&commons_bot, parsed).await {
            Ok(s) => {
                if let Some(swap) = s {
                    swaps.push(swap);
                }
            }
            Err(e) => error!("Error handling template: {}", e),
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
    if modified_wikitext != content {
        let save_options = SaveOptions::summary("Port graphs to charts").mark_as_bot(true);
        let title = page.title().to_string();
        match page.save(modified_wikitext, &save_options).await {
            Ok(_) => info!("Successfully updated page {}", title),
            Err(e) => {
                error!("Failed to update page {}: {}", title, e);
                bail!("Failed to update page {}", title);
            }
        }
    } else {
        info!("No changes made to page {}", page.title());
    }
    Ok(())
}

fn init_logging(_config: &Config) {
    let stdout = log4rs::append::console::ConsoleAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
            // Colorize the output
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}{n}",
        )))
        .build();
    // File with the rest
    let file = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}{n}",
        )))
        .build("graphport.log")
        .expect("Failed to create file appender");
    let config = log4rs::Config::builder()
        .appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout)))
        .appender(log4rs::config::Appender::builder().build("file", Box::new(file)))
        .logger(log4rs::config::Logger::builder().build("graphport", LevelFilter::Info))
        .build(
            log4rs::config::Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .expect("Failed to create log4rs config");
    log4rs::init_config(config).expect("Failed to initialize logging");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let url = url::Url::parse(&config.wiki)?;
    let api_url = url.join("w/api.php")?;
    let rest_url = url.join("api/rest_v1")?;
    // Read the config file
    init_logging(&config);
    let token = config.access_token.clone();
    let wiki_bot = Bot::builder(api_url.to_string(), rest_url.to_string())
        .set_user_agent("GraphPort/1".to_string())
        .set_mark_as_bot(true)
        .set_oauth2_token(config.username.clone(), token.clone())
        .build()
        .await?;
    let commons_bot = Bot::builder(
        "https://commons.wikimedia.org/w/api.php".to_string(),
        "https://commons.wikimedia.org/api/rest_v1".to_string(),
    )
    .set_user_agent("GraphPort/1".to_string())
    .set_mark_as_bot(true)
    .set_oauth2_token(config.username.clone(), token)
    .build()
    .await?;

    loop {
        check_shutdown(&commons_bot, "https://commons.wikimedia.org/", &config).await?;
        check_shutdown(&wiki_bot, &config.wiki, &config).await?;
        // Get the list of articles to process
        let generator =
            CategoryMembers::new(&config.search_category).sort(CategoryMemberSort::Timestamp);
        let mut output = generator.generate(&wiki_bot);
        if let Some(Ok(o)) = output.recv().await {
            let title = o.title().to_string();
            match run_on_page(o, &commons_bot, &wiki_bot).await {
                Ok(_) => debug!("Successfully processed page: {title}"),
                Err(e) => error!("Error processing page {title}: {}", e),
            }
            sleep(Duration::from_secs(9)).await;
        } else {
            info!("No articles found in {}", config.search_category);
        }
        sleep(Duration::from_secs(1)).await;
    }
}
