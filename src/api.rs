use axum::{Json, Router, routing::get};
use crate::model::{StatusResponse, app_version};
{% if include_db %}
use crate::store::Store;
use std::sync::Arc;
{% endif %}

{% if include_db %}
pub fn router(store: Arc<Store>) -> Router {
    let _ = store; // extend: add store to axum State
    Router::new().route("/api/v1/status", get(status))
}
{% else %}
pub fn router() -> Router {
    Router::new().route("/api/v1/status", get(status))
}
{% endif %}

async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok",
        version: app_version(),
    })
}
