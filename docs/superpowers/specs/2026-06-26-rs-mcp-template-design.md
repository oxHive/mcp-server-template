# rs-mcp-template Design Spec

Date: 2026-06-26

## Purpose

A `cargo-generate` template for future Rust MCP server projects following the same design pattern as [hivemind](../../../projects/oxhive/hivemind): Rust binary with embedded MCP server + REST API, optional SQLite storage, optional Vue dashboard.

## Template variables

| Variable | Type | Prompt | Default |
|---|---|---|---|
| `project-name` | string | cargo-generate built-in | — |
| `project_description` | string | "Short description of your project" | — |
| `include_db` | bool | "Include SQLite storage layer (libsql)?" | false |
| `include_dashboard` | bool | "Include Vue dashboard?" | false |
| `dashboard_dir` | string | "Dashboard directory name?" | "dashboard" |

`dashboard_dir` uses `depends_on = ["include_dashboard"]` in `cargo-generate.toml` so it only appears when `include_dashboard = true`.

## Template mechanism

- **Liquid templates**: conditional content *within* files (`Cargo.toml` deps, `lib.rs` mod declarations, `http.rs` router setup, `.justfile` targets)
- **Rhai post-gen hook** (`hooks/post-script.rhai`): deletes entire optional directories/files after generation
  - Deletes `{{dashboard_dir}}/` if `include_dashboard = false`
  - Deletes `src/db.rs`, `src/store.rs`, `migrations/` if `include_db = false`

## File structure

```
{{project-name}}/
├── cargo-generate.toml
├── hooks/
│   └── post-script.rhai
├── Cargo.toml
├── build.rs                      # git SHA version stamping
├── .justfile                     # build, test, install, release-*, conditional frontend targets
├── .gitignore
├── cliff.toml
├── migrations/                   # [deleted by rhai if !include_db]
│   └── 001_init.sql
├── src/
│   ├── main.rs                   # tokio entrypoints + CLI dispatch
│   ├── lib.rs                    # pub mod declarations (Liquid-gated)
│   ├── cli.rs                    # clap CLI: `up`, `mcp install`
│   ├── config.rs                 # ServerSettings, loads ~/.config/{{project-name}}/config.toml
│   ├── http.rs                   # Axum router: /mcp + /api/v1 + dashboard embed (Liquid-gated)
│   ├── api.rs                    # REST router: GET /api/v1/status example
│   ├── server.rs                 # MCP server struct + `ping` example tool
│   ├── model.rs                  # shared domain types
│   ├── db.rs                     # [deleted by rhai if !include_db] libsql setup + migrations
│   └── store.rs                  # [deleted by rhai if !include_db] SqliteStore wrapper
├── tests/
│   └── api_integration.rs        # tower::oneshot integration test skeleton
└── {{dashboard_dir}}/            # [deleted by rhai if !include_dashboard]
    ├── dist/.gitkeep             # placeholder so cargo build doesn't fail before Vue setup
    └── vite.config.snippet.js    # proxy + outDir config to merge after `bun create vue`
```

## Rust architecture

Follows hivemind's request path:

1. `cli.rs` parses args → dispatches to `http::run_up()` or stdio MCP
2. `http.rs` builds Axum router:
   - `/mcp` — `rmcp` streamable HTTP transport backed by `server::{{ProjectName}}`
   - `/api/v1/*` — REST handlers (`api.rs`)
   - `/` + static assets — embedded dashboard (`include_dir`, conditional)
3. `store.rs` (optional) wraps `libsql::Connection`; both surfaces share `Arc<Store>`

Binary name and crate name derived from `{{project-name}}`.

### Key dependencies

```toml
rmcp = { version = "1", features = ["server", "macros", "transport-streamable-http-server"] }
axum = "0.8"
tower-http = { version = "0.7", features = ["cors"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1.2"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
toml = "1.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
# conditional:
libsql = { version = "0.9", features = ["core"] }       # include_db only
include_dir = "0.7"                                      # include_dashboard only
mime_guess = "2"                                         # include_dashboard only
```

### Example MCP tool (`server.rs`)

`ping` tool: takes `message: String`, returns `{"echo": "<message>"}`. Demonstrates `tool!`, `tool_handler!`, `tool_router!` macros from `rmcp`.

### Example REST endpoint (`api.rs`)

`GET /api/v1/status` → `{"status": "ok", "version": "{{project-name}} <git-sha>"}`.

## Dashboard (optional)

Uses **Option Y**: template provides scaffold hooks, user runs `bun create vue@latest` interactively.

Post-`cargo generate` workflow when `include_dashboard = true`:
1. `just setup-frontend` — runs `bun create vue@latest {{dashboard_dir}}` (interactive: user picks TS, router, pinia, etc.)
2. Merge `{{dashboard_dir}}/vite.config.snippet.js` into the generated `vite.config.js` (adds dev proxy `/api → localhost:PORT` and `build.outDir = 'dist'`)
3. `just dashboard` — `cd {{dashboard_dir}} && bun run build`
4. `cargo build` — embeds `{{dashboard_dir}}/dist` via `include_dir!`

`vite.config.snippet.js` content:
```js
// Merge this into your generated vite.config.js:
// server.proxy: { '/api': 'http://localhost:3000' }
// build.outDir: 'dist'
```

The `dist/.gitkeep` placeholder allows `cargo build` to succeed before Vue is set up (embed compiles to an empty dir).

## Config

Global config at `~/.config/{{project-name}}/config.toml`:
```toml
[server]
host = "127.0.0.1"
port = 3000

[dashboard]
port = 3001
api_url = "http://127.0.0.1:3000"
```

## Tooling

- `just` task runner (`.justfile`)
- `cargo-release` for versioning
- `cliff` for changelogs
- `bun` + Vite for dashboard (conditional)
- Integration tests via `tower::oneshot` (no network required)

## Rhai hook details

`hooks/post-script.rhai`:
```rhai
// Delete dashboard dir if not wanted
if !variable::get("include_dashboard") {
    file::delete("{{dashboard_dir}}");
}

// Delete DB files if not wanted
if !variable::get("include_db") {
    file::delete("src/db.rs");
    file::delete("src/store.rs");
    file::delete("migrations");
}
```

## Out of scope

- Sync/replication layer (hivemind-specific)
- Service install commands (`systemd`/`launchd`)
- Token budget / tiktoken (hivemind-specific)
