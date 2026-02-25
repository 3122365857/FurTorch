//! TorchRust HTTP Server
//!
//! Axum-based API server with log watching. Serves the React frontend and provides
//! REST API for config, stats, and drop processing. Uses torchrust-core directly.

mod log_path;
mod log_watcher;
mod state;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use log_path::LogPathConfig;
use log_watcher::LogWatcher;
use state::AppState;

#[derive(Clone)]
struct ServerState {
    app: Arc<AppState>,
    log_watcher_status: Arc<std::sync::RwLock<Option<LogWatcherStatus>>>,
}

#[derive(serde::Serialize)]
struct LogWatcherStatus {
    ok: bool,
    path: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let app_state = Arc::new(AppState::new(project_root.clone()));
    let log_watcher_status = Arc::new(std::sync::RwLock::new(None));

    let server_state = ServerState {
        app: app_state,
        log_watcher_status: log_watcher_status.clone(),
    };

    // Start log watcher from config
    {
        let data_dir = project_root.join("data");
        let config_path = data_dir.join("config.json");
        if let Ok(config_str) = tokio::fs::read_to_string(&config_path).await {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
                let log_watcher_enabled = config.get("log_watcher_enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                if log_watcher_enabled {
                    let log_path_config = LogPathConfig {
                        log_path: config.get("log_path").and_then(|v| v.as_str()).map(String::from),
                        use_tmp_log: config.get("use_tmp_log").and_then(|v| v.as_bool()).unwrap_or(false),
                    };
                    let poll_interval = config
                        .get("log_poll_interval_sec")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1);

                    if let Some((_watcher, path)) = LogWatcher::spawn(
                        server_state.app.clone(),
                        log_path_config,
                        poll_interval,
                        project_root.clone(),
                    ) {
                        let mut status = log_watcher_status.write().unwrap();
                        *status = Some(LogWatcherStatus {
                            ok: true,
                            path: Some(path.display().to_string()),
                            error: None,
                        });
                        println!("Log watcher started: {} (poll {}s)", path.display(), poll_interval);
                    } else {
                        let mut status = log_watcher_status.write().unwrap();
                        *status = Some(LogWatcherStatus {
                            ok: false,
                            path: None,
                            error: Some("No log path. Set log_path in config, or use_tmp_log for debug.".to_string()),
                        });
                        println!("Log watcher: No log path. Set log_path in config, or use_tmp_log for debug.");
                    }
                }
            }
        }
    }

    let web_dir = project_root.join("web").join("dist");

    let api = Router::new()
        .route("/api/config", get(get_config).patch(patch_config))
        .route("/api/price-table", get(get_price_table))
        .route("/api/full-table", get(get_price_table))
        .route("/api/process-drops", post(process_drops))
        .route("/api/stats", get(get_stats).post(reset_stats))
        .route("/api/stats/reset", post(reset_stats))
        .route("/api/health", get(health))
        .route("/api/log-watcher", get(get_log_watcher))
        .with_state(server_state);

    let app = Router::new()
        .merge(api)
        .fallback_service(ServeDir::new(web_dir))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    let port = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3001);
    let addr = (std::net::Ipv4Addr::UNSPECIFIED, port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
        eprintln!("Cannot bind to port {}: {}. Try PORT=3002 cargo run -p torchrust-server", port, e);
        std::process::exit(1);
    });
    println!("TorchRust server running at http://localhost:{}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn get_config(State(state): State<ServerState>) -> impl IntoResponse {
    let config_path = state.app.data_dir.join("config.json");
    match tokio::fs::read_to_string(&config_path).await {
        Ok(s) => {
            let config: serde_json::Value = serde_json::from_str(&s).unwrap_or(serde_json::json!({"locale": "ko"}));
            Json(config)
        }
        Err(_) => Json(serde_json::json!({"locale": "ko", "error": "Failed to load config"})),
    }
}

async fn patch_config(
    State(state): State<ServerState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let config_path = state.app.data_dir.join("config.json");
    let existing: serde_json::Value = tokio::fs::read_to_string(&config_path)
        .await
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let updated = merge_json(&existing, &payload);
    if let Err(e) = tokio::fs::write(
        &config_path,
        serde_json::to_string_pretty(&updated).unwrap_or_default(),
    )
    .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("{}", e)})));
    }
    (StatusCode::OK, Json(updated))
}

fn merge_json(base: &serde_json::Value, patch: &serde_json::Value) -> serde_json::Value {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            let mut out = b.clone();
            for (k, v) in p {
                out.insert(k.clone(), merge_json(b.get(k).unwrap_or(&serde_json::Value::Null), v));
            }
            serde_json::Value::Object(out)
        }
        (_, p) => p.clone(),
    }
}

async fn get_price_table(State(state): State<ServerState>) -> impl IntoResponse {
    let table_path = state.app.data_dir.join("ttd_price.json");
    match tokio::fs::read_to_string(&table_path).await {
        Ok(s) => (StatusCode::OK, Json(serde_json::from_str::<serde_json::Value>(&s).unwrap_or_default())).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to load price table"}))).into_response(),
    }
}

async fn process_drops(
    State(state): State<ServerState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let log_text = body.get("log_text").and_then(|v| v.as_str());
    let Some(log_text) = log_text else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "log_text required"})));
    };

    match state.app.process_log_chunk(log_text).await {
        Ok(()) => {
            let stats = state.app.get_drop_state().await;
            (StatusCode::OK, Json(serde_json::json!({
                "drop_list": stats.drop_list,
                "drop_list_all": stats.drop_list_all,
                "income": stats.income,
                "income_all": stats.income_all,
                "map_count": stats.map_count,
                "total_time_sec": stats.total_time_sec,
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))),
    }
}

async fn get_stats(State(state): State<ServerState>) -> impl IntoResponse {
    let stats = state.app.get_drop_state().await;
    Json(serde_json::json!({
        "drop_list": stats.drop_list,
        "drop_list_all": stats.drop_list_all,
        "income": stats.income,
        "income_all": stats.income_all,
        "map_count": stats.map_count,
        "total_time_sec": stats.total_time_sec,
    }))
}

async fn reset_stats(State(state): State<ServerState>) -> impl IntoResponse {
    state.app.reset_stats().await;
    Json(serde_json::json!({"ok": true}))
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "torchrust-server"}))
}

async fn get_log_watcher(State(state): State<ServerState>) -> impl IntoResponse {
    let status = state.log_watcher_status.read().unwrap();
    match status.as_ref() {
        Some(s) => Json(serde_json::json!({
            "ok": s.ok,
            "path": s.path,
            "error": s.error,
        })),
        None => Json(serde_json::json!({"ok": false, "error": "Not started"})),
    }
}
