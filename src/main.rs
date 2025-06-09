mod config;
mod parser;
pub mod schema;
mod convert;

use std::{time::Duration};

use anyhow::{bail, Context};
use log::{debug, error, info, warn, LevelFilter};
use mwbot::{
    generators::{CategoryMemberSort, CategoryMembers, Generator}, Bot, Page,
    SaveOptions,
};
use tokio::time::sleep;
use crate::{
    config::Config,
    parser::{call_parser, Template},
};
use crate::convert::gen_graph_chart;

pub const TAB_EXT: &str = ".tab";
pub const CHART_EXT: &str = ".chart";

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

struct Swap {
    from: String,
    to: String,
}

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
                    "population (million)" => {
                        name = Some(format!("{country} Total Population"));
                        if !parsed.params.contains_key("title") {
                            parsed.params.insert("title".to_string(), Some(format!("{country} Population")));
                        }
                    }
                    "natural change (per 1000)" => {
                        name = Some(format!("{country} Population Change"));
                    }
                    "natural growth" => {
                        name = Some(format!("{country} Natural Growth"));
                    }
                    "infant mortality (per 1000 live births)" | "infant mortality (per 1000 births)" => {
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

async fn run_on_page(
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

    let mut swaps = vec![];
    let mut errors = vec![];
    for parsed in p.templates {
        match handle_template(commons_bot, parsed, page.title()).await {
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

fn init_logging(_config: &Config) {
    let stdout = log4rs::append::console::ConsoleAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
            // Colorize the output
            "[{h({l})} {M} {d(%Y-%m-%d %H:%M:%S)}] {m}{n}",
        )))
        .build();
    // File with the rest
    let file = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
            "[{l} {M} {d(%Y-%m-%d %H:%M:%S)}] {m}{n}",
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

    // TODO: impl
    // let mut failed_revs = HashMap::new();

    loop {
        check_shutdown(&commons_bot, "https://commons.wikimedia.org/", &config).await?;
        check_shutdown(&wiki_bot, &config.wiki, &config).await?;
        // Get the list of articles to process
        let generator =
            CategoryMembers::new(&config.search_category).sort(CategoryMemberSort::Timestamp);
        let mut output = generator.generate(&wiki_bot);
        if let Some(Ok(o)) = output.recv().await {
            let title = o.title().to_string();
            match run_on_page(o, &commons_bot, &wiki_bot, &config).await {
                Ok(_) => {
                    debug!("Successfully processed page: {title}");
                }
                Err(error) => {
                    error!("Error processing page {title}: {error}");
                }
            }
        } else {
            info!("No articles found in {}", config.search_category);
            sleep(Duration::from_secs(9)).await;
        }
        sleep(Duration::from_secs(1)).await;
    }
}
