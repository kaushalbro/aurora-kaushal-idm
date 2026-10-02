use crate::host_profile::HostProfile;
use aurora_core::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub destination_path: String,
    pub file_size: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: String,
    pub elapsed_seconds: f64,
    pub average_speed: f64,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Persistent SQLite database managing download history and host telemetry.
#[derive(Clone)]
pub struct HistoryDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl HistoryDatabase {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(path).map_err(|e| StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to open SQLite history DB: {}", e),
        )))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to open in-memory SQLite: {}", e),
        )))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                filename TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                file_size INTEGER,
                downloaded_bytes INTEGER NOT NULL,
                status TEXT NOT NULL,
                elapsed_seconds REAL NOT NULL,
                average_speed REAL NOT NULL,
                checksum TEXT,
                created_at TEXT NOT NULL,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS host_profiles (
                host TEXT PRIMARY KEY,
                preferred_connection_count INTEGER NOT NULL,
                average_throughput REAL NOT NULL,
                average_rtt_ms REAL NOT NULL,
                range_reliability REAL NOT NULL,
                preferred_protocol TEXT NOT NULL,
                sample_count INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("SQLite schema init error: {}", e),
        )))?;
        Ok(())
    }

    pub fn insert_download(&self, record: &DownloadRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO downloads (
                id, url, filename, destination_path, file_size, downloaded_bytes,
                status, elapsed_seconds, average_speed, checksum, created_at, completed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.id,
                record.url,
                record.filename,
                record.destination_path,
                record.file_size,
                record.downloaded_bytes,
                record.status,
                record.elapsed_seconds,
                record.average_speed,
                record.checksum,
                record.created_at.to_rfc3339(),
                record.completed_at.map(|d| d.to_rfc3339()),
            ],
        )
        .map_err(|e| StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to insert download record: {}", e),
        )))?;
        Ok(())
    }

    pub fn get_all_downloads(&self) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, url, filename, destination_path, file_size, downloaded_bytes,
                        status, elapsed_seconds, average_speed, checksum, created_at, completed_at
                 FROM downloads ORDER BY created_at DESC",
            )
            .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let rows = stmt
            .query_map([], |row| {
                let created_str: String = row.get(10)?;
                let completed_str: Option<String> = row.get(11)?;

                Ok(DownloadRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    filename: row.get(2)?,
                    destination_path: row.get(3)?,
                    file_size: row.get(4)?,
                    downloaded_bytes: row.get(5)?,
                    status: row.get(6)?,
                    elapsed_seconds: row.get(7)?,
                    average_speed: row.get(8)?,
                    checksum: row.get(9)?,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    completed_at: completed_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|d| d.with_timezone(&Utc))
                            .ok()
                    }),
                })
            })
            .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let mut results = Vec::new();
        for r in rows {
            if let Ok(rec) = r {
                results.push(rec);
            }
        }
        Ok(results)
    }

    pub fn upsert_host_profile(&self, profile: &HostProfile) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO host_profiles (
                host, preferred_connection_count, average_throughput, average_rtt_ms,
                range_reliability, preferred_protocol, sample_count, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                profile.host,
                profile.preferred_connection_count,
                profile.average_throughput,
                profile.average_rtt_ms,
                profile.range_reliability,
                profile.preferred_protocol,
                profile.sample_count,
                profile.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to upsert host profile: {}", e),
        )))?;
        Ok(())
    }

    pub fn get_host_profile(&self, host: &str) -> Result<Option<HostProfile>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT host, preferred_connection_count, average_throughput, average_rtt_ms,
                        range_reliability, preferred_protocol, sample_count, updated_at
                 FROM host_profiles WHERE host = ?1",
            )
            .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let mut rows = stmt
            .query(params![host])
            .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        if let Some(row) = rows.next().map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))? {
            let updated_str: String = row.get(7).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            Ok(Some(HostProfile {
                host: row.get(0).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
                preferred_connection_count: row.get(1).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
                average_throughput: row.get(2).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
                average_rtt_ms: row.get(3).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
                range_reliability: row.get(4).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
                preferred_protocol: row.get(5).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
                sample_count: row.get(6).map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }
}
