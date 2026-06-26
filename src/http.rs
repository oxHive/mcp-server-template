use crate::{api, config::ServerSettings, server::{{project-name | pascal_case}}};
use anyhow::Result;
use axum::{Router, body::Body, http::{StatusCode, header}, response::Response, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::sync::Arc;
{% if include_db %}
use crate::store::Store;
{% endif %}
{% if include_dashboard %}
use include_dir::{Dir, include_dir};

static DASHBOARD: Dir = include_dir!("$CARGO_MANIFEST_DIR/{{dashboard_dir}}/dist");
{% endif %}

{% if include_db %}
pub fn app_router(store: Arc<Store>) -> Router {
    let mcp = StreamableHttpService::new(
        {
            let store = store.clone();
            move || Ok({{project-name | pascal_case}}::new(store.clone()))
        },
        Arc::new(LocalSessionManager::default()),
        Default::default(),
    );
    api::router(store).nest_service("/mcp", mcp)
}
{% else %}
pub fn app_router() -> Router {
    let mcp = StreamableHttpService::new(
        move || Ok({{project-name | pascal_case}}::new()),
        Arc::new(LocalSessionManager::default()),
        Default::default(),
    );
    api::router().nest_service("/mcp", mcp)
}
{% endif %}

{% if include_dashboard %}
pub fn dashboard_router(api_url: &str) -> Router {
    let config_js = format!("window.APP_API = {};\n", serde_json::json!(api_url));
    Router::new()
        .route(
            "/config.js",
            get({
                let body = config_js.clone();
                move || {
                    let b = body.clone();
                    async move {
                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/javascript")
                            .body(Body::from(b))
                            .unwrap()
                    }
                }
            }),
        )
        .fallback(get(|req: axum::extract::Request| async move {
            let path = req.uri().path().trim_start_matches('/');
            let path = if path.is_empty() { "index.html" } else { path };
            match DASHBOARD.get_file(path) {
                Some(file) => {
                    let mime = mime_guess::from_path(path).first_or_octet_stream();
                    Response::builder()
                        .header(header::CONTENT_TYPE, mime.as_ref())
                        .body(Body::from(file.contents()))
                        .unwrap()
                }
                None => match DASHBOARD.get_file("index.html") {
                    Some(file) => Response::builder()
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(file.contents()))
                        .unwrap(),
                    None => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("not found"))
                        .unwrap(),
                },
            }
        }))
}
{% endif %}

pub async fn run_up(settings: &ServerSettings) -> Result<()> {
    {% if include_db %}
    let db_path = crate::db::resolve_db_path();
    let database = crate::db::open_database(&db_path).await?;
    let conn = database.connect()?;
    crate::db::run_migrations(&conn).await?;
    let store = Arc::new(Store::new(conn));
    let app = app_router(store);
    {% else %}
    let app = app_router();
    {% endif %}

    let listener = tokio::net::TcpListener::bind(
        (settings.host.as_str(), settings.port)
    ).await?;
    tracing::info!("MCP: http://{}:{}/mcp", settings.host, settings.port);
    tracing::info!("API: http://{}:{}/api/v1", settings.host, settings.port);

    {% if include_dashboard %}
    let dash = dashboard_router(&settings.api_url);
    let dash_listener = tokio::net::TcpListener::bind(
        (settings.host.as_str(), settings.dashboard_port)
    ).await?;
    tracing::info!("Dashboard: http://{}:{}", settings.host, settings.dashboard_port);

    tokio::try_join!(
        async { axum::serve(listener, app).await.map_err(anyhow::Error::from) },
        async { axum::serve(dash_listener, dash).await.map_err(anyhow::Error::from) },
    )?;
    {% else %}
    axum::serve(listener, app).await?;
    {% endif %}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn app_router_serves_status() {
        {% if include_db %}
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test.db").to_string_lossy().to_string();
        let database = crate::db::open_database(&db_path).await.unwrap();
        let conn = database.connect().unwrap();
        crate::db::run_migrations(&conn).await.unwrap();
        let store = Arc::new(Store::new(conn));
        let app = app_router(store);
        {% else %}
        let app = app_router();
        {% endif %}

        let resp = app
            .oneshot(Request::builder().uri("/api/v1/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
