mod convert;

use crate::convert::handle_graph_chart;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(clap::Parser)]
struct Args {
    /// Article
    article: String,
    /// Which wiki to use
    #[clap(long, default_value = "en.wikipedia.org")]
    wiki: String,
}

#[derive(Serialize, Deserialize)]
struct Template {
    pub name: String,
    pub params: HashMap<String, Option<String>>,
}

#[derive(Serialize, Deserialize)]
struct OutRoot {
    pub data: Vec<Template>,
}

fn build_url(article: &str, wiki: &str) -> String {
    // Replace spaces with underscores
    let article = article.replace(' ', "_");
    let mut url = url::Url::parse(wiki).unwrap();
    url.set_path(&format!("w/index.php?action=raw&title={}", article));
    url.to_string()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut url = url::Url::parse(&args.wiki).unwrap();
    let api_url = url.join("w/api.php").unwrap();
    let rest_url = url.join("api/rest_v1").unwrap();
    let bot = mwbot::Bot::builder(api_url.to_string(), rest_url.to_string()).set_user_agent("GraphPort/1".to_string()).build().await.unwrap();
    // Download the article
    let page = bot.page(&args.article).expect("Failed to get page");
    let content = page.wikitext().await.unwrap();
    // Write the content to in.txt
    std::fs::write("in.txt", content).expect("Failed to write in.txt");
    // Call main.py
    let output = std::process::Command::new("uv")
        .arg("run")
        .arg("main.py")
        .output()
        .expect("Failed to execute main.py");
    if !output.status.success() {
        eprintln!("Error running main.py: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    // Read out.json
    let input = std::fs::read_to_string("out.json").expect("Failed to read out.json");
    let templates: OutRoot = serde_json::from_str(&input).expect("Failed to parse JSON");
    for parsed in templates.data {
        match &*parsed.name {
            "Graph:Chart" | "GraphChart" => {
                let name = inquire::Text::new("Enter chart name:").prompt().unwrap();
                handle_graph_chart(name, parsed.params);
            }
            _ => {
                // println!("Unknown tag: {}", parsed.name);
            }
        }
    }
}
