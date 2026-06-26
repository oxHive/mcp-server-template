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
