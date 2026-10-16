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
   - `DATABASE_URL`: `postgres://user:pass@host/db` (Get this from your Render Postgres instance)
6. Render will provide a URL like `https://your-rust-backend.onrender.com`.

### 2. Database Service (Render - Postgres)

1. Create a new **PostgreSQL** database on Render.
2. Copy the **Internal Database URL** (or External if you want to test locally).
3. Paste it into the `DATABASE_URL` env var of your Web Service.
4. The backend will automatically create the `raft_logs` table on the first boot.

### 2. Frontend Dashboard (Vercel)

To deploy a Next.js frontend to Vercel for free:

1. **Restore/Add Frontend:** Ensure your frontend code (e.g., Next.js) is in a directory (e.g., `frontend/`). *Note: TypeScript components were previously removed to save space; ensure `node_modules` are gitignored.*
2. **Push to GitHub:** Commit and push your frontend directory.
3. **Connect to Vercel:**
   - Log in to [Vercel](https://vercel.com).
   - Click **"Add New"** > **"Project"**.
   - Import this GitHub repository.
4. **Configure Project:**
   - Set **Framework Preset** to `Next.js`.
   - Set **Root Directory** to `frontend`.
5. **Environment Variables:**
   - `NEXT_PUBLIC_BACKEND_URL`: `https://your-rust-backend.onrender.com`
   - `NEXT_PUBLIC_WS_URL`: `wss://your-rust-backend.onrender.com/ws/telemetry`
6. **Deploy:** Click **Deploy**. Vercel will automatically build and host your frontend on their global CDN.

### 3. Backend Service (Vercel - Serverless Functions)

To deploy the Rust backend to Vercel for free:

1. **Install Vercel CLI:** `npm i -g vercel`
2. **Configure `vercel.json`:** Create a `vercel.json` in the root (or `backend-rust`) to use the `@vercel/rust` runtime.
3. **Restructure (Optional):** Vercel expects Rust code in an `api/` directory for serverless execution.
4. **Deploy:** Run `vercel` in your terminal.
5. **Free Tier Note:** Vercel's free tier is excellent for serverless functions. However, since Raft is a stateful consensus protocol, keep in mind that serverless functions are ephemeral (they shut down when not in use). For a persistent Raft cluster, Render (from Step 1) is recommended, but Vercel is great for the API layer.

### 4. Troubleshooting & AI Agent Prompts

If your dashboard shows **"Backend Offline"**, you can use an AI agent (like Cursor, Claude Code, or Claude Desktop) with the **Render MCP Server** to fix it automatically.

**Copy and paste this prompt to your AI assistant:**

> "I have a Raft Cluster project. Use the Render MCP tool to:
> 1. Find my 'Raft Backend' web service and my 'Raft Database' Postgres instance.
> 2. Ensure the Web Service has the correct `DATABASE_URL` from the Postgres instance.
> 3. Verify the Web Service `PORT` is set to `8080`.
> 4. Check the latest build logs to ensure the Rust engine started successfully.
> 5. Give me the public URL of the backend so I can update my Vercel environment variables."

---

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
