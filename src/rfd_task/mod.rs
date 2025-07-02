use std::sync::Arc;
use std::time::Duration;
use mwbot::Bot;
use tokio::sync::RwLock;
use tracing::info;
use crate::config::Config;
use crate::parser;

const MAIN_RFD_PAGE: &str = "Wikipedia:Redirects for discussion";

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
        let page = wiki_bot.page(MAIN_RFD_PAGE)?;
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

            #[derive(Clone, Debug, Default)]
            struct Rfd {
                name: String,
                associated_redirects: Vec<String>,
            }

            struct State {
                rfds_started: bool,
                rfds: Vec<Rfd>,
            }

            impl State {
                fn new() -> Self {
                    State {
                        rfds_started: false,
                        rfds: Vec::new(),
                    }
                }

                fn start_rfds(&mut self) {
                    assert!(!self.rfds_started, "RFDS already started, cannot start again.");
                    self.rfds_started = true;
                    assert!(self.rfds.is_empty(), "RFDS already non-zero length.");
                    self.rfds.push(Rfd::default());
                }

                fn current_rfd(&self) -> &Rfd {
                    self.rfds.last().unwrap()
                }

                fn current_rfd_mut(&mut self) -> &mut Rfd {
                    self.rfds.last_mut().unwrap()
                }

                fn finalize_rfd(&mut self) {
                    assert!(self.rfds_started, "RFDS not started, cannot finalize.");
                    assert!(!self.current_rfd().name.is_empty(), "RFD name is empty, cannot finalize.");
                    assert!(!self.current_rfd().associated_redirects.is_empty(), "RFD has no associated redirects, cannot finalize.");
                    self.rfds.push(Rfd::default());
                }
            }

            let mut state = State::new();

            for node in &parsed_rfd.parsed.nodes {
                if let Some(comment) = node.clone().into_comment() {
                    if comment.text.contains("Add new entries directly below this line") {
                        state.start_rfds();
                        continue; // Skip the comment line
                    }
                }
                if state.rfds_started {
                    // dbg!(&node.text);
                    if let Some(heading) = node.clone().into_heading() {
                        if heading.level() == 4 {
                            state.current_rfd_mut().name = heading.title_str().to_string().trim().to_string();
                        }
                    } else {
                        dbg!(&node.text);
                        if let Some(span) = node.clone().into_html_entity() {
                            dbg!(span);
                        }
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
