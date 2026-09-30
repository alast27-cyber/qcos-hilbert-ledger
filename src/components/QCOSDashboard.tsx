import React, { useState } from 'react';
import { useLedgerStream } from './useLedgerStream';

export function QCOSDashboard() {
  const { metrics, isConnected, error, injectState } = useLedgerStream();
  const [loading, setLoading] = useState(false);

  const handleManualInjection = async () => {
    setLoading(true);
    try {
      await injectState({
        client_id: "qcos_dashboard_client",
        mode_index: 2,
        amplitude_re: 0.7071,
        amplitude_im: 0.7071,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6 bg-slate-950 text-white min-h-screen">
      <header className="flex justify-between items-center mb-6 border-b border-slate-800 pb-4">
        <h1 className="text-xl font-bold tracking-wider">QCOS Hilbert Ledger Telemetry</h1>
        <div className="flex items-center gap-2">
          <span className={`w-3 h-3 rounded-full ${isConnected ? 'bg-emerald-500 animate-pulse' : 'bg-rose-500'}`} />
          <span className="text-sm font-mono uppercase">{metrics?.status || 'DISCONNECTED'}</span>
        </div>
      </header>

      {error && <div className="mb-4 p-3 bg-rose-950/50 border border-rose-800 text-rose-300 rounded">{error}</div>}

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <div className="p-4 bg-slate-900 border border-slate-800 rounded-lg">
          <div className="text-slate-400 text-xs uppercase font-mono">Total Injections</div>
          <div className="text-2xl font-bold mt-1">{metrics?.total_injections ?? 0}</div>
        </div>
        <div className="p-4 bg-slate-900 border border-slate-800 rounded-lg">
          <div className="text-slate-400 text-xs uppercase font-mono">Branches Pruned</div>
          <div className="text-2xl font-bold mt-1">{metrics?.total_branches_pruned ?? 0}</div>
        </div>
        <div className="p-4 bg-slate-900 border border-slate-800 rounded-lg">
          <div className="text-slate-400 text-xs uppercase font-mono">Active Nodes</div>
          <div className="text-2xl font-bold mt-1">{metrics?.active_nodes_count ?? 0}</div>
        </div>
        <div className="p-4 bg-slate-900 border border-slate-800 rounded-lg">
          <div className="text-slate-400 text-xs uppercase font-mono">Average Prune Rate</div>
          <div className="text-2xl font-bold mt-1">{(metrics?.average_prune_rate ?? 0).toFixed(2)} /s</div>
        </div>
      </div>

      <div className="flex gap-4">
        <button
          onClick={handleManualInjection}
          disabled={loading}
          className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-medium rounded transition"
        >
          {loading ? 'Injecting State...' : 'Trigger State Injection'}
        </button>
      </div>
    </div>
  );
}