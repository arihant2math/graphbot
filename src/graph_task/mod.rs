use std::{sync::Arc, time::Duration};

use anyhow::{Context, bail};
use convert::{ConversionOutput, gen_graph_chart};
use graphbot_config::Config;
use mwbot::{
    Bot, Page, SaveOptions,
    generators::{
        Generator,
        categories::{CategoryMemberSort, CategoryMembers},
    },
};
use serde::Deserialize;
use tokio::{
    sync::{Mutex, RwLock, mpsc, mpsc::Receiver, oneshot, oneshot::Sender},
    task,
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, error, info, trace, warn};

use crate::{
    CHART_EXT, TAB_EXT, api_utils,
    failed_revs::FailedRevs,
    parser::{Node, NodeInnerTemplate, call_parser},
    rev_info::RevInfo,
};

mod convert;
pub mod schema;

type PageRequest = (Page, Option<RevInfo>, Sender<anyhow::Result<()>>);

struct Swap {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
struct ExternalTabDocument {
    schema: ExternalTabSchema,
}

#[derive(Debug, Deserialize)]
struct ExternalTabSchema {
    fields: Vec<ExternalTabField>,
}

#[derive(Debug, Deserialize)]
struct ExternalTabField {
    name: String,
    #[serde(rename = "type")]
    field_type: String,
}

fn validate_external_tab_schema(
    content: &str,
    tab_file_name: &str,
    x_field: Option<&str>,
) -> anyhow::Result<()> {
    let document: ExternalTabDocument = serde_json::from_str(content).with_context(|| {
        format!("External table Data:{tab_file_name} is not valid tabular JSON")
    })?;
    let fields = &document.schema.fields;
    if fields.len() < 2 {
        bail!(
            "External table Data:{tab_file_name} must contain an x field and at least one y field"
        );
    }

    if let Some(x_field) = x_field.map(str::trim).filter(|field| !field.is_empty())
        && fields[0].name != x_field
    {
        bail!(
            "External table Data:{tab_file_name} uses '{}' as its first field, but the graph requests x='{x_field}'",
            fields[0].name
        );
    }

    let non_numeric_fields: Vec<_> = fields[1..]
        .iter()
        .filter(|field| field.field_type != "number")
        .map(|field| format!("{} ({})", field.name, field.field_type))
        .collect();
    if !non_numeric_fields.is_empty() {
        bail!(
            "External table Data:{tab_file_name} has non-numeric y fields: {}",
            non_numeric_fields.join(", ")
        );
    }
    Ok(())
}

async fn load_external_tab(
    bot: &Bot,
    tab_file_name: &str,
    x_field: Option<&str>,
) -> anyhow::Result<String> {
    let requested_title = format!("Data:{tab_file_name}");
    let page = bot
        .page(&requested_title)
        .with_context(|| format!("Invalid external table title {requested_title}"))?;
    if !page
        .exists()
        .await
        .with_context(|| format!("Failed to check whether {requested_title} exists"))?
    {
        bail!("External table {requested_title} does not exist");
    }
    let content = page
        .wikitext()
        .await
        .with_context(|| format!("Failed to load external table {requested_title}"))?;
    validate_external_tab_schema(&content, tab_file_name, x_field)?;

    let canonical_file_name = page
        .title()
        .strip_prefix("Data:")
        .map(str::to_string)
        .with_context(|| {
            format!(
                "External table resolved outside the Data namespace: {}",
                page.title()
            )
        })?;
    if !canonical_file_name.to_ascii_lowercase().ends_with(TAB_EXT) {
        bail!("External table must resolve to a .tab page: Data:{canonical_file_name}");
    }
    Ok(canonical_file_name)
}

#[tracing::instrument(skip(bot, template), fields(graph_name = name, source_revision = rev_url))]
async fn create_pages(
    bot: &Bot,
    template: &Node<NodeInnerTemplate>,
    name: &str,
    rev_url: &str,
) -> anyhow::Result<String> {
    let file_name = name.replace(' ', "_");
    let chart_file_name = format!("Data:{file_name}{CHART_EXT}");
    let mut modded_template = template.clone();
    // These parameters control the article embedding or target file name, not the chart JSON.
    modded_template.params_remove("width");
    modded_template.params_remove("name");
    let out = gen_graph_chart(name, &modded_template.params_map(), rev_url)
        .with_context(|| format!("Failed to convert graph '{name}'"))?;

    let (mut chart, generated_tab, data_file_name, data_mode) = match out {
        ConversionOutput::GeneratedData { chart, tab } => {
            let data_file_name = chart.source.clone();
            (chart, Some(tab), data_file_name, "generated")
        }
        ConversionOutput::ExistingData {
            chart,
            tab_file_name,
            x_field,
        } => {
            let canonical_file_name = load_external_tab(bot, &tab_file_name, x_field.as_deref())
                .await
                .with_context(|| format!("Failed to use external table Data:{tab_file_name}"))?;
            (chart, None, canonical_file_name, "existing")
        }
    };
    chart.source.clone_from(&data_file_name);
    info!(
        data_mode,
        data_file = %format!("Data:{data_file_name}"),
        chart_file = %chart_file_name,
        "Prepared graph conversion"
    );

    if let Some(tab) = generated_tab {
        let tab_file_name = format!("Data:{data_file_name}");
        let tab_text = serde_json::to_string_pretty(&tab)
            .with_context(|| format!("Failed to serialize {tab_file_name}"))?;
        let tab_file_page = bot.page(&tab_file_name)?;
        if tab_file_page.exists().await? {
            warn!(tab_file = %tab_file_name, "Tab file already exists; skipping creation");
        } else if let Err(error) = tab_file_page
            .save(
                tab_text,
                &SaveOptions::summary(&format!(
                    "GraphBot: Create tab file with data from a {} template. Source: {}",
                    template.name_str(),
                    rev_url
                ))
                .mark_as_bot(true),
            )
            .await
        {
            let error = anyhow::Error::from(error);
            error!(
                tab_file = %tab_file_name,
                error = %format!("{error:#}"),
                "Failed to create tab file"
            );
            return Err(error).with_context(|| format!("Failed to save {tab_file_name}"));
        } else {
            info!(tab_file = %tab_file_name, "Created tab file");
        }
    }

    let chart_text = serde_json::to_string_pretty(&chart)
        .with_context(|| format!("Failed to serialize {chart_file_name}"))?;
    let chart_file_page = bot.page(&chart_file_name)?;
    if chart_file_page.exists().await? {
        warn!(chart_file = %chart_file_name, "Chart file already exists; skipping creation");
    } else if let Err(error) = chart_file_page
        .save(
            chart_text,
            &SaveOptions::summary(&format!(
                "GraphBot: Create chart file with data from a {} template. Source: {}",
                template.name_str(),
                rev_url
            ))
            .mark_as_bot(true),
        )
        .await
    {
        let error = anyhow::Error::from(error);
        error!(
            chart_file = %chart_file_name,
            error = %format!("{error:#}"),
            "Failed to create chart file"
        );
        return Err(error).with_context(|| format!("Failed to save {chart_file_name}"));
    } else {
        info!(chart_file = %chart_file_name, "Created chart file");
    }

    info!(
        data_mode,
        "Successfully prepared chart for article replacement"
    );
    let inside = if let Some(width) = template.params_get("width").flatten() {
        format!("Chart|definition={name}{CHART_EXT}|data={data_file_name}|Width={width}")
    } else {
        format!("Chart|definition={name}{CHART_EXT}|data={data_file_name}")
    };
    Ok(format!("{}{inside}{}", "{{", "}}"))
}

fn trim_comments(s: &str) -> String {
    if s.contains("<!--") && s.contains("-->") {
        let mut new_value = s.to_string();
        while let Some(start) = new_value.find("<!--") {
            if let Some(end) = new_value[start + 4..].find("-->") {
                new_value.replace_range(start..start + 4 + end + 3, "");
            } else {
                break;
            }
        }
        return new_value;
    }
    s.to_string()
}

#[test]
fn test_trim_comments() {
    let s = "This is a test <!-- comment --> string.";
    assert_eq!(trim_comments(s), "This is a test  string.");
    let s = "No comments here.";
    assert_eq!(trim_comments(s), "No comments here.");
    let s = "Multiple <!-- first --> comments <!-- second --> here.";
    assert_eq!(trim_comments(s), "Multiple  comments  here.");
    let s = "Unclosed <!-- comment here.";
    assert_eq!(trim_comments(s), "Unclosed <!-- comment here.");
    let s = "--> No opening comment.";
    assert_eq!(trim_comments(s), "--> No opening comment.");
}

#[test]
fn validates_external_tab_schema() {
    let content = r#"{
        "schema": {
            "fields": [
                {"name": "date", "type": "string"},
                {"name": "highTemp", "type": "number"},
                {"name": "lowTemp", "type": "number"}
            ]
        }
    }"#;

    validate_external_tab_schema(content, "weather/Honolulu.tab", Some("date")).unwrap();
}

#[test]
fn rejects_external_tab_with_wrong_x_field() {
    let content = r#"{
        "schema": {
            "fields": [
                {"name": "date", "type": "string"},
                {"name": "temperature", "type": "number"}
            ]
        }
    }"#;

    let error =
        validate_external_tab_schema(content, "weather/Honolulu.tab", Some("year")).unwrap_err();

    assert!(error.to_string().contains("requests x='year'"));
}

#[tracing::instrument(
    skip(bot, parsed, page, rev_info, config),
    fields(template = %parsed.name_str().trim(), page = %page.title())
)]
async fn handle_template(
    bot: &Bot,
    parsed: Node<NodeInnerTemplate>,
    page: Page,
    rev_info: Option<RevInfo>,
    config: &RwLock<Config>,
) -> anyhow::Result<Option<Swap>> {
    let mut parsed = parsed;
    parsed.params = parsed
        .params
        .clone()
        .into_iter()
        .map(|mut param| {
            param.name = param.name.trim().to_string();
            param.value = param.value.clone().map(|v| v.trim().to_string());
            // Find and remove any html comments
            if let Some(ref v) = param.value {
                param.value = Some(trim_comments(v));
            }
            param
        })
        .collect();
    let title = page.title().to_string();
    match parsed.name_str().trim() {
        "PortGraph" | "Graph:Chart" | "GraphChart" => {
            trace!("Template {:?}", parsed);
            let mut name = parsed.params_get("name").flatten();

            // Special handling for demographics related pages
            if name.is_none() && title.starts_with("Demographics of ") {
                // now we need to extract the name from the title
                let country = title.trim_start_matches("Demographics of").trim();
                let country = country.trim_start_matches("the").trim();
                if country.is_empty() {
                    bail!("Country name empty, unreachable");
                }
                if parsed
                    .params_map()
                    .get("y2Title")
                    .cloned()
                    .flatten()
                    .is_some()
                {
                    bail!(
                        "y2Title is not supported for demographics pages without template graph name"
                    );
                }
                match &*parsed.params_map().get("y1Title").cloned().flatten().ok_or_else(|| {
                    anyhow::anyhow!("'y1Title' parameter is required on demographics pages without template graph name")
                })?.to_ascii_lowercase() {
                    s if s.starts_with("population") => {
                        name = Some(format!("{country} Total Population"));
                        if !parsed.params_map().contains_key("title") {
                            parsed.params_insert("title".to_string(), Some(format!("{country} Population")));
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
                        if !parsed.params_map().contains_key("title") {
                            parsed.params_insert("title".to_string(), Some("Total Fertility Rate".to_string()));
                        }
                    }
                    _ => {
                        bail!("Unsupported y1Title for demographics page: {}", parsed.params_map().get("y1Title").cloned().flatten().unwrap_or_default());
                    }
                }
            }

            let name = name
                .ok_or_else(|| anyhow::anyhow!("'name' parameter is required to port the graph"))?;
            let rev_url = if let Some(rev_info) = rev_info {
                format!(
                    "{}w/index.php?title={}&oldid={}",
                    config.read().await.wiki,
                    title.replace(' ', "_"),
                    rev_info.id
                )
            } else {
                format!(
                    "{}w/index.php?title={}",
                    config.read().await.wiki,
                    title.replace(' ', "_")
                )
            };
            let swap = create_pages(bot, &parsed, &name, &rev_url)
                .await
                .context("Failed to generate/create pages")?;
            Ok(Some(Swap {
                from: parsed.text,
                to: swap,
            }))
        }
        _ => Ok(None),
    }
}

#[tracing::instrument(skip_all)]
pub async fn run_on_page(
    page: Page,
    rev_info: Option<RevInfo>,
    commons_bot: &Bot,
    _wiki_bot: &Bot,
    config: &RwLock<Config>,
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
    let (content, ()) = tokio::join!(content_future, rm_future);
    let content = content.context("Failed to get wikitext")?;
    let p = call_parser(&content, config).await?;

    let mut tasks = vec![];
    for parsed in p.parsed.templates {
        let template_name = parsed.name_str().trim().to_string();
        let graph_name = parsed.params_get("name").flatten();
        tasks.push(async {
            let result =
                handle_template(commons_bot, parsed, page.clone(), rev_info.clone(), config).await;
            (template_name, graph_name, result)
        });
    }
    let task_results = futures::future::join_all(tasks).await;

    let mut swaps = vec![];
    let mut errors = vec![];
    for (template_name, graph_name, result) in task_results {
        match result {
            Ok(s) => {
                if let Some(swap) = s {
                    swaps.push(swap);
                }
            }
            Err(error) => {
                error!(
                    page = %page.title(),
                    template = %template_name,
                    graph_name = graph_name.as_deref().unwrap_or("<missing>"),
                    error = %format!("{error:#}"),
                    "Error handling graph template"
                );
                errors.push(error);
            }
        }
    }
    let mut modified_wikitext = content.clone();
    for swap in swaps {
        if modified_wikitext.contains(&swap.from) {
            modified_wikitext = modified_wikitext.replace(&swap.from, &swap.to);
        } else {
            warn!(
                page = %page.title(),
                template_length = swap.from.len(),
                "Original graph template was not found during replacement"
            );
        }
    }
    // Save the modified wikitext back to the page
    let title = page.title().to_string();
    if modified_wikitext == content {
        info!("No changes made to page {title}");
        tokio::time::sleep(Duration::from_secs(1)).await;
    } else {
        let save_options = SaveOptions::summary("Port graphs to charts").mark_as_bot(true);
        match page.save(modified_wikitext, &save_options).await {
            Ok(_) => info!("Successfully updated page {title}"),
            Err(error) => {
                let error = anyhow::Error::from(error);
                error!(
                    page = %title,
                    error = %format!("{error:#}"),
                    "Failed to update page"
                );
                return Err(error).with_context(|| format!("Failed to update page {title}"));
            }
        }
    }
    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Errors occurred while processing page {title}: {errors:?}"
        ));
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn page_handler(
    rx: Arc<Mutex<Receiver<PageRequest>>>,
    commons_bot: Arc<Bot>,
    wiki_bot: Arc<Bot>,
    config: Arc<RwLock<Config>>,
) {
    loop {
        let mut rx_lock = rx.lock().await;
        if let Some((page, rev_info, result_handler)) = rx_lock.recv().await {
            drop(rx_lock);
            let page_title = page.title().to_string();
            if result_handler
                .send(run_on_page(page, rev_info, &commons_bot, &wiki_bot, &config).await)
                .is_err()
            {
                warn!(page = %page_title, "Page result receiver was dropped");
            }
        } else {
            // Channel closed, exit the loop
            info!("Page request channel closed, exiting worker.");
            break;
        }
    }
}

async fn spawn_workers(
    wiki_bot: &Arc<Bot>,
    commons_bot: &Arc<Bot>,
    config: &Arc<RwLock<Config>>,
    rx: &Arc<Mutex<Receiver<PageRequest>>>,
) -> Vec<JoinHandle<()>> {
    let num_workers = config
        .read()
        .await
        .graph_task
        .num_workers
        .unwrap_or_else(|| num_cpus::get().clamp(1, 8));
    let mut workers = Vec::with_capacity(num_workers);
    trace!("Spawning {} workers for page handling", num_workers);
    for i in 0..num_workers {
        let wiki_bot = Arc::clone(wiki_bot);
        let commons_bot = Arc::clone(commons_bot);
        let config = Arc::clone(config);
        let rx = Arc::clone(rx);
        let worker = task::spawn(async move {
            info!("Starting worker #{i} for page handling");
            page_handler(rx, commons_bot, wiki_bot, config).await;
        });
        workers.push(worker);
    }
    workers
}

pub async fn graph_task(
    commons_bot: Arc<Bot>,
    wiki_bot: Arc<Bot>,
    config: Arc<RwLock<Config>>,
) -> anyhow::Result<()> {
    // check for parser load
    // if parsing nothing fails, it must be not running or very broken
    if let Err(error) = call_parser("", &config).await {
        error!(
            error = %format!("{error:#}"),
            "Parser failed to parse empty input; the parser service may be unavailable"
        );
        return Err(error);
    }
    info!("Starting Graph Port task");
    let failed_revs = Arc::new(FailedRevs::load(&config).await?);

    // Create workers
    let (page_sender, page_reciever) = mpsc::channel(100);
    let rx = Arc::new(Mutex::new(page_reciever));
    let workers = spawn_workers(&wiki_bot, &commons_bot, &config, &rx).await;

    loop {
        if config.read().await.shutdown_graph_task {
            info!("Shutdown flag is set, exiting.");
            break;
        }
        if config.read().await.pause_graph_task {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        // Get the list of articles to process
        let generator = CategoryMembers::new(&config.read().await.graph_task.search_category)
            .sort(CategoryMemberSort::Timestamp);
        let mut output = generator.generate(&wiki_bot);
        while let Some(o) = output.next().await {
            if let Err(e) = o {
                error!("Error receiving page: {e}");
                continue;
            }
            let o = o?;
            debug!("Processing page: {}", o.title());
            let revid = api_utils::get_revid(&o, &wiki_bot).await;
            let page_title = o.title().to_string();
            let rev_info = revid.map(|id| RevInfo::new(id, page_title.clone()));
            if let Some(ref rev_info) = rev_info {
                if failed_revs.contains_key(rev_info).await? {
                    debug!(
                        "Skipping page {} with revision ID {} due to previous failure",
                        rev_info.page_title, rev_info.id
                    );
                    continue;
                }
                trace!(
                    "Processing page {} with revision ID {}",
                    rev_info.page_title, rev_info.id
                );
            }
            if config.read().await.shutdown_graph_task {
                info!("Shutdown flag is set, exiting.");
                break;
            }

            let (send, rec) = oneshot::channel();
            if let Err(e) = page_sender.send((o, rev_info.clone(), send)).await {
                error!("Failed to send page to handler: {e}");
                continue;
            }
            trace!("Page {page_title} sent to handlers");
            task::spawn({
                let failed_revs = Arc::clone(&failed_revs);
                async move {
                    match rec.await {
                        Ok(result) => {
                            if let Err(error) = result {
                                error!(
                                    page = %page_title,
                                    revision_id = rev_info.as_ref().map(|rev| rev.id),
                                    error = %format!("{error:#}"),
                                    "Error processing page"
                                );
                                if let Some(rev_info) = rev_info {
                                    failed_revs.insert(rev_info, error).await.unwrap_or_else(
                                        |db_error| {
                                            error!(
                                                page = %page_title,
                                                error = %format!("{db_error:#}"),
                                                "Failed to record failed revision"
                                            );
                                        },
                                    );
                                }
                            }
                        }
                        Err(error) => error!(
                            page = %page_title,
                            error = %error,
                            "Failed to receive page result"
                        ),
                    }
                }
            });
        }
        if config.read().await.shutdown_graph_task {
            info!("Shutdown flag is set, exiting.");
            break;
        }
        info!(
            "No more articles found in {}",
            config.read().await.graph_task.search_category
        );
        sleep(Duration::from_secs(10)).await;
    }
    drop(workers); // Ensure all worker handles are dropped right before exiting
    Ok(())
}
