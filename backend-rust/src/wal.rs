use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use sqlx::{PgPool, Row};

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Command {
    pub op: String, // "SET" | "DELETE"
    pub key: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct LogEntry {
    pub index: i64, // Database uses i64 for BIGINT
    pub term: i64,
    pub command: serde_json::Value, // Store as JSONB
    pub timestamp: i64,
    pub checksum: String,
}

#[derive(Clone, Debug)]
pub struct Wal {
    pub entries: Vec<LogEntry>,
    pool: Option<PgPool>,
}

impl Wal {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self {
            entries: Vec::new(),
            pool,
        }
    }

    pub async fn load_from_db(&mut self) -> Result<(), sqlx::Error> {
        if let Some(pool) = &self.pool {
            let entries = sqlx::query_as::<_, LogEntry>("SELECT * FROM raft_logs ORDER BY index ASC")
                .fetch_all(pool)
                .await?;
            self.entries = entries;
        }
        Ok(())
    }

    pub async fn append(&mut self, term: u64, command: Command) -> LogEntry {
        let index = if let Some(last) = self.entries.last() {
            last.index + 1
        } else {
            1
        };
        let timestamp = chrono::Utc::now().timestamp_millis();
        let cmd_val = serde_json::to_value(&command).unwrap();
        
        let mut entry = LogEntry {
            index,
            term: term as i64,
            command: cmd_val,
            timestamp,
            checksum: String::new(),
        };
        entry.checksum = self.calculate_checksum(&entry);
        
        if let Some(pool) = &self.pool {
            let _ = sqlx::query(
                "INSERT INTO raft_logs (index, term, command, timestamp, checksum) VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(entry.index)
            .bind(entry.term)
            .bind(&entry.command)
            .bind(entry.timestamp)
            .bind(&entry.checksum)
            .execute(pool)
            .await;
        }

        self.entries.push(entry.clone());
        entry
    }

    fn calculate_checksum(&self, entry: &LogEntry) -> String {
        let cmd_str = entry.command.to_string();
        let data = format!("{}-{}-{}-{}", entry.index, entry.term, cmd_str, entry.timestamp);
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)[..8].to_string()
    }

    pub fn get_entries(&self) -> Vec<serde_json::Value> {
        self.entries.iter().map(|e| {
            serde_json::json!({
                "index": e.index,
                "term": e.term,
                "command": e.command,
                "timestamp": e.timestamp,
                "checksum": e.checksum
            })
        }).collect()
    }

    pub fn get_last_log_index(&self) -> u64 {
        self.entries.last().map(|e| e.index as u64).unwrap_or(0)
    }

    pub fn get_last_log_term(&self) -> u64 {
        self.entries.last().map(|e| e.term as u64).unwrap_or(0)
    }

    pub async fn truncate(&mut self, from_index: u64) {
        if let Some(pool) = &self.pool {
            let _ = sqlx::query("DELETE FROM raft_logs WHERE index >= $1")
                .bind(from_index as i64)
                .execute(pool)
                .await;
        }
        self.entries.retain(|e| e.index < from_index as i64);
    }

    pub async fn append_entries(&mut self, entries: Vec<LogEntry>) {
        if let Some(pool) = &self.pool {
            for entry in &entries {
                let _ = sqlx::query(
                    "INSERT INTO raft_logs (index, term, command, timestamp, checksum) 
                     VALUES ($1, $2, $3, $4, $5) 
                     ON CONFLICT (index) DO UPDATE SET term = $2, command = $3, timestamp = $4, checksum = $5"
                )
                .bind(entry.index)
                .bind(entry.term)
                .bind(&entry.command)
                .bind(entry.timestamp)
                .bind(&entry.checksum)
                .execute(pool)
                .await;
            }
        }
        // Simplified: in a real system we'd merge carefully
        self.entries.extend(entries);
        self.entries.sort_by_key(|e| e.index);
        self.entries.dedup_by_key(|e| e.index);
    }
}
