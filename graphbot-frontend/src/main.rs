use std::sync::Arc;

use axum::{Json, Router, extract::State, response::Html, routing::get};
use axum::http::StatusCode;
use graphbot_config::Config;
use graphbot_db::{graph_failed_conversions, prelude::GraphFailedConversions};
use sea_orm::{ConnectOptions, Database, DbConn, EntityTrait};
use tera::Tera;

async fn root(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let mut context = tera::Context::new();
    let failed = GraphFailedConversions::find()
        .all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    context.insert("failed_revs", &failed);
    Ok(Html(state.tera.render("index.html", &context).map_err(|e| {
        eprintln!("Template error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?))
}

async fn json_failed_revs(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<graph_failed_conversions::Model>> {
    Json(GraphFailedConversions::find().all(&state.db).await.unwrap())
}

struct AppState {
    db: DbConn,
    tera: Tera,
}

#[tokio::main]
async fn main() {
    let config = Config::load().unwrap();
    let url = config.graph_task.db_url;
    let mut options = ConnectOptions::new(&url);
    options.max_connections(1);
    println!("Connecting to database ...");
    let db = Database::connect(options).await.unwrap();
    println!("Connected to database successfully.");
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let state = Arc::new(AppState { db, tera });

    let app = Router::new()
        .route("/", get(root))
        .route("/failed_revs.json", get(json_failed_revs))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
