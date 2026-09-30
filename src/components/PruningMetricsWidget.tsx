import React, { useState, useEffect } from 'react';

interface MetricsData {
  status: string;
  total_injections: number;
  total_branches_pruned: number;
  active_nodes_count: number;
  average_prune_rate: number;
  timestamp: string;
}

export const PruningMetricsWidget: React.FC = () => {
  const [metrics, setMetrics] = useState<MetricsData | null>(null);
  const [isConnected, setIsConnected] = useState<boolean>(false);

  useEffect(() => {
    // Open WebSocket connection to Axum gateway
    const ws = new WebSocket('ws://127.0.0.1:8081/api/v1/ledger/ws');

    ws.onopen = () => setIsConnected(true);
    
    ws.onmessage = (event) => {
      try {
        const data: MetricsData = JSON.parse(event.data);
        setMetrics(data);
      } catch (err) {
        console.error('Failed to parse WebSocket telemetry:', err);
      }
    };

    ws.onclose = () => setIsConnected(false);
    ws.onerror = () => setIsConnected(false);

    // Cleanup socket on component unmount
    return () => {
      ws.close();
    };
  }, []);

  return (
    <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 text-slate-100 shadow-2xl font-mono">
      <div className="flex items-center justify-between mb-6 pb-4 border-b border-slate-800">
        <div>
          <h2 className="text-lg font-bold tracking-wider text-cyan-400">QLLM HILBERT LEDGER</h2>
          <p className="text-xs text-slate-400">Real-Time WebSocket Telemetry Stream</p>
        </div>
        <div className="flex items-center space-x-2">
          <span className={`w-3 h-3 rounded-full ${isConnected ? 'bg-emerald-500 animate-pulse' : 'bg-rose-500'}`} />
          <span className="text-xs uppercase tracking-widest text-slate-400">
            {isConnected ? 'STREAMING (WS)' : 'DISCONNECTED'}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="bg-slate-950 p-4 rounded-lg border border-slate-800/60">
          <span className="text-xs text-slate-400 block mb-1">Total Injections</span>
          <span className="text-2xl font-bold text-slate-200">
            {metrics?.total_injections.toLocaleString() ?? '---'}
          </span>
        </div>

        <div className="bg-slate-950 p-4 rounded-lg border border-slate-800/60">
          <span className="text-xs text-slate-400 block mb-1">Branches Pruned</span>
          <span className="text-2xl font-bold text-amber-400">
            {metrics?.total_branches_pruned.toLocaleString() ?? '---'}
          </span>
        </div>

        <div className="bg-slate-950 p-4 rounded-lg border border-slate-800/60">
          <span className="text-xs text-slate-400 block mb-1">Active Agent Nodes</span>
          <span className="text-2xl font-bold text-indigo-400">
            {metrics?.active_nodes_count ?? '---'}
          </span>
        </div>

        <div className="bg-slate-950 p-4 rounded-lg border border-slate-800/60">
          <span className="text-xs text-slate-400 block mb-1">Avg Prune Rate</span>
          <span className="text-2xl font-bold text-teal-400">
            {metrics ? `${metrics.average_prune_rate.toFixed(2)}x` : '---'}
          </span>
        </div>
      </div>

      <div className="mt-6 pt-3 border-t border-slate-800/60 text-[10px] text-slate-500 flex justify-between">
        <span>WS: ws://127.0.0.1:8081/api/v1/ledger/ws</span>
        <span>LAST SYNC: {metrics?.timestamp ? new Date(metrics.timestamp).toLocaleTimeString() : '---'}</span>
      </div>
    </div>
  );
};