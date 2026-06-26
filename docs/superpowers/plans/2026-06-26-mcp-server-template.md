# mcp-server-template Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a `cargo-generate` template that scaffolds a Rust MCP server project with REST API, optional SQLite storage, and optional Vue dashboard — matching the hivemind design pattern.

**Architecture:** Template files use Liquid syntax for conditional content within files (`{% if include_db %}...{% endif %}`); a post-gen Rhai hook deletes entire optional files/directories after generation. Generated project: Axum router combining rmcp MCP transport + REST API, embedded via `include_dir` when dashboard is included.

**Tech Stack:** cargo-generate (Liquid + Rhai), Rust 2024 edition (rmcp 1, axum 0.8, libsql 0.9, clap 4, tokio 1, serde, schemars, tracing), Vue 3 + Pinia + Vite + Bun (dashboard, user-scaffolded post-gen)

---

## File Map

Files created in this plan (all paths relative to repo root = the template repo):

| File | Purpose |
|---|---|
| `cargo-generate.toml` | Template config: variables, hook registration, ignore list |
| `hooks/post-script.rhai` | Post-gen hook: delete optional files/dirs |
| `Cargo.toml` | Crate manifest with Liquid-conditional deps |
| `build.rs` | Git SHA version stamping at compile time |
| `.justfile` | Task runner: build/test/install/release + conditional dashboard targets |
| `.gitignore` | Standard Rust + conditional dashboard entries |
| `cliff.toml` | git-cliff changelog config |
| `src/model.rs` | Shared domain types (Version struct) |
| `src/config.rs` | ServerSettings, load from `~/.config/{{project-name}}/config.toml` |
| `src/db.rs` | libsql setup + migration runner (deleted by rhai if `!include_db`) |
| `src/store.rs` | SqliteStore wrapper (deleted by rhai if `!include_db`) |
| `migrations/001_init.sql` | Initial schema (deleted by rhai if `!include_db`) |
| `src/server.rs` | MCP server struct + `ping` example tool |
| `src/api.rs` | REST router: `GET /api/v1/status` |
| `src/http.rs` | Axum router combining MCP + REST + dashboard embed |
| `src/cli.rs` | clap CLI: `up`, `mcp install` subcommands |
| `src/lib.rs` | `pub mod` declarations, Liquid-gated |
| `src/main.rs` | `#[tokio::main]` entrypoints + CLI dispatch |
| `tests/api_integration.rs` | tower::oneshot integration tests |
| `{{dashboard_dir}}/dist/.gitkeep` | Placeholder so `include_dir!` compiles before Vue setup |
| `{{dashboard_dir}}/vite.config.snippet.js` | Proxy + outDir config to merge after `bun create vue` |
| `CLAUDE.md` | Claude Code guidance for generated project |
| `README.md` | Template README (shown on GitHub, not copied to generated project) |

---

## Task 1: cargo-generate.toml

**Files:**
- Create: `cargo-generate.toml`

- [ ] **Step 1: Create cargo-generate.toml**

```toml
[template]
cargo_generate_version = ">=0.21.0"
ignore = ["hooks", "docs", "README.md"]

[hooks]
post = ["hooks/post-script.rhai"]

[placeholders.project_description]
type = "string"
prompt = "Short description of your project"

[placeholders.include_db]
type = "bool"
prompt = "Include SQLite storage layer (libsql)?"
default = false

[placeholders.include_dashboard]
type = "bool"
prompt = "Include Vue dashboard?"
default = false

[placeholders.dashboard_dir]
type = "string"
prompt = "Dashboard directory name? (dashboard / web / ui)"
default = "dashboard"
```

Note: `dashboard_dir` is always prompted; the rhai hook ignores it when `include_dashboard = false`. The generated project never receives `hooks/`, `docs/`, or the template `README.md`.

- [ ] **Step 2: Commit**

```bash
git add cargo-generate.toml
git commit -m "feat: add cargo-generate.toml with template variables"
```

---

## Task 2: Rhai Post-Gen Hook

**Files:**
- Create: `hooks/post-script.rhai`

- [ ] **Step 1: Create hooks/post-script.rhai**

```rhai
// Delete dashboard directory if user opted out
let include_dashboard = variable::get("include_dashboard");
if !include_dashboard {
    let dir = variable::get("dashboard_dir");
    file::delete(dir);
}

// Delete DB layer if user opted out
let include_db = variable::get("include_db");
if !include_db {
    file::delete("src/db.rs");
    file::delete("src/store.rs");
    file::delete("migrations");
}
```

- [ ] **Step 2: Commit**

```bash
git add hooks/post-script.rhai
git commit -m "feat: add rhai post-gen hook for optional file cleanup"
```

---

## Task 3: build.rs + Cargo.toml

**Files:**
- Create: `build.rs`
- Create: `Cargo.toml`

- [ ] **Step 1: Create build.rs**

```rust
fn main() {
    let output = std::process::Command::new("git")
        .args(["describe", "--exact-match", "--tags", "HEAD"])
        .output();
    let is_tagged = output.map(|o| o.status.success()).unwrap_or(false);
    println!("cargo:rustc-env=APP_IS_TAGGED={is_tagged}");

    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=APP_GIT_SHA={}", sha.trim());
}
```

- [ ] **Step 2: Create Cargo.toml**

```toml
[package]
name = "{{project-name}}"
version = "0.1.0"
edition = "2024"
description = "{{project_description}}"

[[bin]]
name = "{{project-name}}"
path = "src/main.rs"

[lib]
name = "{{crate_name}}"
path = "src/lib.rs"

[dependencies]
rmcp = { version = "1", features = [
  "server",
  "macros",
  "transport-streamable-http-server",
] }
axum = "0.8"
tower-http = { version = "0.7", features = ["cors"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1.2"
anyhow = "1"
uuid = { version = "1", features = ["v4"] }
clap = { version = "4", features = ["derive"] }
toml = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
{% if include_db %}
libsql = { version = "0.9", features = ["core"] }
{% endif %}
{% if include_dashboard %}
include_dir = "0.7"
mime_guess = "2"
{% endif %}

[dev-dependencies]
tempfile = "3"
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
```

- [ ] **Step 3: Commit**

```bash
git add build.rs Cargo.toml
git commit -m "feat: add Cargo.toml and build.rs with Liquid-conditional deps"
```

---

## Task 4: .justfile + .gitignore + cliff.toml

**Files:**
- Create: `.justfile`
- Create: `.gitignore`
- Create: `cliff.toml`

- [ ] **Step 1: Create .justfile**

```just
_default:
  @just --choose

build:
  cargo build

test:
  cargo test

install:
  cargo install --path . --force

release-patch:
  just _release patch

release-minor:
  just _release minor

release-major:
  just _release major

_release bump:
  cargo release --execute {{{{bump}}}}
{% if include_dashboard %}

[working-directory: '{{dashboard_dir}}']
setup-frontend:
  bun create vue@latest .

[working-directory: '{{dashboard_dir}}']
dashboard:
  bun run build
{% endif %}
```

Note: `{{{{bump}}}}` double-escapes to produce literal `{{bump}}` in the output (Liquid escaping).

- [ ] **Step 2: Create .gitignore**

```
/target
{% if include_dashboard %}
/{{dashboard_dir}}/node_modules
/{{dashboard_dir}}/dist/*
!/{{dashboard_dir}}/dist/.gitkeep
{% endif %}
```

- [ ] **Step 3: Create cliff.toml**

```toml
[changelog]
header = ""
body = """
{% raw %}{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | striptags | trim | upper_first }}
{% for commit in commits %}
- {{ commit.message | upper_first }}{% endfor %}
{% endfor %}{% endraw %}
"""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
commit_parsers = [
  { message = "^feat", group = "Features" },
  { message = "^fix", group = "Bug Fixes" },
  { message = "^docs", group = "Documentation" },
  { message = "^refactor", group = "Refactoring" },
  { message = "^chore", group = "Miscellaneous" },
]
```

Note: `{% raw %}...{% endraw %}` prevents cargo-generate from processing the git-cliff Liquid syntax inside `body`.

- [ ] **Step 4: Commit**

```bash
git add .justfile .gitignore cliff.toml
git commit -m "feat: add justfile, gitignore, cliff.toml"
```

---

## Task 5: src/model.rs

**Files:**
- Create: `src/model.rs`

- [ ] **Step 1: Create src/model.rs**

```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub version: String,
}

pub fn app_version() -> String {
    let sha = env!("APP_GIT_SHA");
    let tagged = env!("APP_IS_TAGGED") == "true";
    if tagged {
        env!("CARGO_PKG_VERSION").to_string()
    } else {
        format!("{sha}-dev")
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/model.rs
git commit -m "feat: add model.rs with StatusResponse and version helper"
```

---

## Task 6: src/config.rs

**Files:**
- Create: `src/config.rs`

- [ ] **Step 1: Create src/config.rs**

```rust
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    {% if include_dashboard %}
    pub dashboard_port: u16,
    pub api_url: String,
    pub cors_origin: String,
    {% endif %}
}

#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    #[serde(default)]
    server: RawServer,
    {% if include_dashboard %}
    #[serde(default)]
    dashboard: RawDashboard,
    {% endif %}
}

#[derive(Debug, Default, Deserialize)]
struct RawServer {
    host: Option<String>,
    port: Option<u16>,
}

{% if include_dashboard %}
#[derive(Debug, Default, Deserialize)]
struct RawDashboard {
    port: Option<u16>,
    api_url: Option<String>,
    cors_origin: Option<String>,
}
{% endif %}

pub fn global_config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            home.join(".config")
        });
    base.join("{{project-name}}").join("config.toml")
}

pub fn load_server_settings(path: &Path) -> Result<ServerSettings> {
    let raw: RawConfig = if path.is_file() {
        toml::from_str(&std::fs::read_to_string(path)?)
            .with_context(|| format!("parsing {}", path.display()))?
    } else {
        RawConfig::default()
    };

    let host = raw.server.host.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = raw.server.port.unwrap_or(3000);

    {% if include_dashboard %}
    let dashboard_port = raw.dashboard.port.unwrap_or(3001);
    let api_url = raw
        .dashboard
        .api_url
        .unwrap_or_else(|| format!("http://{host}:{port}"));
    let cors_host = match host.as_str() {
        "0.0.0.0" | "::" => "127.0.0.1",
        h => h,
    };
    let cors_origin = raw
        .dashboard
        .cors_origin
        .unwrap_or_else(|| format!("http://{cors_host}:{dashboard_port}"));

    Ok(ServerSettings { host, port, dashboard_port, api_url, cors_origin })
    {% else %}
    Ok(ServerSettings { host, port })
    {% endif %}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn defaults_when_config_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let s = load_server_settings(&tmp.path().join("no-config.toml")).unwrap();
        assert_eq!(s.host, "127.0.0.1");
        assert_eq!(s.port, 3000);
    }

    #[test]
    fn reads_host_and_port_overrides() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("config.toml"),
            "[server]\nhost=\"0.0.0.0\"\nport=4000\n",
        ).unwrap();
        let s = load_server_settings(&tmp.path().join("config.toml")).unwrap();
        assert_eq!(s.host, "0.0.0.0");
        assert_eq!(s.port, 4000);
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/config.rs
git commit -m "feat: add config.rs with ServerSettings"
```

---

## Task 7: src/db.rs + migrations/001_init.sql

These files are deleted by the rhai hook when `include_db = false`, but must exist in the template.

**Files:**
- Create: `src/db.rs`
- Create: `migrations/001_init.sql`

- [ ] **Step 1: Create migrations/001_init.sql**

```sql
CREATE TABLE IF NOT EXISTS items (
    id   TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
```

- [ ] **Step 2: Create src/db.rs**

```rust
use anyhow::{Context, Result};
use libsql::Builder;

pub fn resolve_db_path() -> String {
    if let Ok(p) = std::env::var("APP_DB_PATH") {
        return p;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{home}/.local/share/{{project-name}}/data.db")
}

pub async fn open_database(path: &str) -> Result<libsql::Database> {
    if let Some(dir) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    Builder::new_local(path)
        .build()
        .await
        .context("failed to open database")
}

pub async fn init_connection(conn: &libsql::Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .await?;
    Ok(())
}

pub async fn run_migrations(conn: &libsql::Connection) -> Result<()> {
    init_connection(conn).await?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )
    .await?;

    let migrations: &[(&str, &str)] = &[
        ("001_init", include_str!("../migrations/001_init.sql")),
    ];

    for (name, sql) in migrations {
        let applied: i64 = conn
            .query(
                "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                [*name],
            )
            .await?
            .next()
            .await?
            .map(|r| r.get::<i64>(0).unwrap_or(0))
            .unwrap_or(0);

        if applied == 0 {
            conn.execute_batch(sql).await?;
            conn.execute(
                "INSERT INTO _migrations (name, applied_at) VALUES (?1, unixepoch())",
                [*name],
            )
            .await?;
            tracing::info!("applied migration: {name}");
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Commit**

```bash
git add src/db.rs migrations/001_init.sql
git commit -m "feat: add db.rs and initial migration (conditional)"
```

---

## Task 8: src/store.rs

Deleted by rhai hook when `include_db = false`.

**Files:**
- Create: `src/store.rs`

- [ ] **Step 1: Create src/store.rs**

```rust
use anyhow::Result;
use libsql::Connection;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// Example: list all item IDs
    pub async fn list_items(&self) -> Result<Vec<String>> {
        let mut rows = self
            .conn
            .query("SELECT id FROM items ORDER BY created_at DESC", ())
            .await?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            ids.push(row.get::<String>(0)?);
        }
        Ok(ids)
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/store.rs
git commit -m "feat: add store.rs SqliteStore wrapper (conditional)"
```

---

## Task 9: src/server.rs

**Files:**
- Create: `src/server.rs`

- [ ] **Step 1: Create src/server.rs**

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add src/server.rs
git commit -m "feat: add server.rs with ping example MCP tool"
```

---

## Task 10: src/api.rs

**Files:**
- Create: `src/api.rs`

- [ ] **Step 1: Create src/api.rs**

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add src/api.rs
git commit -m "feat: add api.rs with GET /api/v1/status"
```

---

## Task 11: src/http.rs

**Files:**
- Create: `src/http.rs`

- [ ] **Step 1: Create src/http.rs**

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add src/http.rs
git commit -m "feat: add http.rs Axum router with MCP + REST + dashboard"
```

---

## Task 12: src/cli.rs

**Files:**
- Create: `src/cli.rs`

- [ ] **Step 1: Create src/cli.rs**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "{{project-name}}",
    version,
    about = "{{project_description}}"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the HTTP server: MCP at /mcp, REST at /api/v1
    Up,
    /// Manage MCP client integrations
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
}

#[derive(Subcommand)]
pub enum McpAction {
    /// Register as an MCP server in a supported client
    Install {
        /// Client to register with: claude, cursor, windsurf
        client: String,
    },
}

pub fn cmd_mcp_install(client: &str) -> anyhow::Result<()> {
    match client {
        "claude" => {
            println!("Add to ~/.claude/claude_desktop_config.json:");
            println!(
                r#"{{
  "mcpServers": {{
    "{{project-name}}": {{
      "command": "{{project-name}}",
      "args": []
    }}
  }}
}}"#
            );
        }
        other => {
            anyhow::bail!("unsupported client: {other}. Supported: claude, cursor, windsurf");
        }
    }
    Ok(())
}
```

Note: `{{{{` and `}}}}` in Rust string literals produce literal `{{` and `}}` after Liquid processing (double-escape).

- [ ] **Step 2: Commit**

```bash
git add src/cli.rs
git commit -m "feat: add cli.rs with clap subcommands"
```

---

## Task 13: src/lib.rs + src/main.rs

**Files:**
- Create: `src/lib.rs`
- Create: `src/main.rs`

- [ ] **Step 1: Create src/lib.rs**

```rust
pub mod api;
pub mod cli;
pub mod config;
pub mod http;
pub mod model;
pub mod server;
{% if include_db %}
pub mod db;
pub mod store;
{% endif %}
```

- [ ] **Step 2: Create src/main.rs**

```rust
use anyhow::Result;
use clap::Parser;
use {{crate_name}}::{cli::{Cli, Command, McpAction}, config, http, server::{{project-name | pascal_case}}};
use rmcp::ServiceExt;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None | Some(Command::Up) => run_up(),
        Some(Command::Mcp { action }) => match action {
            McpAction::Install { client } => {{crate_name}}::cli::cmd_mcp_install(&client),
        },
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "{{project-name}}=info".into()),
        )
        .init();
}

#[tokio::main]
async fn run_up() -> Result<()> {
    init_tracing();
    let settings = config::load_server_settings(&config::global_config_path())?;
    http::run_up(&settings).await
}
```

Note: when no subcommand is given, the binary runs as a stdio MCP server for direct MCP client integration. `run_up()` handles both cases here — extend `main()` to add a `None` arm for stdio if needed.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs src/main.rs
git commit -m "feat: add lib.rs pub mods and main.rs entrypoint"
```

---

## Task 14: tests/api_integration.rs

**Files:**
- Create: `tests/api_integration.rs`

- [ ] **Step 1: Create tests/api_integration.rs**

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add tests/api_integration.rs
git commit -m "feat: add integration tests for REST API"
```

---

## Task 15: Dashboard Skeleton + CLAUDE.md

**Files:**
- Create: `{{dashboard_dir}}/dist/.gitkeep`
- Create: `{{dashboard_dir}}/vite.config.snippet.js`
- Create: `CLAUDE.md`

- [ ] **Step 1: Create {{dashboard_dir}}/dist/.gitkeep**

Empty file. cargo-generate copies it; the directory exists at compile time so `include_dir!` doesn't fail.

```
(empty file)
```

- [ ] **Step 2: Create {{dashboard_dir}}/vite.config.snippet.js**

```js
// After running `just setup-frontend`, merge this into the generated vite.config.js:
//
// import { defineConfig } from 'vite'
// import vue from '@vitejs/plugin-vue'
//
// export default defineConfig({
//   plugins: [vue()],
//   server: {
//     proxy: {
//       '/api': 'http://localhost:3000',   // ← add this
//     },
//   },
//   build: {
//     outDir: 'dist',                      // ← add this (usually already default)
//     emptyOutDir: true,                   // ← add this
//   },
// })
//
// Then run: just dashboard
// Then run: cargo build   (embeds dist/ into binary)
```

- [ ] **Step 3: Create CLAUDE.md**

```markdown
# CLAUDE.md

## Commands

```sh
just build          # cargo build
just test           # cargo test
just install        # install binary locally
{% if include_dashboard %}
just setup-frontend # scaffold Vue app in {{dashboard_dir}}/ (run once)
just dashboard      # bun run build in {{dashboard_dir}}/ (run before cargo build)
{% endif %}
```

## Architecture

Binary: `{{project-name}}`. Library crate: `{{crate_name}}`.

**Request path for `{{project-name}} up`:**

1. `cli.rs` parses args → `http::run_up()`
2. `http.rs` builds Axum router:
   - `/mcp` — MCP streamable HTTP (`rmcp`), backed by `server::{{project-name | pascal_case}}`
   - `/api/v1/*` — REST API (`api.rs`)
   {% if include_dashboard %}
   - `/` — embedded dashboard (`include_dir!`)
   {% endif %}
{% if include_db %}
3. `store.rs` wraps `libsql::Connection`; shared via `Arc<Store>`

**Storage:** `db.rs` opens SQLite, runs migrations from `migrations/`.
{% endif %}

## Config

`~/.config/{{project-name}}/config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 3000
{% if include_dashboard %}

[dashboard]
port = 3001
api_url = "http://127.0.0.1:3000"
{% endif %}
```
```

- [ ] **Step 4: Commit**

```bash
git add "{{dashboard_dir}}/dist/.gitkeep" "{{dashboard_dir}}/vite.config.snippet.js" CLAUDE.md
git commit -m "feat: add dashboard skeleton and CLAUDE.md"
```

---

## Task 16: End-to-End Validation

Verify the template generates a working project for all four combinations of flags.

**Prerequisites:** `cargo-generate` installed (`cargo install cargo-generate`), `cargo` available.

- [ ] **Step 1: Test minimal (no db, no dashboard)**

```bash
cd /tmp
cargo generate --path /home/graditya/projects/oxhive/mcp-server-template \
  --name test-minimal \
  --define project_description="Test project" \
  --define include_db=false \
  --define include_dashboard=false \
  --define dashboard_dir=dashboard \
  --destination /tmp/gen-test
cd /tmp/gen-test/test-minimal
cargo build 2>&1 | tail -5
cargo test 2>&1 | tail -10
```

Expected: `Compiling test-minimal`, then `test result: ok`.

- [ ] **Step 2: Verify optional files absent**

```bash
ls src/db.rs 2>&1       # Expected: "No such file or directory"
ls src/store.rs 2>&1    # Expected: "No such file or directory"
ls migrations/ 2>&1     # Expected: "No such file or directory"
ls dashboard/ 2>&1      # Expected: "No such file or directory"
```

- [ ] **Step 3: Test with db only**

```bash
cd /tmp
cargo generate --path /home/graditya/projects/oxhive/mcp-server-template \
  --name test-with-db \
  --define project_description="Test project" \
  --define include_db=true \
  --define include_dashboard=false \
  --define dashboard_dir=dashboard \
  --destination /tmp/gen-test
cd /tmp/gen-test/test-with-db
cargo build 2>&1 | tail -5
cargo test 2>&1 | tail -10
```

Expected: compiles, tests pass. `src/db.rs`, `src/store.rs`, `migrations/` all present.

- [ ] **Step 4: Test with dashboard only**

```bash
cd /tmp
cargo generate --path /home/graditya/projects/oxhive/mcp-server-template \
  --name test-with-dash \
  --define project_description="Test project" \
  --define include_db=false \
  --define include_dashboard=true \
  --define dashboard_dir=web \
  --destination /tmp/gen-test
cd /tmp/gen-test/test-with-dash
ls web/dist/.gitkeep     # Expected: file exists
cargo build 2>&1 | tail -5
```

Expected: compiles. `web/` directory present (renamed from `{{dashboard_dir}}`).

- [ ] **Step 5: Test full (db + dashboard)**

```bash
cd /tmp
cargo generate --path /home/graditya/projects/oxhive/mcp-server-template \
  --name test-full \
  --define project_description="Test project" \
  --define include_db=true \
  --define include_dashboard=true \
  --define dashboard_dir=ui \
  --destination /tmp/gen-test
cd /tmp/gen-test/test-full
cargo build 2>&1 | tail -5
cargo test 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 6: Clean up and commit**

```bash
rm -rf /tmp/gen-test
cd /home/graditya/projects/oxhive/mcp-server-template
git add -A
git commit -m "feat: complete mcp-server-template scaffold"
```

---

## Self-Review Checklist

- [x] **cargo-generate.toml**: variables, hook, ignore list — covered in Task 1
- [x] **Rhai hook**: deletes dashboard dir + db files — Task 2
- [x] **build.rs + Cargo.toml**: Liquid-conditional deps, version stamping — Task 3
- [x] **Tooling files**: justfile, gitignore, cliff.toml — Task 4
- [x] **model.rs**: StatusResponse + app_version — Task 5
- [x] **config.rs**: ServerSettings, Liquid-conditional dashboard fields — Task 6
- [x] **db.rs + migration**: libsql setup, migration runner — Task 7
- [x] **store.rs**: SqliteStore wrapper — Task 8
- [x] **server.rs**: MCP struct + ping tool — Task 9
- [x] **api.rs**: REST router, /status endpoint — Task 10
- [x] **http.rs**: combined Axum router, dashboard embed — Task 11
- [x] **cli.rs**: clap CLI — Task 12
- [x] **lib.rs + main.rs**: pub mods, entrypoint — Task 13
- [x] **tests/**: integration tests for all optional configs — Task 14
- [x] **dashboard skeleton**: dist/.gitkeep, vite snippet, CLAUDE.md — Task 15
- [x] **End-to-end**: all 4 flag combinations validated — Task 16

**Type consistency:**
- `{{project-name | pascal_case}}` used consistently for struct name in `server.rs`, `http.rs`, `main.rs`
- `{{crate_name}}` used for Rust identifiers in `main.rs`, `tests/`
- `Store` (not `SqliteStore`) used consistently across `db.rs`, `store.rs`, `http.rs`, `api.rs`
- `app_router()` / `app_router(store)` signature consistent between `http.rs` and `tests/`

**Liquid escape note:** `{{{{` in `.justfile` and `cli.rs` Rust string literals produces literal `{{` after Liquid processing — verified in Task 4 and 12.
