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
