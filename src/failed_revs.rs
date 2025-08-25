use std::time::Duration;
use crate::rev_info::RevInfo;
use graphbot_config::Config;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, sqlx::types::chrono, ConnectOptions};
use tokio::sync::RwLock;

pub struct FailedRevs(DatabaseConnection);

impl FailedRevs {
    pub async fn load(config: &RwLock<Config>) -> anyhow::Result<Self> {
        let url = config.read().await.graph_task.db_url.clone();
        let mut options = ConnectOptions::new(&url);
        options.max_connections(4);
        let db = Database::connect(options).await?;
        Ok(Self(db))
    }

    pub async fn get(&self, rev_info: &RevInfo) -> anyhow::Result<Option<String>> {
        let prev =
            graphbot_db::prelude::GraphFailedConversions::find_by_id(rev_info.page_title.clone())
                .one(&self.0)
                .await?;
        if let Some(prev) = prev {
            Ok(prev.error)
        } else {
            Ok(None)
        }
    }

    pub async fn insert(&self, rev_info: RevInfo, error: anyhow::Error) -> anyhow::Result<()> {
        let error = error.to_string();
        if let Some(entry) =
            graphbot_db::prelude::GraphFailedConversions::find_by_id(rev_info.page_title.clone())
                .one(&self.0)
                .await?
        {
            let mut updated_entry = entry.into_active_model();
            updated_entry.rev_id = ActiveValue::Set(rev_info.id as i32);
            updated_entry.error = ActiveValue::Set(Some(error));
            updated_entry.date = ActiveValue::Set(chrono::Utc::now()); // Force update date to current time
            updated_entry.update(&self.0).await?;
        } else {
            let new_entry = graphbot_db::graph_failed_conversions::ActiveModel {
                page_title: ActiveValue::Set(rev_info.page_title.clone()),
                rev_id: ActiveValue::Set(rev_info.id as i32),
                error: ActiveValue::Set(Some(error)),
                date: ActiveValue::Set(chrono::Utc::now()),
            };
            new_entry.insert(&self.0).await?;
        }
        Ok(())
    }

    pub async fn contains_key(&self, rev_info: &RevInfo) -> anyhow::Result<bool> {
        let prev =
            graphbot_db::prelude::GraphFailedConversions::find_by_id(rev_info.page_title.clone())
                .one(&self.0)
                .await?;
        if let Some(prev) = &prev {
            if prev.rev_id != rev_info.id as i32 {
                // If the rev_id is different, delete the entry
                graphbot_db::prelude::GraphFailedConversions::delete_by_id(
                    rev_info.page_title.clone(),
                )
                .exec(&self.0)
                .await?;
                return Ok(false);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
