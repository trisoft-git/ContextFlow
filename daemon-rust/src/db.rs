use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub session_id: String,
    pub event_type: String,
    pub content: String,
    pub metadata: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                status TEXT,
                start_time DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT,
                event_type TEXT,
                content TEXT,
                metadata TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        // Ensure at least one session exists
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, status) VALUES ('default', 'active')",
            [],
        )?;
        
        Ok(Database { conn })
    }

    pub fn record_event(&self, event: &Event) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO events (session_id, event_type, content, metadata) VALUES (?1, ?2, ?3, ?4)",
            (&event.session_id, &event.event_type, &event.content, &event.metadata),
        )?;
        Ok(())
    }

    pub fn get_recent_events(&self, limit: usize) -> Result<Vec<Event>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, event_type, content, metadata FROM events ORDER BY timestamp DESC, id DESC LIMIT ?1"
        )?;
        let event_iter = stmt.query_map([limit], |row| {
            Ok(Event {
                session_id: row.get(0)?,
                event_type: row.get(1)?,
                content: row.get(2)?,
                metadata: row.get(3)?,
            })
        })?;

        let mut events = Vec::new();
        for event in event_iter {
            events.push(event?);
        }
        Ok(events)
    }

    pub fn get_active_session_id(&self) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT id FROM sessions WHERE status = 'active' ORDER BY start_time DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_event_count(&self) -> Result<usize> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM events")?;
        let count: usize = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_event_recording_and_metadata() {
        // 1. 메모리 내 SQLite 테스트 데이터베이스 생성
        let conn = Connection::open_in_memory().unwrap();
        
        // 스키마 초기화
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                status TEXT,
                start_time DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT,
                event_type TEXT,
                content TEXT,
                metadata TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        ).unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, status) VALUES ('default', 'active')",
            [],
        ).unwrap();

        let db = Database { conn };

        // 2. 활성 세션 정상 로드 확인
        let active_session = db.get_active_session_id().unwrap();
        assert_eq!(active_session, Some("default".to_string()));

        // 3. 메타데이터를 포함한 이벤트 기록 검증
        let test_metadata = Some("{\"exitCode\":0}".to_string());
        db.record_event(&Event {
            session_id: "default".to_string(),
            event_type: "terminal_command".to_string(),
            content: "cargo check".to_string(),
            metadata: test_metadata.clone(),
        }).unwrap();

        // 4. 이벤트 역직렬화 및 값 일치 확인
        let recent = db.get_recent_events(5).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].event_type, "terminal_command");
        assert_eq!(recent[0].content, "cargo check");
        assert_eq!(recent[0].metadata, test_metadata);
    }
}
