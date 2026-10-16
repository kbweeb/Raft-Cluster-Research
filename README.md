# Raft Distributed System

This project consists of a Raft consensus engine written in Rust.

## Deployment Guide

### 1. Backend Service (Render - Docker)

1. Create a new **Web Service** on Render.
2. Connect your repository.
3. Set **Root Directory** to `backend-rust`.
4. Set **Environment** to `Docker`. (Render will use `backend-rust/Dockerfile`).
5. Set **Environment Variables**:
   - `PORT`: `8080`
6. Render will provide a URL like `https://your-rust-backend.onrender.com`.

### 2. Backend Service (Vercel - Serverless Functions)

To deploy the Rust backend to Vercel for free:

1. **Install Vercel CLI:** `npm i -g vercel`
2. **Configure `vercel.json`:** Create a `vercel.json` in the root (or `backend-rust`) to use the `@vercel/rust` runtime.
3. **Restructure (Optional):** Vercel expects Rust code in an `api/` directory for serverless execution.
4. **Deploy:** Run `vercel` in your terminal.
5. **Free Tier Note:** Vercel's free tier is excellent for serverless functions. However, since Raft is a stateful consensus protocol, keep in mind that serverless functions are ephemeral (they shut down when not in use). For a persistent Raft cluster, Render (from Step 1) is recommended, but Vercel is great for the API layer.

## Rust Backend

The Rust implementation provides a Raft consensus engine with high performance and memory safety.

### Building Rust Backend
1. Ensure you have Rust and a C linker (MSVC or Mingw-w64) installed.
2. Navigate to `backend-rust/`.
3. Build with: `cargo build --release`
4. Run with: `cargo run` (Defaults to port 8080).

## Verification Walkthrough

1. **Elections:** Upon startup, nodes will start elections. Observe the Gold (Leader) and Blue (Followers) badges.
2. **Replication:** Use the "Client Console" to `SET x=10`. Watch as the entry appears in the leader's log and replicates to followers. Notice the transition from dashed (uncommitted) to solid green glow (committed).
3. **Failover:** Click on the Leader node to "Kill" it. Observe as a follower times out and becomes a Candidate, eventually being elected as the new Leader.
4. **Consistency:** Revive the killed node. Watch it catch up by replaying log entries from the new leader and eventually reaching consistency.
5. **Chaos:** Adjust latency and packet drop sliders to see how the cluster handles degraded network conditions.
