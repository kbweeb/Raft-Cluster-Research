use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Command {
    pub op: String, // "SET" | "DELETE"
    pub key: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub command: Command,
    pub timestamp: i64,
    pub checksum: String,
}

#[derive(Clone, Debug, Default)]
pub struct Wal {
    pub entries: Vec<LogEntry>,
}

impl Wal {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, term: u64, command: Command) -> LogEntry {
        let index = if let Some(last) = self.entries.last() {
            last.index + 1
        } else {
            1
        };
        let timestamp = chrono::Utc::now().timestamp_millis();
        let mut entry = LogEntry {
            index,
            term,
            command,
            timestamp,
            checksum: String::new(),
        };
        entry.checksum = self.calculate_checksum(&entry);
        self.entries.push(entry.clone());
        entry
    }

    fn calculate_checksum(&self, entry: &LogEntry) -> String {
        let cmd_str = serde_json::to_string(&entry.command).unwrap_or_default();
        let data = format!("{}-{}-{}-{}", entry.index, entry.term, cmd_str, entry.timestamp);
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)[..8].to_string()
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.clone()
    }

    pub fn get_entry(&self, index: u64) -> Option<&LogEntry> {
        self.entries.iter().find(|e| e.index == index)
    }

    pub fn get_last_log_index(&self) -> u64 {
        self.entries.last().map(|e| e.index).unwrap_or(0)
    }

    pub fn get_last_log_term(&self) -> u64 {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }

    pub fn truncate(&mut self, from_index: u64) {
        self.entries.retain(|e| e.index < from_index);
    }

    pub fn append_entries(&mut self, mut entries: Vec<LogEntry>) {
        self.entries.append(&mut entries);
    }

    pub fn verify_checksum(&self, entry: &LogEntry) -> bool {
        self.calculate_checksum(entry) == entry.checksum
    }
}
