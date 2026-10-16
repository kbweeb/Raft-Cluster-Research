"use client";

import React, { useEffect, useState, useCallback, useRef } from "react";
import { 
  Activity, 
  Cpu, 
  Database, 
  Zap, 
  RefreshCw, 
  Skull, 
  Heart, 
  Network,
  Settings,
  Terminal,
  ChevronRight,
  ShieldCheck,
  AlertCircle
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

// --- Utilities ---
function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// --- Types ---
interface LogEntry {
  term: number;
  command: {
    op: string;
    key: string;
    value: any;
  };
}

interface NodeData {
  id: string;
  role: "LEADER" | "FOLLOWER" | "CANDIDATE";
  term: number;
  isAlive: boolean;
  log: LogEntry[];
  commitIndex: number;
  lastApplied: number;
  stateMachine: Record<string, any>;
}

interface MessageInFlight {
  from: string;
  to: string;
  type: string;
}

interface ClusterData {
  nodes: NodeData[];
  inFlight: MessageInFlight[];
}

// --- Constants ---
const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:8080";
const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8080/ws/telemetry";

// --- Components ---

const NodeCard = ({ 
  node, 
  onKill, 
  onRevive 
}: { 
  node: NodeData; 
  onKill: (id: string) => void;
  onRevive: (id: string) => void;
}) => {
  const isLeader = node.role === "LEADER";
  const isCandidate = node.role === "CANDIDATE";
  
  return (
    <motion.div
      layout
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      className={cn(
        "relative flex flex-col gap-4 rounded-2xl border p-5 shadow-sm transition-all duration-300 bg-white dark:bg-zinc-900",
        node.isAlive 
          ? isLeader 
            ? "border-amber-500 shadow-amber-500/10 dark:border-amber-500/30" 
            : isCandidate 
              ? "border-purple-500 shadow-purple-500/10 dark:border-purple-500/30"
              : "border-zinc-200 dark:border-zinc-800"
          : "border-red-200 dark:border-red-900/30 opacity-60 grayscale"
      )}
    >
      {/* Node Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={cn(
            "flex h-10 w-10 items-center justify-center rounded-xl",
            isLeader ? "bg-amber-100 text-amber-600" : 
            isCandidate ? "bg-purple-100 text-purple-600" : "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400"
          )}>
            <Cpu size={20} />
          </div>
          <div>
            <h3 className="text-sm font-bold text-zinc-900 dark:text-zinc-100">{node.id}</h3>
            <div className="flex items-center gap-1.5">
              <span className={cn(
                "h-2 w-2 rounded-full",
                node.isAlive ? "bg-emerald-500 animate-pulse" : "bg-red-500"
              )} />
              <span className="text-[10px] font-medium uppercase tracking-wider text-zinc-500">
                {node.role}
              </span>
            </div>
          </div>
        </div>
        
        <button
          onClick={() => node.isAlive ? onKill(node.id) : onRevive(node.id)}
          className={cn(
            "flex h-8 w-8 items-center justify-center rounded-lg border transition-colors",
            node.isAlive 
              ? "border-red-100 bg-red-50 text-red-600 hover:bg-red-100 dark:border-red-900/30 dark:bg-red-900/20 dark:text-red-400" 
              : "border-emerald-100 bg-emerald-50 text-emerald-600 hover:bg-emerald-100 dark:border-emerald-900/30 dark:bg-emerald-900/20 dark:text-emerald-400"
          )}
        >
          {node.isAlive ? <Skull size={14} /> : <Heart size={14} />}
        </button>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 gap-2">
        <div className="rounded-lg bg-zinc-50 p-2 dark:bg-zinc-800/50">
          <div className="text-[10px] text-zinc-500">Term</div>
          <div className="font-mono text-sm font-semibold">{node.term}</div>
        </div>
        <div className="rounded-lg bg-zinc-50 p-2 dark:bg-zinc-800/50">
          <div className="text-[10px] text-zinc-500">Commit Idx</div>
          <div className="font-mono text-sm font-semibold">{node.commitIndex}</div>
        </div>
      </div>

      {/* Log Preview */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between px-1">
          <span className="text-[10px] font-semibold uppercase text-zinc-400">Log entries ({node.log.length})</span>
          {node.log.length > 0 && (
             <span className="text-[10px] font-mono text-zinc-500">Tail: {node.log.length - 1}</span>
          )}
        </div>
        <div className="flex gap-1 overflow-x-auto pb-1 no-scrollbar">
          {node.log.slice(-8).map((entry, idx) => (
            <div 
              key={idx}
              className={cn(
                "h-6 w-6 shrink-0 flex items-center justify-center rounded border text-[10px] font-mono",
                (node.log.length - 8 + idx) < node.commitIndex
                  ? "bg-emerald-50 border-emerald-200 text-emerald-700 dark:bg-emerald-900/20 dark:border-emerald-800 dark:text-emerald-400"
                  : "bg-zinc-50 border-zinc-200 text-zinc-600 dark:bg-zinc-800 dark:border-zinc-700 dark:text-zinc-500"
              )}
              title={`${entry.command.op} ${entry.command.key}`}
            >
              {entry.term}
            </div>
          ))}
          {node.log.length === 0 && (
            <div className="h-6 w-full flex items-center justify-center rounded border border-dashed border-zinc-200 dark:border-zinc-800 text-[10px] text-zinc-400">
              EMPTY
            </div>
          )}
        </div>
      </div>
      
      {/* State Machine */}
      <div className="space-y-1.5">
        <span className="px-1 text-[10px] font-semibold uppercase text-zinc-400">State Machine</span>
        <div className="rounded-lg border border-zinc-100 bg-zinc-50/50 p-2 dark:border-zinc-800 dark:bg-zinc-800/20">
          {Object.entries(node.stateMachine).length > 0 ? (
            <div className="grid grid-cols-2 gap-1 text-[11px]">
              {Object.entries(node.stateMachine).slice(0, 4).map(([k, v]) => (
                <div key={k} className="flex justify-between">
                  <span className="text-zinc-500">{k}:</span>
                  <span className="font-mono font-medium text-zinc-700 dark:text-zinc-300">{JSON.stringify(v)}</span>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center text-[10px] italic text-zinc-400">No data</div>
          )}
        </div>
      </div>
    </motion.div>
  );
};

export default function RaftDashboard() {
  const [cluster, setCluster] = useState<ClusterData>({ nodes: [], inFlight: [] });
  const [logs, setLogs] = useState<string[]>([]);
  const [connected, setConnected] = useState(false);
  const [latency, setLatency] = useState(0);
  const [dropRate, setDropRate] = useState(0);
  const [proposeKey, setProposeKey] = useState("x");
  const [proposeValue, setProposeValue] = useState("1");
  const ws = useRef<WebSocket | null>(null);

  const addLog = useCallback((msg: string) => {
    setLogs(prev => [msg, ...prev].slice(0, 50));
  }, []);

  const connect = useCallback(() => {
    try {
      ws.current = new WebSocket(WS_URL);
      ws.current.onopen = () => {
        setConnected(true);
        addLog("Connected to Raft Cluster backend");
      };
      ws.current.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        if (msg.type === "DIFF") {
          setCluster(msg.data);
        }
      };
      ws.current.onclose = () => {
        setConnected(false);
        addLog("Disconnected. Attempting reconnect...");
        setTimeout(connect, 2000);
      };
      ws.current.onerror = () => {
        setConnected(false);
      };
    } catch (e) {
      console.error(e);
    }
  }, [addLog]);

  useEffect(() => {
    connect();
    return () => ws.current?.close();
  }, [connect]);

  const handleAction = async (path: string, method: string = "POST", body: any = null) => {
    try {
      const res = await fetch(`${BACKEND_URL}${path}`, {
        method,
        headers: { "Content-Type": "application/json" },
        body: body ? JSON.stringify(body) : null
      });
      const data = await res.json();
      if (!data.success) {
        addLog(`Error: ${data.error || "Action failed"}`);
      }
    } catch (e) {
      addLog(`Network error calling ${path}`);
    }
  };

  const propose = () => {
    handleAction("/api/propose", "POST", { 
      op: "SET", 
      key: proposeKey, 
      value: parseInt(proposeValue) || proposeValue 
    });
    addLog(`Sent proposal: SET ${proposeKey}=${proposeValue}`);
  };

  const updateNetwork = (lat: number, drop: number) => {
    handleAction("/api/network-conditions", "POST", { latency: lat, dropRate: drop });
  };

  return (
    <div className="min-h-screen bg-zinc-50 text-zinc-900 dark:bg-black dark:text-zinc-100 selection:bg-amber-100 dark:selection:bg-amber-900/30">
      {/* Header */}
      <header className="sticky top-0 z-50 border-b border-zinc-200 bg-white/80 backdrop-blur-md dark:border-zinc-800 dark:bg-black/80">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-3 sm:px-6">
          <div className="flex items-center gap-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-amber-500 text-white shadow-lg shadow-amber-500/20">
              <Network size={22} />
            </div>
            <div>
              <h1 className="text-lg font-bold tracking-tight">Raft Cluster Research</h1>
              <div className="flex items-center gap-2">
                <span className={cn(
                  "h-1.5 w-1.5 rounded-full",
                  connected ? "bg-emerald-500" : "bg-red-500"
                )} />
                <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-500">
                  {connected ? "Telemetric Stream Active" : "Backend Offline"}
                </span>
              </div>
            </div>
          </div>
          
          <div className="flex items-center gap-3">
            <div className="hidden items-center gap-6 rounded-full border border-zinc-100 bg-zinc-50/50 px-5 py-2 dark:border-zinc-800 dark:bg-zinc-900/50 sm:flex">
              <div className="flex flex-col items-center">
                <span className="text-[9px] font-bold uppercase text-zinc-400">Nodes</span>
                <span className="text-sm font-mono font-bold">{cluster.nodes.length}</span>
              </div>
              <div className="h-6 w-[1px] bg-zinc-200 dark:bg-zinc-800" />
              <div className="flex flex-col items-center">
                <span className="text-[9px] font-bold uppercase text-zinc-400">Term</span>
                <span className="text-sm font-mono font-bold">
                  {Math.max(0, ...cluster.nodes.map(n => n.term))}
                </span>
              </div>
              <div className="h-6 w-[1px] bg-zinc-200 dark:bg-zinc-800" />
              <div className="flex flex-col items-center">
                <span className="text-[9px] font-bold uppercase text-zinc-400">In-Flight</span>
                <span className="text-sm font-mono font-bold">{cluster.inFlight.length}</span>
              </div>
            </div>
            
            <button className="flex h-10 w-10 items-center justify-center rounded-full hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors">
              <Settings size={20} className="text-zinc-500" />
            </button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-7xl p-4 sm:p-6 lg:p-8">
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-12">
          
          {/* Left Column: Visualization & Controls */}
          <div className="lg:col-span-8 space-y-8">
            
            {/* Cluster View */}
            <section className="space-y-4">
              <div className="flex items-center justify-between px-2">
                <h2 className="flex items-center gap-2 text-sm font-bold uppercase tracking-wider text-zinc-500">
                  <Activity size={16} /> Cluster Topology
                </h2>
                <span className="text-[11px] text-zinc-400">Real-time node state synchronization</span>
              </div>
              
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
                <AnimatePresence mode="popLayout">
                  {cluster.nodes.map((node) => (
                    <NodeCard 
                      key={node.id} 
                      node={node} 
                      onKill={(id) => handleAction(`/api/nodes/${id}/kill`)}
                      onRevive={(id) => handleAction(`/api/nodes/${id}/revive`)}
                    />
                  ))}
                </AnimatePresence>
              </div>
            </section>

            {/* Network Visualization (Abstract representation) */}
            <section className="rounded-2xl border border-zinc-200 bg-white p-6 dark:border-zinc-800 dark:bg-zinc-900/50">
              <div className="mb-6 flex items-center justify-between">
                <h2 className="flex items-center gap-2 text-sm font-bold uppercase tracking-wider text-zinc-500">
                  <Network size={16} /> Network Activity
                </h2>
                <div className="flex gap-2">
                  {cluster.inFlight.slice(-5).map((m, i) => (
                    <motion.div
                      key={i}
                      initial={{ scale: 0 }}
                      animate={{ scale: 1 }}
                      className="px-2 py-0.5 rounded-full bg-amber-50 text-amber-600 border border-amber-100 text-[9px] font-bold"
                    >
                      {m.type}
                    </motion.div>
                  ))}
                </div>
              </div>
              
              <div className="relative h-48 w-full rounded-xl bg-zinc-50 dark:bg-black/50 overflow-hidden flex items-center justify-center border border-zinc-100 dark:border-zinc-800">
                {cluster.inFlight.length > 0 ? (
                  <div className="flex flex-col items-center gap-3">
                    <div className="flex gap-2">
                       {Array.from({ length: Math.min(cluster.inFlight.length, 12) }).map((_, i) => (
                         <motion.div
                            key={i}
                            animate={{ 
                              y: [0, -10, 0],
                              opacity: [0.5, 1, 0.5]
                            }}
                            transition={{ 
                              duration: 1, 
                              repeat: Infinity, 
                              delay: i * 0.1 
                            }}
                            className="w-2 h-8 bg-amber-400 rounded-full"
                         />
                       ))}
                    </div>
                    <span className="text-xs font-medium text-zinc-500 animate-pulse">
                      {cluster.inFlight.length} Messages in transit...
                    </span>
                  </div>
                ) : (
                  <div className="flex flex-col items-center gap-2 text-zinc-400">
                    <Zap size={24} className="opacity-20" />
                    <span className="text-xs italic">Network Idle</span>
                  </div>
                )}
                
                {/* Decorative Grid */}
                <div className="absolute inset-0 opacity-[0.03] dark:opacity-[0.05] pointer-events-none" 
                     style={{ backgroundImage: 'radial-gradient(circle, #000 1px, transparent 1px)', backgroundSize: '20px 20px' }} />
              </div>
            </section>
          </div>

          {/* Right Column: Console & Controls */}
          <div className="lg:col-span-4 space-y-8">
            
            {/* Command Panel */}
            <section className="rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
              <h2 className="mb-4 flex items-center gap-2 text-sm font-bold uppercase tracking-wider text-zinc-500">
                <Database size={16} /> Client Console
              </h2>
              
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-[10px] font-bold uppercase text-zinc-400">Propose State Change</label>
                  <div className="flex gap-2">
                    <input 
                      value={proposeKey}
                      onChange={e => setProposeKey(e.target.value)}
                      placeholder="Key"
                      className="w-20 rounded-lg border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm font-mono dark:border-zinc-800 dark:bg-zinc-800"
                    />
                    <input 
                      value={proposeValue}
                      onChange={e => setProposeValue(e.target.value)}
                      placeholder="Value"
                      className="flex-1 rounded-lg border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm font-mono dark:border-zinc-800 dark:bg-zinc-800"
                    />
                  </div>
                  <button 
                    onClick={propose}
                    disabled={!connected}
                    className="flex w-full items-center justify-center gap-2 rounded-lg bg-zinc-900 px-4 py-2 text-sm font-bold text-white transition-all hover:bg-zinc-800 active:scale-[0.98] disabled:opacity-50 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200"
                  >
                    <ChevronRight size={16} /> Propose SET
                  </button>
                </div>

                <div className="h-[1px] bg-zinc-100 dark:bg-zinc-800" />

                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <label className="text-[10px] font-bold uppercase text-zinc-400">Chaos Latency</label>
                    <span className="text-[10px] font-mono font-bold text-amber-600">{latency}ms</span>
                  </div>
                  <input 
                    type="range" min="0" max="500" step="10"
                    value={latency}
                    onChange={e => {
                      const val = parseInt(e.target.value);
                      setLatency(val);
                      updateNetwork(val, dropRate);
                    }}
                    className="w-full accent-amber-500"
                  />
                  
                  <div className="flex items-center justify-between pt-2">
                    <label className="text-[10px] font-bold uppercase text-zinc-400">Packet Drop Rate</label>
                    <span className="text-[10px] font-mono font-bold text-red-600">{(dropRate * 100).toFixed(0)}%</span>
                  </div>
                  <input 
                    type="range" min="0" max="1" step="0.05"
                    value={dropRate}
                    onChange={e => {
                      const val = parseFloat(e.target.value);
                      setDropRate(val);
                      updateNetwork(latency, val);
                    }}
                    className="w-full accent-red-500"
                  />
                </div>
              </div>
            </section>

            {/* System Logs */}
            <section className="rounded-2xl border border-zinc-200 bg-zinc-900 p-4 shadow-xl dark:border-zinc-800">
              <div className="mb-3 flex items-center justify-between border-b border-zinc-800 pb-2">
                <h2 className="flex items-center gap-2 text-[10px] font-bold uppercase tracking-widest text-zinc-400">
                  <Terminal size={14} /> System Event Log
                </h2>
                <div className="flex gap-1.5">
                  <span className="h-2 w-2 rounded-full bg-red-500/20" />
                  <span className="h-2 w-2 rounded-full bg-amber-500/20" />
                  <span className="h-2 w-2 rounded-full bg-emerald-500/20" />
                </div>
              </div>
              <div className="h-[300px] overflow-y-auto font-mono text-[11px] leading-relaxed text-zinc-300 no-scrollbar space-y-1">
                {logs.map((log, i) => (
                  <div key={i} className="flex gap-2">
                    <span className="text-zinc-600">[{new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })}]</span>
                    <span className={cn(
                      log.includes("Error") ? "text-red-400" : 
                      log.includes("Connected") ? "text-emerald-400" : "text-zinc-300"
                    )}>{log}</span>
                  </div>
                ))}
                {logs.length === 0 && <div className="text-zinc-600 italic">Awaiting telemetry...</div>}
              </div>
            </section>
          </div>
        </div>
      </main>

      {/* Footer Branding */}
      <footer className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
        <div className="flex flex-col items-center justify-between gap-4 border-t border-zinc-200 pt-8 dark:border-zinc-800 sm:flex-row">
          <div className="flex items-center gap-2 text-zinc-500">
            <ShieldCheck size={18} />
            <span className="text-xs font-medium tracking-tight">Raft Consensus Protocol Explorer &copy; 2026</span>
          </div>
          <div className="flex gap-4">
            <a href="#" className="text-xs font-semibold text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-100 transition-colors">Documentation</a>
            <a href="#" className="text-xs font-semibold text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-100 transition-colors">GitHub Repo</a>
          </div>
        </div>
      </footer>
    </div>
  );
}
