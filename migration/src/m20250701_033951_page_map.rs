use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PageMap::Table)
                    .if_not_exists()
                    .col(string(PageMap::Title).primary_key())
                    .col(integer(PageMap::RevId).unique_key())
                    .col(timestamp(PageMap::Timestamp).default(chrono::Utc::now()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(PageMap::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PageMap {
    Table,
    Title,
    RevId,
    Timestamp
}
