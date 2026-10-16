use serde::{Deserialize, Serialize};
use crate::wal::{Wal, LogEntry, Command};
use std::collections::{HashMap, HashSet};
use rand::Rng;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NodeRole {
    LEADER,
    FOLLOWER,
    CANDIDATE,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RaftMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub from: String,
    pub to: String,
    pub term: u64,
    pub data: serde_json::Value,
}

use sqlx::PgPool;

pub struct RaftNode {
    pub id: String,
    pub role: NodeRole,
    pub current_term: u64,
    pub voted_for: Option<String>,
    pub log: Wal,
    pub commit_index: u64,
    pub last_applied: u64,
    pub state_machine: HashMap<String, serde_json::Value>,
    pub peers: Vec<String>,
    pub is_alive: bool,
    
    pub next_index: HashMap<String, u64>,
    pub match_index: HashMap<String, u64>,
    pub votes: HashSet<String>,
    pub pool: Option<PgPool>,
}

impl RaftNode {
    pub fn new(id: String, peers: Vec<String>, pool: Option<PgPool>) -> Self {
        Self {
            id,
            role: NodeRole::FOLLOWER,
            current_term: 0,
            voted_for: None,
            log: Wal::new(pool.clone()),
            commit_index: 0,
            last_applied: 0,
            state_machine: HashMap::new(),
            peers,
            is_alive: true,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            votes: HashSet::new(),
            pool,
        }
    }

    pub fn kill(&mut self) {
        self.is_alive = false;
    }

    pub fn revive(&mut self) {
        self.is_alive = true;
        self.role = NodeRole::FOLLOWER;
    }

    pub fn tick(&mut self) -> Vec<RaftMessage> {
        // Ticking logic is handled by the Cluster/Server with timers
        Vec::new()
    }

    pub fn start_election(&mut self) -> Vec<RaftMessage> {
        if !self.is_alive { return Vec::new(); }
        self.role = NodeRole::CANDIDATE;
        self.current_term += 1;
        self.voted_for = Some(self.id.clone());
        self.votes.clear();
        self.votes.insert(self.id.clone());

        let mut messages = Vec::new();
        for peer in &self.peers {
            messages.push(RaftMessage {
                msg_type: "RequestVote".to_string(),
                from: self.id.clone(),
                to: peer.clone(),
                term: self.current_term,
                data: serde_json::json!({
                    "lastLogIndex": self.log.get_last_log_index(),
                    "lastLogTerm": self.log.get_last_log_term()
                }),
            });
        }
        messages
    }

    pub async fn handle_message(&mut self, msg: RaftMessage) -> Vec<RaftMessage> {
        if !self.is_alive { return Vec::new(); }

        if msg.term > self.current_term {
            self.current_term = msg.term;
            self.role = NodeRole::FOLLOWER;
            self.voted_for = None;
        }

        match msg.msg_type.as_str() {
            "RequestVote" => self.handle_request_vote(msg),
            "RequestVoteResponse" => self.handle_request_vote_response(msg),
            "AppendEntries" => self.handle_append_entries(msg).await,
            "AppendEntriesResponse" => self.handle_append_entries_response(msg),
            _ => Vec::new(),
        }
    }

    fn handle_request_vote(&mut self, msg: RaftMessage) -> Vec<RaftMessage> {
        let mut vote_granted = false;
        let last_log_index = msg.data["lastLogIndex"].as_u64().unwrap_or(0);
        let last_log_term = msg.data["lastLogTerm"].as_u64().unwrap_or(0);

        let log_ok = (last_log_term > self.log.get_last_log_term()) ||
                     (last_log_term == self.log.get_last_log_term() && last_log_index >= self.log.get_last_log_index());

        if msg.term >= self.current_term && (self.voted_for.is_none() || self.voted_for == Some(msg.from.clone())) && log_ok {
            vote_granted = true;
            self.voted_for = Some(msg.from.clone());
        }

        vec![RaftMessage {
            msg_type: "RequestVoteResponse".to_string(),
            from: self.id.clone(),
            to: msg.from,
            term: self.current_term,
            data: serde_json::json!({ "voteGranted": vote_granted }),
        }]
    }

    fn handle_request_vote_response(&mut self, msg: RaftMessage) -> Vec<RaftMessage> {
        if self.role != NodeRole::CANDIDATE || msg.term != self.current_term { return Vec::new(); }
        
        if msg.data["voteGranted"].as_bool().unwrap_or(false) {
            self.votes.insert(msg.from);
            if self.votes.size_hint().0 + 1 > (self.peers.len() + 1) / 2 {
                // Actually, the size is just self.votes.len()
            }
            if self.votes.len() > (self.peers.len() + 1) / 2 {
                return self.become_leader();
            }
        }
        Vec::new()
    }

    fn become_leader(&mut self) -> Vec<RaftMessage> {
        self.role = NodeRole::LEADER;
        self.votes.clear();
        self.next_index.clear();
        self.match_index.clear();
        for peer in &self.peers {
            self.next_index.insert(peer.clone(), self.log.get_last_log_index() + 1);
            self.match_index.insert(peer.clone(), 0);
        }
        self.send_heartbeats()
    }

    pub fn send_heartbeats(&mut self) -> Vec<RaftMessage> {
        if !self.is_alive || self.role != NodeRole::LEADER { return Vec::new(); }
        let mut messages = Vec::new();
        for peer in &self.peers {
            messages.push(RaftMessage {
                msg_type: "AppendEntries".to_string(),
                from: self.id.clone(),
                to: peer.clone(),
                term: self.current_term,
                data: serde_json::json!({
                    "prevLogIndex": self.log.get_last_log_index(),
                    "prevLogTerm": self.log.get_last_log_term(),
                    "entries": [],
                    "leaderCommit": self.commit_index
                }),
            });
        }
        messages
    }

    pub async fn handle_append_entries(&mut self, msg: RaftMessage) -> Vec<RaftMessage> {
        if msg.term < self.current_term {
            return vec![RaftMessage {
                msg_type: "AppendEntriesResponse".to_string(),
                from: self.id.clone(),
                to: msg.from,
                term: self.current_term,
                data: serde_json::json!({ "success": false, "lastLogIndex": self.log.get_last_log_index() }),
            }];
        }

        self.role = NodeRole::FOLLOWER;
        
        let prev_log_index = msg.data["prevLogIndex"].as_u64().unwrap_or(0);
        let prev_log_term = msg.data["prevLogTerm"].as_u64().unwrap_or(0);
        let entries: Vec<LogEntry> = serde_json::from_value(msg.data["entries"].clone()).unwrap_or_default();
        let leader_commit = msg.data["leaderCommit"].as_u64().unwrap_or(0);

        let prev_entry = self.log.entries.iter().find(|e| e.index == prev_log_index as i64);
        if prev_log_index > 0 && (prev_entry.is_none() || prev_entry.unwrap().term != prev_log_term as i64) {
            return vec![RaftMessage {
                msg_type: "AppendEntriesResponse".to_string(),
                from: self.id.clone(),
                to: msg.from,
                term: self.current_term,
                data: serde_json::json!({ "success": false, "lastLogIndex": self.log.get_last_log_index() }),
            }];
        }

        if !entries.is_empty() {
            self.log.truncate(entries[0].index as u64).await;
            self.log.append_entries(entries).await;
        }

        if leader_commit > self.commit_index {
            self.commit_index = leader_commit.min(self.log.get_last_log_index());
            self.apply_to_state_machine();
        }

        vec![RaftMessage {
            msg_type: "AppendEntriesResponse".to_string(),
            from: self.id.clone(),
            to: msg.from,
            term: self.current_term,
            data: serde_json::json!({ "success": true, "lastLogIndex": self.log.get_last_log_index() }),
        }]
    }

    pub async fn propose(&mut self, command: Command) -> bool {
        if self.role != NodeRole::LEADER || !self.is_alive { return false; }
        self.log.append(self.current_term, command).await;
        true
    }

    pub fn replicate_log(&mut self) -> Vec<RaftMessage> {
        if self.role != NodeRole::LEADER || !self.is_alive { return Vec::new(); }
        let mut messages = Vec::new();
        for peer in &self.peers {
            let next_idx = *self.next_index.get(peer).unwrap_or(&(self.log.get_last_log_index() + 1));
            let entries: Vec<LogEntry> = self.log.entries.iter().filter(|e| e.index >= next_idx as i64).cloned().collect();
            let prev_entry = self.log.entries.iter().find(|e| e.index == (next_idx - 1) as i64);

            messages.push(RaftMessage {
                msg_type: "AppendEntries".to_string(),
                from: self.id.clone(),
                to: peer.clone(),
                term: self.current_term,
                data: serde_json::json!({
                    "prevLogIndex": prev_entry.map(|e| e.index).unwrap_or(0),
                    "prevLogTerm": prev_entry.map(|e| e.term).unwrap_or(0),
                    "entries": entries,
                    "leaderCommit": self.commit_index
                }),
            });
        }
        messages
    }

    fn handle_append_entries_response(&mut self, msg: RaftMessage) -> Vec<RaftMessage> {
        if self.role != NodeRole::LEADER || msg.term != self.current_term { return Vec::new(); }

        let success = msg.data["success"].as_bool().unwrap_or(false);
        let last_log_index = msg.data["lastLogIndex"].as_u64().unwrap_or(0);

        if success {
            self.match_index.insert(msg.from.clone(), last_log_index);
            self.next_index.insert(msg.from, last_log_index + 1);
            self.update_commit_index();
        } else {
            let next = self.next_index.get(&msg.from).unwrap_or(&1);
            self.next_index.insert(msg.from, (*next).saturating_sub(1).max(1));
            return self.replicate_log();
        }
        Vec::new()
    }

    fn update_commit_index(&mut self) {
        let mut indices: Vec<u64> = self.match_index.values().cloned().collect();
        indices.push(self.log.get_last_log_index());
        indices.sort();
        let majority_index = indices[indices.len() / 2];
        if majority_index > self.commit_index {
            if let Some(entry) = self.log.get_entry(majority_index) {
                if entry.term == self.current_term {
                    self.commit_index = majority_index;
                    self.apply_to_state_machine();
                }
            }
        }
    }

    fn apply_to_state_machine(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            let entry = self.log.entries.iter().find(|e| e.index == self.last_applied as i64).cloned();
            if let Some(e) = entry {
                let cmd: Command = serde_json::from_value(e.command).unwrap();
                match cmd.op.as_str() {
                    "SET" => {
                        if let Some(val) = cmd.value {
                            self.state_machine.insert(cmd.key, val);
                        }
                    }
                    "DELETE" => {
                        self.state_machine.remove(&cmd.key);
                    }
                    _ => {}
                }
            }
        }
    }
}
