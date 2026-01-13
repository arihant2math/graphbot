use std::sync::Arc;

use axum::{Json, Router, extract::State, response::Html, routing::get};
use graphbot_config::Config;
use graphbot_db::{graph_failed_conversions, prelude::GraphFailedConversions};
use sea_orm::{ConnectOptions, Database, DbConn, EntityTrait};
use tera::Tera;

async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut context = tera::Context::new();
    let failed = GraphFailedConversions::find().all(&state.db).await.unwrap();
    context.insert("failed_revs", &failed);
    Html(state.tera.render("index.html", &context).unwrap())
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
    options.max_connections(3);
    let db = Database::connect(options).await.unwrap();
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };
    let state = Arc::new(AppState { db, tera });
    println!("Connected to database successfully.");

    let app = Router::new()
        .route("/", get(root))
        .route("/failed_revs.json", get(json_failed_revs))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
