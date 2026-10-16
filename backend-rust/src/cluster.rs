use crate::raft_node::{RaftNode, RaftMessage, NodeRole};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::Rng;
use serde::Serialize;
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize)]
pub struct MessageInFlight {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub msg_type: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClusterDiff {
    pub nodes: Vec<serde_json::Value>,
    pub in_flight: Vec<MessageInFlight>,
}

use sqlx::PgPool;

pub struct Cluster {
    pub nodes: HashMap<String, Arc<Mutex<RaftNode>>>,
    pub partitions: Vec<Vec<String>>,
    pub latency: u64,
    pub drop_rate: f64,
    pub message_queue: Vec<(RaftMessage, u64)>,
    pub tx: broadcast::Sender<serde_json::Value>,
    pub pool: Option<PgPool>,
}

impl Cluster {
    pub fn new(node_ids: Vec<String>, tx: broadcast::Sender<serde_json::Value>, pool: Option<PgPool>) -> Arc<Mutex<Self>> {
        let mut nodes = HashMap::new();
        for id in &node_ids {
            let peers: Vec<String> = node_ids.iter().filter(|&p| p != id).cloned().collect();
            nodes.insert(id.clone(), Arc::new(Mutex::new(RaftNode::new(id.clone(), peers, pool.clone()))));
        }

        Arc::new(Mutex::new(Self {
            nodes,
            partitions: Vec::new(),
            latency: 0,
            drop_rate: 0.0,
            message_queue: Vec::new(),
            tx,
            pool,
        }))
    }

    pub fn enqueue_message(&mut self, msg: RaftMessage) {
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.drop_rate { return; }

        if !self.partitions.is_empty() {
            let sender_partition = self.partitions.iter().find(|p| p.contains(&msg.from));
            let receiver_partition = self.partitions.iter().find(|p| p.contains(&msg.to));
            if sender_partition != receiver_partition { return; }
        }

        let deliver_at = chrono::Utc::now().timestamp_millis() as u64 + self.latency + rng.gen_range(0..20);
        self.message_queue.push((msg, deliver_at));
    }

    pub async fn process_messages(&mut self) {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let mut to_deliver = Vec::new();
        self.message_queue.retain(|(msg, deliver_at)| {
            if *deliver_at <= now {
                to_deliver.push(msg.clone());
                false
            } else {
                true
            }
        });

        for msg in to_deliver {
            if let Some(node_lock) = self.nodes.get(&msg.to) {
                let mut node = node_lock.lock().unwrap();
                let responses = node.handle_message(msg).await;
                drop(node);
                for resp in responses {
                    self.enqueue_message(resp);
                }
            }
        }
    }

    pub fn get_cluster_diff(&self) -> serde_json::Value {
        let mut nodes_info = Vec::new();
        for node_lock in self.nodes.values() {
            let node = node_lock.lock().unwrap();
            nodes_info.push(serde_json::json!({
                "id": node.id,
                "role": match node.role {
                    NodeRole::LEADER => "LEADER",
                    NodeRole::FOLLOWER => "FOLLOWER",
                    NodeRole::CANDIDATE => "CANDIDATE",
                },
                "term": node.current_term,
                "isAlive": node.is_alive,
                "log": node.log.get_entries(),
                "commitIndex": node.commit_index,
                "lastApplied": node.last_applied,
                "stateMachine": node.state_machine
            }));
        }

        let in_flight: Vec<MessageInFlight> = self.message_queue.iter().map(|(msg, _)| MessageInFlight {
            from: msg.from.clone(),
            to: msg.to.clone(),
            msg_type: msg.msg_type.clone(),
        }).collect();

        serde_json::json!({
            "type": "DIFF",
            "data": {
                "nodes": nodes_info,
                "inFlight": in_flight
            }
        })
    }
}
