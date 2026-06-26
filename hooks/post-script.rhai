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
