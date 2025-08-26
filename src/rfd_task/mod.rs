use std::{sync::Arc, time::Duration};

use crate::parser;
use graphbot_config::Config;
use mwapi_responses::query;
use mwbot::Bot;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;

const MAIN_RFD_PAGE: &str = "Wikipedia:Redirects for discussion";

#[derive(Clone, Debug, Default)]
struct Redirect {
    name: String,
    target: String,
}

#[derive(Clone, Debug, Default)]
struct Rfd {
    name: String,
    associated_redirects: Vec<Redirect>,
}

// struct PaginatedRedirectIter {
//     bot: Arc<Bot>,
//     title: String,
//     current: Vec<PageResult>,
//     r#continue: Option<()>,
// }
//
// impl PaginatedRedirectIter {
//     fn new(bot: Arc<Bot>, title: &str) -> Self {
//         PaginatedRedirectIter {
//             bot,
//             title: title.to_string(),
//             current: Vec::new(),
//             r#continue: Some(()),
//         }
//     }
// }
//
// impl Iterator for PaginatedRedirectIter {
//     type Item = anyhow::Result<RedirectResult>;
//
//     async fn next(&mut self) -> Option<Self::Item> {
//         if !self.current.is_empty() {
//             if let Some(redirect) = self.current.pop() {
//                 return
// Some(Ok(redirect.redirects.into_iter().next().unwrap()));             } else
// {                 unreachable!();
//             }
//         }
//         let response  = mwapi_responses::query_api(&self.bot.api(), [("prop",
// "redirects"), ("titles", &self.title), ("rdlimit", "max")]).await;         if
// let Some(page) = self.current.pop() {             if let Some(redirect) =
// page.redirects.into_iter().next() {                 return
// Some(Ok(redirect));             }
//         }
//         None
//     }
// }

/// Determines whether there are any other redirects, in any namespace, that
/// meet one or more of the following criteria:
/// - Are marked as an avoided-double redirect of a nominated redirect
/// - Are redirects to the nominated redirect
/// - Redirect to the same target as the nominated redirect and differ only in
///   the presence or absence of non-alphanumeric characters, and/or differ only
///   in case
async fn inference(rfd: Rfd, wiki_bot: &Bot) -> anyhow::Result<()> {
    info!("Inference called for RFD: {:?}", rfd);
    for redirect in rfd.associated_redirects {
        // Iterate through the associated redirects for redirect.target
        // let mut similar_redirects = Vec::new();
        fn normalize_str(s: &str) -> String {
            diacritics::remove_diacritics(s)
                .to_lowercase()
                .replace(" ", "")
                .replace("_", "")
                .replace("-", "")
                .replace("'", "")
                .replace(".", "")
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{COMMONS_API_URL, COMMONS_REST_URL, USER_AGENT};
    use graphbot_config::Config;
    use mwbot::Bot;
    use tokio::join;

    #[tokio::test]
    async fn test_inference() {
        let config = Arc::new(RwLock::new(Config::default()));
        let url = url::Url::parse(&config.read().await.wiki).unwrap();
        let api_url = url.join("w/api.php").unwrap();
        let rest_url = url.join("api/rest_v1").unwrap();
        let token = config.read().await.access_token.clone();
        let wiki_bot = Bot::builder(api_url.to_string(), rest_url.to_string())
            .set_user_agent(USER_AGENT.to_string())
            .build()
            .await
            .unwrap();
        let wiki_bot = Arc::new(wiki_bot);
        let rfd = Rfd {
            name: "Test RFD".to_string(),
            associated_redirects: vec![Redirect {
                name: "Fur cap".to_string(),
                target: "Hangul".to_string(),
            }],
        };
        let inference = inference(rfd, &wiki_bot).await.unwrap();
    }
}

pub async fn rfd_task(wiki_bot: Arc<Bot>, config: Arc<RwLock<Config>>) -> anyhow::Result<()> {
    info!("Starting RFD task");
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
            if template
                .name_str()
                .starts_with("Wikipedia:Redirects for discussion/Log")
            {
                rfd_pages.push(template.name_str().to_string());
            }
        }
        for rfd_page in rfd_pages {
            info!("Processing RFD page: {}", rfd_page);
            let rfd_page_obj = wiki_bot.page(&rfd_page)?;
            let rfd_text = rfd_page_obj.wikitext().await?;
            let parsed_rfd = parser::call_parser(&rfd_text, &config).await?;

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
                    assert!(
                        !self.rfds_started,
                        "RFDS already started, cannot start again."
                    );
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
                    assert!(
                        !self.current_rfd().name.is_empty(),
                        "RFD name is empty, cannot finalize."
                    );
                    assert!(
                        !self.current_rfd().associated_redirects.is_empty(),
                        "RFD has no associated redirects, cannot finalize."
                    );
                    self.rfds.push(Rfd::default());
                }
            }

            let mut state = State::new();

            for node in &parsed_rfd.parsed.nodes {
                if let Some(comment) = node.clone().into_comment() {
                    if comment
                        .text
                        .contains("Add new entries directly below this line")
                    {
                        state.start_rfds();
                        continue; // Skip the comment line
                    }
                }
                if state.rfds_started {
                    // dbg!(&node.text);
                    if let Some(heading) = node.clone().into_heading() {
                        if heading.level() == 4 {
                            state.current_rfd_mut().name =
                                heading.title_str().to_string().trim().to_string();
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
