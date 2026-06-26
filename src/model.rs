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
