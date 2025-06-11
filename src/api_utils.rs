use mwbot::{Bot, Page};

pub async fn get_revid(page: &Page, bot: &Bot) -> Option<u64> {
    let resp = bot.parsoid().get(page.title()).await.ok()?;
    resp.revision_id()
}
