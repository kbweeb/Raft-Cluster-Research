# Raft Distributed System Visualization

This project consists of a Raft consensus engine and a real-time Next.js dashboard. The engine is available in two versions:
- **TypeScript (Original):** Located in `backend/`
- **Rust (High Performance):** Located in `backend-rust/`

## Deployment Guide

### 1. Backend Service (Render)

You can deploy either the TypeScript or the Rust version of the backend.

#### Option A: Rust Backend (Recommended)
1. Create a new **Web Service** on Render.
2. Connect your repository.
3. Set **Root Directory** to `backend-rust`.
4. Set **Environment** to `Docker`. (Render will use `backend-rust/Dockerfile`).
5. Set **Environment Variables**:
   - `PORT`: `8080`
6. Render will provide a URL like `https://your-rust-backend.onrender.com`.

#### Option B: TypeScript Backend
1. Create a new **Web Service** on Render.
2. Connect your repository.
3. Set **Root Directory** to `backend`.
4. Set **Environment** to `Docker`.
5. Render will provide a URL like `https://your-ts-backend.onrender.com`.

### 2. Frontend Dashboard (Vercel)

1. Create a new project on Vercel.
2. Connect your repository.
3. Set **Framework Preset**: `Next.js`.
4. Set **Root Directory**: `frontend`.
5. Set **Environment Variables**:
   - `NEXT_PUBLIC_BACKEND_URL`: `https://your-backend-url.onrender.com`
   - `NEXT_PUBLIC_WS_URL`: `wss://your-backend-url.onrender.com/ws/telemetry` (Use `wss://` for production).
6. Deploy.

## Rust Backend (Alternative)

The Rust implementation provides the same API and WebSocket telemetry as the TypeScript version but with better performance and memory safety.

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
