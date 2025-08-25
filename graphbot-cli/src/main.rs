use clap::{Parser, Subcommand};
use sea_orm::{ConnectionTrait, EntityTrait, Schema};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Database operations
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Create a new database
    Create,
    Delete
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Db { command } => match command {
            DbCommands::Create => {
                let config = graphbot_config::Config::load().unwrap();
                println!("Creating database at {}...", config.graph_task.db_url);
                let connection = sea_orm::Database::connect(&config.graph_task.db_url)
                    .await
                    .expect("Failed to connect to the database");
                async fn create_table<E: EntityTrait>(connection: &sea_orm::DatabaseConnection, table: E) {
                    let backend = connection.get_database_backend();
                    let schema = Schema::new(backend);
                    let sql = backend.build(&schema.create_table_from_entity(table));
                    connection.execute(sql).await.unwrap();
                }
                create_table(&connection, graphbot_db::graph_failed_conversions::Entity).await;
            }
            DbCommands::Delete => {
                println!("Deleting database is not implemented yet.");
            }
        },
    }
}