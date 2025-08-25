use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use graphbot_config::Config;

struct AppState {
    config: Arc<RwLock<Config>>,
}

async fn get_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = state.config.read().await;
    let mut headers = HeaderMap::new();
    headers.append("Content-Type", "application/json".parse().unwrap());
    match serde_json::to_string(&*config) {
        Ok(json) => (StatusCode::OK, headers, json),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            headers,
            format!("Failed to serialize config. Error: {err}"),
        ),
    }
}

async fn post_config(State(state): State<Arc<AppState>>, new_config: String) -> impl IntoResponse {
    let mut config = state.config.write().await;
    let mut headers = HeaderMap::new();
    headers.append("Content-Type", "application/json".parse().unwrap());
    match serde_json::from_str::<Value>(&new_config) {
        Ok(updated_config) => {
            let config_value: Value = match serde_json::to_value(&*config) {
                Ok(value) => value,
                Err(err) => {
                    info!("Failed to serialize current config: {err}");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        headers,
                        json!({
                            "error": {
                                "message": format!("Failed to serialize current config. Error: {err}")
                            }
                        }).to_string(),
                    );
                }
            };
            let merged_config = config_value
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .chain(
                            updated_config
                                .as_object()
                                .unwrap_or(&serde_json::Map::new())
                                .iter(),
                        )
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<serde_json::Map<String, Value>>()
                })
                .unwrap_or_default();
            match serde_json::from_value(Value::Object(merged_config)) {
                Ok(new_config) => {
                    *config = new_config;
                    info!("Config updated successfully");
                }
                Err(err) => {
                    info!("Failed to parse new config: {err}");
                    return (
                        StatusCode::BAD_REQUEST,
                        headers,
                        json!({
                            "error": {
                                "message": format!("Failed to parse new config. Error: {err}")
                            }
                        })
                        .to_string(),
                    );
                }
            }
            (StatusCode::OK, headers, json!({}).to_string())
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            headers,
            json!({
                "error": {
                    "message": format!("Failed to parse config. Error: {err}")
                }
            })
            .to_string(),
        ),
    }
}

pub async fn run(config: Arc<RwLock<Config>>) -> anyhow::Result<()> {
    info!("Starting server");
    let shared_state = Arc::new(AppState { config });
    let app = Router::new()
        .route("/config", get(get_config).post(post_config))
        .with_state(shared_state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());
    let port = {
        let config = shared_state.config.read().await;
        config.server.port
    };
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, app).await?;
    Ok(())
}
