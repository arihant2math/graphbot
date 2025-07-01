use sea_orm_migration::{prelude::*, schema::*};
use crate::m20250701_033951_page_map::PageMap;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Error::Table)
                    .if_not_exists()
                    .col(pk_auto(Error::Id))
                    .foreign_key(ForeignKey::create()
                        .name("error_rev_id_fkey")
                        .from(PageMap::Table, PageMap::RevId)
                        .to(Error::Table, Error::RevId)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                    )
                    .col(string(Error::Text))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Error::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Error {
    Table,
    Id,
    RevId,
    Text,
}
