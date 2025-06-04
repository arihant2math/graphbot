mod convert;
pub mod schema;

use crate::convert::handle_graph_chart;
use anyhow::Context;
use clap::Parser;
use mwbot::{Bot, SaveOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const TAB_EXT: &str = ".tab";
pub const CHART_EXT: &str = ".chart";

#[derive(clap::Parser)]
struct Args {
    /// Article
    article: String,
    /// Which wiki to use
    #[clap(long, default_value = "https://en.wikipedia.org/")]
    wiki: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Template {
    pub name: String,
    pub params: HashMap<String, Option<String>>,
    pub wikitext: String
}

#[derive(Serialize, Deserialize)]
struct OutRoot {
    pub data: Vec<Template>,
}

const BOT_NAME: &str = "GraphBot";

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    access_token: String,
    client_secret: String,
    client_id: String,
}

async fn create_pages(bot: &Bot, template: Template, name: &str) -> anyhow::Result<()> {
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
            Ok(_) => println!("Tab file {} created successfully.", tab_file_name),
            Err(e) => {
                eprintln!("Failed to create tab file {}: {}", tab_file_name, e);
                println!("Please create the tab file manually.");
                println!(
                    "Tab file content ({tab_file_name}):\n{}",
                    serde_json::to_string_pretty(&out.tab)?
                );
            }
        }
    } else {
        println!(
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
            Ok(_) => println!("Chart file {} created successfully.", chart_file_name),
            Err(e) => {
                eprintln!("Failed to create chart file {}: {}", chart_file_name, e);
                println!("Please create the chart file manually.");
                println!(
                    "Chart file content ({chart_file_name}):\n{}",
                    serde_json::to_string_pretty(&out.chart)?
                );
            }
        }
    } else {
        println!(
            "Chart file {} already exists, skipping creation.",
            chart_file_name
        );
    }
    println!("Successfully created tab and chart files for {}.", name);
    let inside = if let Some(width) = template.params.get("width").cloned().flatten() {
        format!("ChartDisplay|Definition={name}{CHART_EXT}|Width={width}")
    } else {
        format!("ChartDisplay|Definition={name}{CHART_EXT}")
    };
    println!("Usage: {}{inside}{}", "{{", "}}");

    Ok(())
}

async fn handle_template(bot: &Bot, parsed: Template, args: &Args) -> anyhow::Result<()> {
    match &*parsed.name {
        "Graph:Chart" | "GraphChart" => {
            let name = if args.article.starts_with("Demographics") && parsed.params.get("yAxisTitle").cloned().flatten().is_some() {
                let country = args.article.clone().split(" ").last().unwrap().to_string();
                let y_axis_title = parsed.params.get("yAxisTitle").cloned().flatten().unwrap_or_default();
                let y1_title = parsed.params.get("y1Title").cloned().flatten().unwrap_or_default();

                let default = if y1_title == "Total Fertility Rate" && y_axis_title == "TFR" {
                    Some("TFR")
                } else if y1_title.starts_with("population") {
                    Some("Total Population")
                } else if y1_title == "Natural change (per 1000)" {
                    Some("Population Change")
                } else if y1_title == "Infant Mortality (per 1000 live births)" {
                    Some("Infant Mortality Rate")
                } else {
                    None
                };
                if let Some(default) = default {
                    let text = format!("{country} {default}");
                    inquire::Text::new("Enter chart name:").with_default(&text).prompt()?
                } else {
                    inquire::Text::new("Enter chart name:").prompt()?
                }
            } else {
                inquire::Text::new("Enter chart name:").prompt()?
            };
            let confirm = inquire::Confirm::new(&format!("Create chart with name {name}?"))
                .with_default(true)
                .prompt()?;
            if !confirm {
                println!("Chart creation cancelled.");
                return Ok(());
            }
            create_pages(bot, parsed, &name)
                .await
                .context("Failed to generate/create pages")?;
        }
        "ConvertGraphChart" => {
            let name = parsed.params.get("name").cloned().flatten().ok_or_else(|| {
                anyhow::anyhow!("'name' parameter is required for ConvertGraphChart")
            })?;
            create_pages(bot, parsed, &name)
                .await
                .context("Failed to generate/create pages")?;
        }
        _ => {
            // println!("Unknown tag: {}", parsed.name);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let url = url::Url::parse(&args.wiki)?;
    let api_url = url.join("w/api.php")?;
    let rest_url = url.join("api/rest_v1")?;
    // Read the config file
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").expect("Failed to read config.toml"),
    )?;
    let token = config.access_token;
    let wiki_bot = Bot::builder(api_url.to_string(), rest_url.to_string())
        .set_user_agent("GraphPort/1".to_string())
        .set_mark_as_bot(true)
        .set_oauth2_token(BOT_NAME.to_string(), token.clone())
        .build()
        .await?;
    let commons_bot = Bot::builder(
        "https://commons.wikimedia.org/w/api.php".to_string(),
        "https://commons.wikimedia.org/api/rest_v1".to_string(),
    )
        .set_user_agent("GraphPort/1".to_string())
        .set_mark_as_bot(true)
        .set_oauth2_token(BOT_NAME.to_string(), token)
        .build()
        .await?;
    if wiki_bot.page(&format!("User:{}/Shutdown", BOT_NAME))?
        .exists()
        .await?
    {
        eprintln!("This bot has been shut down on {}. Please do not use it.", args.wiki);
        std::process::exit(1);
    }
    if commons_bot.page(&format!("User:{}/Shutdown", BOT_NAME))?
        .exists()
        .await?
    {
        eprintln!("This bot has been shut down on Commons. Please do not use it.");
        std::process::exit(1);
    }

    // Download the article
    let page = wiki_bot.page(&args.article).expect("Failed to get page");
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
    // Write the content to in.txt
    std::fs::write("in.txt", content).expect("Failed to write in.txt");
    // Call main.py
    let output = std::process::Command::new("uv")
        .arg("run")
        .arg("main.py")
        .output()
        .expect("Failed to execute main.py");
    if !output.status.success() {
        eprintln!(
            "Error running main.py:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }
    // Read out.json
    let input = std::fs::read_to_string("out.json").expect("Failed to read out.json");
    let templates: OutRoot = serde_json::from_str(&input).expect("Failed to parse JSON");
    for parsed in templates.data {
        match handle_template(&commons_bot, parsed, &args).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error handling template: {}", e),
        }
    }
    Ok(())
}
