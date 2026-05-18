use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use rmcp::transport::streamable_http_server::session::SessionState;
use rmcp::transport::streamable_http_server::session::SessionStore;
use rmcp::transport::streamable_http_server::session::SessionStoreError;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct SqliteSessionStore {
    path: PathBuf,
    ttl_seconds: Option<i64>,
}

impl SqliteSessionStore {
    pub fn new(
        path: impl Into<PathBuf>,
        ttl_seconds: Option<i64>,
    ) -> Result<Self, SessionStoreError> {
        let store = Self {
            path: path.into(),
            ttl_seconds,
        };
        store.ensure_parent_dir()?;
        store.with_connection(|connection| {
            connection.pragma_update(None, "journal_mode", "WAL")?;
            connection.pragma_update(None, "busy_timeout", 5000)?;
            connection.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS mcp_sessions (
                    session_id TEXT PRIMARY KEY,
                    state_json TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    expires_at INTEGER
                );
                CREATE INDEX IF NOT EXISTS idx_mcp_sessions_expires_at
                    ON mcp_sessions(expires_at);
                "#,
            )?;
            Ok(())
        })?;
        Ok(store)
    }

    fn ensure_parent_dir(&self) -> Result<(), SessionStoreError> {
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(box_error)?;
        }
        Ok(())
    }

    fn with_connection<T>(
        &self,
        action: impl FnOnce(&Connection) -> Result<T, SessionStoreError>,
    ) -> Result<T, SessionStoreError> {
        let connection = Connection::open(&self.path).map_err(box_error)?;
        connection
            .pragma_update(None, "busy_timeout", 5000)
            .map_err(box_error)?;
        action(&connection)
    }

    fn expires_at(&self, now: i64) -> Option<i64> {
        self.ttl_seconds.filter(|ttl| *ttl > 0).map(|ttl| now + ttl)
    }
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    async fn load(&self, session_id: &str) -> Result<Option<SessionState>, SessionStoreError> {
        let now = unix_timestamp();
        self.with_connection(|connection| {
            delete_expired(connection, now)?;
            let state_json = connection
                .query_row(
                    "SELECT state_json FROM mcp_sessions WHERE session_id = ?1 AND (expires_at IS NULL OR expires_at > ?2)",
                    params![session_id, now],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(box_error)?;

            state_json
                .map(|value| serde_json::from_str(&value).map_err(box_error))
                .transpose()
        })
    }

    async fn store(&self, session_id: &str, state: &SessionState) -> Result<(), SessionStoreError> {
        let now = unix_timestamp();
        let expires_at = self.expires_at(now);
        let state_json = serde_json::to_string(state).map_err(box_error)?;

        self.with_connection(|connection| {
            connection
                .execute(
                    r#"
                    INSERT INTO mcp_sessions (session_id, state_json, created_at, updated_at, expires_at)
                    VALUES (?1, ?2, ?3, ?3, ?4)
                    ON CONFLICT(session_id) DO UPDATE SET
                        state_json = excluded.state_json,
                        updated_at = excluded.updated_at,
                        expires_at = excluded.expires_at
                    "#,
                    params![session_id, state_json, now, expires_at],
                )
                .map_err(box_error)?;
            Ok(())
        })
    }

    async fn delete(&self, session_id: &str) -> Result<(), SessionStoreError> {
        self.with_connection(|connection| {
            connection
                .execute(
                    "DELETE FROM mcp_sessions WHERE session_id = ?1",
                    params![session_id],
                )
                .map_err(box_error)?;
            Ok(())
        })
    }
}

fn delete_expired(connection: &Connection, now: i64) -> Result<(), SessionStoreError> {
    connection
        .execute(
            "DELETE FROM mcp_sessions WHERE expires_at IS NOT NULL AND expires_at <= ?1",
            params![now],
        )
        .map_err(box_error)?;
    Ok(())
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn box_error(error: impl std::error::Error + Send + Sync + 'static) -> SessionStoreError {
    Box::new(error)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rmcp::model::ClientCapabilities;
    use rmcp::model::Implementation;
    use rmcp::model::InitializeRequestParams;

    use super::*;

    #[tokio::test]
    async fn stores_loads_and_deletes_session_state() {
        let path = temp_db_path("roundtrip");
        let store = SqliteSessionStore::new(&path, Some(60)).unwrap();
        let state = session_state();

        store.store("session-1", &state).await.unwrap();
        let loaded = store.load("session-1").await.unwrap().unwrap();

        assert_eq!(loaded.initialize_params.client_info.name, "test-client");

        store.delete("session-1").await.unwrap();
        assert!(store.load("session-1").await.unwrap().is_none());
        cleanup_db(&path);
    }

    #[tokio::test]
    async fn session_state_survives_store_recreation() {
        let path = temp_db_path("recreate");
        let state = session_state();

        SqliteSessionStore::new(&path, Some(60))
            .unwrap()
            .store("session-1", &state)
            .await
            .unwrap();
        let loaded = SqliteSessionStore::new(&path, Some(60))
            .unwrap()
            .load("session-1")
            .await
            .unwrap();

        assert!(loaded.is_some());
        cleanup_db(&path);
    }

    fn session_state() -> SessionState {
        SessionState::new(InitializeRequestParams::new(
            ClientCapabilities::default(),
            Implementation::new("test-client", "0"),
        ))
    }

    fn temp_db_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("supernode-mcp-{name}-{unique}.sqlite3"))
    }

    fn cleanup_db(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{}", path.display(), suffix));
        }
    }
}
