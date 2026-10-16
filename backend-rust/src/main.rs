mod wal;
mod raft_node;
mod cluster;

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use cluster::Cluster;
use raft_node::{NodeRole, Command};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use futures_util::{StreamExt, SinkExt};
use sqlx::postgres::PgPoolOptions;

struct AppState {
    cluster: Arc<Mutex<Cluster>>,
    tx: broadcast::Sender<serde_json::Value>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let (tx, _) = broadcast::channel(100);
    
    // Setup database
    let database_url = std::env::var("DATABASE_URL").ok();
    let pool = if let Some(url) = database_url {
        let p = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .expect("Failed to connect to Postgres");
            
        // Initialize tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS raft_logs (
                index BIGINT PRIMARY KEY,
                term BIGINT NOT NULL,
                command JSONB NOT NULL,
                timestamp BIGINT NOT NULL,
                checksum TEXT NOT NULL
            )"
        )
        .execute(&p)
        .await
        .expect("Failed to create raft_logs table");
        
        Some(p)
    } else {
        println!("DATABASE_URL not set, running in-memory only");
        None
    };

    let node_ids = vec![
        "Node-1".to_string(),
        "Node-2".to_string(),
        "Node-3".to_string(),
        "Node-4".to_string(),
        "Node-5".to_string(),
    ];
    let cluster = Cluster::new(node_ids, tx.clone(), pool);
    
    // Load state from DB
    {
        let c = cluster.lock().unwrap();
        for node_lock in c.nodes.values() {
            let mut node = node_lock.lock().unwrap();
            let _ = node.log.load_from_db().await;
            // Apply logs to state machine
            node.apply_to_state_machine();
        }
    }

    let state = Arc::new(AppState {
        cluster: cluster.clone(),
        tx: tx.clone(),
    });

    // Background tasks
    let cluster_clone = cluster.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
        loop {
            interval.tick().await;
            let mut c = cluster_clone.lock().unwrap();
            c.process_messages().await;
        }
    });

    let cluster_clone = cluster.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            let diff = {
                let c = cluster_clone.lock().unwrap();
                c.get_cluster_diff()
            };
            let _ = tx_clone.send(diff);
        }
    });

    // Raft Logic Timers (Elections and Heartbeats)
    let cluster_clone = cluster.clone();
    tokio::spawn(async move {
        // Election timers
        let mut election_timers: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut rng = rand::thread_rng();
        
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        loop {
            interval.tick().await;
            let now = chrono::Utc::now().timestamp_millis() as u64;
            
            let (to_replicate, to_elect) = {
                let mut c = cluster_clone.lock().unwrap();
                let mut replicate_msgs = Vec::new();
                let mut elect_msgs = Vec::new();

                for (id, node_lock) in &c.nodes {
                    let mut node = node_lock.lock().unwrap();
                    if !node.is_alive { continue; }
                    
                    if node.role == NodeRole::LEADER {
                        let heartbeats = node.send_heartbeats();
                        replicate_msgs.extend(heartbeats);
                    } else {
                        let timeout = election_timers.get(id).cloned().unwrap_or(0);
                        if now >= timeout {
                            let msgs = node.start_election();
                            elect_msgs.extend(msgs);
                            use rand::Rng;
                            let next_timeout = now + 150 + rand::thread_rng().gen_range(0..150);
                            election_timers.insert(id.clone(), next_timeout);
                        }
                    }
                }
                (replicate_msgs, elect_msgs)
            };

            let mut c = cluster_clone.lock().unwrap();
            for m in to_replicate { c.enqueue_message(m); }
            for m in to_elect { c.enqueue_message(m); }
        }
    });

    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/ws/telemetry", get(ws_handler))
        .route("/api/propose", post(propose))
        .route("/api/nodes/:id/kill", post(kill_node))
        .route("/api/nodes/:id/revive", post(revive_node))
        .route("/api/partition", post(set_partition))
        .route("/api/network-conditions", post(set_network_conditions))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Backend Raft Engine (Rust) listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        let payload = serde_json::to_string(&msg).unwrap();
        if socket.send(Message::Text(payload)).await.is_err() {
            break;
        }
    }
}

#[derive(Deserialize)]
struct ProposeRequest {
    op: String,
    key: String,
    value: Option<serde_json::Value>,
}

async fn propose(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProposeRequest>,
) -> Json<serde_json::Value> {
    let leader_id = {
        let c = state.cluster.lock().unwrap();
        c.nodes.iter().find(|(_, n)| {
            let node = n.lock().unwrap();
            node.role == NodeRole::LEADER && node.is_alive
        }).map(|(id, _)| id.clone())
    };

    if let Some(id) = leader_id {
        let mut c = state.cluster.lock().unwrap();
        let node_lock = c.nodes.get_mut(&id).unwrap();
        let mut node = node_lock.lock().unwrap();
        let success = node.propose(Command {
            op: payload.op,
            key: payload.key,
            value: payload.value,
        }).await;
        Json(serde_json::json!({ "success": success }))
    } else {
        Json(serde_json::json!({ "success": false, "error": "No active leader" }))
    }
}

async fn kill_node(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let c = state.cluster.lock().unwrap();
    if let Some(node_lock) = c.nodes.get(&id) {
        let mut node = node_lock.lock().unwrap();
        node.kill();
        Json(serde_json::json!({ "success": true }))
    } else {
        Json(serde_json::json!({ "success": false, "error": "Node not found" }))
    }
}

async fn revive_node(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let c = state.cluster.lock().unwrap();
    if let Some(node_lock) = c.nodes.get(&id) {
        let mut node = node_lock.lock().unwrap();
        node.revive();
        Json(serde_json::json!({ "success": true }))
    } else {
        Json(serde_json::json!({ "success": false, "error": "Node not found" }))
    }
}

#[derive(Deserialize)]
struct PartitionRequest {
    partitions: Vec<Vec<String>>,
}

async fn set_partition(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PartitionRequest>,
) -> Json<serde_json::Value> {
    let mut c = state.cluster.lock().unwrap();
    c.partitions = payload.partitions;
    Json(serde_json::json!({ "success": true }))
}

#[derive(Deserialize)]
struct NetworkConditionsRequest {
    latency: u64,
    #[serde(rename = "dropRate")]
    drop_rate: f64,
}

async fn set_network_conditions(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NetworkConditionsRequest>,
) -> Json<serde_json::Value> {
    let mut c = state.cluster.lock().unwrap();
    c.latency = payload.latency;
    c.drop_rate = payload.drop_rate;
    Json(serde_json::json!({ "success": true }))
}
