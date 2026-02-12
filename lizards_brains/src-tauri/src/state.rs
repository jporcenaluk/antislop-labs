use crate::timer::{Origin, Session, SessionStatus};
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct StateManager {
    conn: Mutex<Connection>,
}

impl StateManager {
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        let manager = StateManager {
            conn: Mutex::new(conn),
        };
        manager.run_migrations()?;
        Ok(manager)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let manager = StateManager {
            conn: Mutex::new(conn),
        };
        manager.run_migrations()?;
        Ok(manager)
    }

    fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY NOT NULL,
                label TEXT NOT NULL CHECK(length(label) >= 1 AND length(label) <= 64),
                duration_secs INTEGER NOT NULL CHECK(duration_secs > 0),
                started_at TEXT NOT NULL,
                ended_at TEXT,
                origin TEXT NOT NULL CHECK(origin IN ('Human', 'Agent')),
                status TEXT NOT NULL CHECK(status IN ('Running', 'Completed', 'Stopped'))
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);",
        )?;
        Ok(())
    }

    pub fn save_session(&self, session: &Session) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, label, duration_secs, started_at, ended_at, origin, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.id,
                session.label,
                session.duration_secs,
                session.started_at,
                session.ended_at,
                format!("{:?}", session.origin),
                format!("{:?}", session.status),
            ],
        )?;
        Ok(())
    }

    pub fn update_session(
        &self,
        id: &str,
        status: &SessionStatus,
        ended_at: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET status = ?1, ended_at = ?2 WHERE id = ?3",
            params![format!("{:?}", status), ended_at, id],
        )?;
        Ok(())
    }

    pub fn get_history(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Vec<Session>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut query = "SELECT id, label, duration_secs, started_at, ended_at, origin, status FROM sessions WHERE 1=1".to_string();
        let mut param_values: Vec<String> = Vec::new();

        if let Some(start) = start_date {
            query.push_str(&format!(" AND started_at >= ?{}", param_values.len() + 1));
            param_values.push(start.to_string());
        }
        if let Some(end) = end_date {
            query.push_str(&format!(" AND started_at <= ?{}", param_values.len() + 1));
            param_values.push(end.to_string());
        }
        query.push_str(" ORDER BY started_at DESC");

        let mut stmt = conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();

        let sessions = stmt.query_map(params.as_slice(), |row| {
            let origin_str: String = row.get(5)?;
            let status_str: String = row.get(6)?;
            Ok(Session {
                id: row.get(0)?,
                label: row.get(1)?,
                duration_secs: row.get(2)?,
                started_at: row.get(3)?,
                ended_at: row.get(4)?,
                origin: match origin_str.as_str() {
                    "Agent" => Origin::Agent,
                    _ => Origin::Human,
                },
                status: match status_str.as_str() {
                    "Completed" => SessionStatus::Completed,
                    "Stopped" => SessionStatus::Stopped,
                    _ => SessionStatus::Running,
                },
            })
        })?;

        sessions.collect()
    }

    pub fn cleanup_stale_running(&self) -> Result<usize, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "UPDATE sessions SET status = 'Stopped', ended_at = datetime('now') WHERE status = 'Running'",
            [],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(id: &str, label: &str, origin: Origin, status: SessionStatus) -> Session {
        Session {
            id: id.to_string(),
            label: label.to_string(),
            duration_secs: 1500,
            started_at: "2024-01-01T12:00:00Z".to_string(),
            ended_at: None,
            origin,
            status,
        }
    }

    #[test]
    fn test_create_and_query() {
        let mgr = StateManager::in_memory().unwrap();
        let session = make_session("s1", "Work", Origin::Human, SessionStatus::Running);
        mgr.save_session(&session).unwrap();

        let history = mgr.get_history(None, None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].label, "Work");
    }

    #[test]
    fn test_update_session() {
        let mgr = StateManager::in_memory().unwrap();
        let session = make_session("s1", "Work", Origin::Human, SessionStatus::Running);
        mgr.save_session(&session).unwrap();

        mgr.update_session("s1", &SessionStatus::Completed, "2024-01-01T12:25:00Z")
            .unwrap();

        let history = mgr.get_history(None, None).unwrap();
        assert_eq!(history[0].status, SessionStatus::Completed);
        assert_eq!(history[0].ended_at.as_deref(), Some("2024-01-01T12:25:00Z"));
    }

    #[test]
    fn test_cleanup_stale_running() {
        let mgr = StateManager::in_memory().unwrap();
        let s1 = make_session("s1", "Stale", Origin::Human, SessionStatus::Running);
        let mut s2 = make_session("s2", "Done", Origin::Human, SessionStatus::Completed);
        s2.ended_at = Some("2024-01-01T12:25:00Z".to_string());
        mgr.save_session(&s1).unwrap();
        mgr.save_session(&s2).unwrap();

        let cleaned = mgr.cleanup_stale_running().unwrap();
        assert_eq!(cleaned, 1);

        let history = mgr.get_history(None, None).unwrap();
        assert!(history.iter().all(|s| s.status != SessionStatus::Running));
    }

    #[test]
    fn test_date_filtering() {
        let mgr = StateManager::in_memory().unwrap();
        let mut s1 = make_session("s1", "Early", Origin::Human, SessionStatus::Completed);
        s1.started_at = "2024-01-01T10:00:00Z".to_string();
        s1.ended_at = Some("2024-01-01T10:25:00Z".to_string());
        let mut s2 = make_session("s2", "Late", Origin::Human, SessionStatus::Completed);
        s2.started_at = "2024-01-02T10:00:00Z".to_string();
        s2.ended_at = Some("2024-01-02T10:25:00Z".to_string());

        mgr.save_session(&s1).unwrap();
        mgr.save_session(&s2).unwrap();

        let filtered = mgr.get_history(Some("2024-01-02T00:00:00Z"), None).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].label, "Late");
    }

    #[test]
    fn test_agent_origin_persistence() {
        let mgr = StateManager::in_memory().unwrap();
        let session = make_session("s1", "AI Work", Origin::Agent, SessionStatus::Completed);
        mgr.save_session(&session).unwrap();

        let history = mgr.get_history(None, None).unwrap();
        assert_eq!(history[0].origin, Origin::Agent);
    }

    #[test]
    fn test_empty_history() {
        let mgr = StateManager::in_memory().unwrap();
        let history = mgr.get_history(None, None).unwrap();
        assert!(history.is_empty());
    }
}
