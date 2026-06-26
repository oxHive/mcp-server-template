use rmcp::{
    RoleServer,
    handler::server::wrapper::Parameters,
    model::CallToolResult,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::json;
{% if include_db %}
use crate::store::Store;
use std::sync::Arc;
{% endif %}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PingInput {
    /// Message to echo back
    pub message: String,
}

pub struct {{project-name | pascal_case}} {
    {% if include_db %}
    store: Arc<Store>,
    {% endif %}
}

impl {{project-name | pascal_case}} {
    {% if include_db %}
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }
    {% else %}
    pub fn new() -> Self {
        Self {}
    }
    {% endif %}
}

#[tool_router]
impl {{project-name | pascal_case}} {
    #[tool(description = "Echo a message back — replace with your first real tool")]
    async fn ping(
        &self,
        Parameters(input): Parameters<PingInput>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json!({ "echo": input.message }).to_string(),
        )]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for {{project-name | pascal_case}} {}
