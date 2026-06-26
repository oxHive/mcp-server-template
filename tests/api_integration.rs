use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
{% if include_db %}
use {{crate_name}}::{db, http::app_router, store::Store};
use std::sync::Arc;

async fn test_app() -> axum::Router {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("test.db").to_string_lossy().to_string();
    let database = db::open_database(&db_path).await.unwrap();
    let conn = database.connect().unwrap();
    db::run_migrations(&conn).await.unwrap();
    let store = Arc::new(Store::new(conn));
    app_router(store)
}
{% else %}
use {{crate_name}}::http::app_router;

async fn test_app() -> axum::Router {
    app_router()
}
{% endif %}

#[tokio::test]
async fn status_returns_ok() {
    let app = test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let app = test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
