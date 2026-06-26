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
