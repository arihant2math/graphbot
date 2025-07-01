pub use sea_orm_migration::prelude::*;

pub(crate) mod m20250701_033951_page_map;
pub(crate)  mod m20250701_034647_error;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250701_033951_page_map::Migration),
            Box::new(m20250701_034647_error::Migration),
        ]
    }
}
