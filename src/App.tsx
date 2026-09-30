import React from 'react';
import { PruningMetricsWidget } from './components/PruningMetricsWidget';

export function App() {
  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 font-mono p-6">
      {/* Top Navigation / Header */}
      <header className="max-w-7xl mx-auto mb-8 pb-4 border-b border-slate-800 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-extrabold tracking-widest text-cyan-400">
            QCOS // QUANTUM OPERATING SYSTEM
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Core Engine: Hilbert Ledger v0.1.0 | Gateway: Active (Port 8081)
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-cyan-950 text-cyan-400 border border-cyan-800">
            AGENTQ SIMULATION MODE
          </span>
        </div>
      </header>

      {/* Main Dashboard Grid */}
      <main className="max-w-7xl mx-auto grid grid-cols-1 lg:grid-cols-3 gap-6">
        
        {/* Left Column: Pruning Metrics & Telemetry (Spans 2 columns on large screens) */}
        <section className="lg:col-span-2 space-y-6">
          <PruningMetricsWidget />

          {/* Secondary Operational Panel */}
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 shadow-xl">
            <h3 className="text-sm font-bold text-slate-300 mb-3 uppercase tracking-wider">
              Active Semantic Probability Tree
            </h3>
            <div className="bg-slate-950 p-4 rounded-lg border border-slate-800/60 text-xs text-slate-400 font-mono space-y-2">
              <p className="text-cyan-400">root [mode: 0] (score: 1.00)</p>
              <p className="pl-4 text-slate-300">├── "Quantum" [mode: 0] (prob: 0.998) - ACTIVE</p>
              <p className="pl-8 text-slate-500">└── [Pruned: Threshold P &lt; 0.05]</p>
              <p className="pl-4 text-rose-400">└── "Classical" [mode: 1] (prob: 0.010) - PRUNED</p>
            </div>
          </div>
        </section>

        {/* Right Column: Node Activity & Quick Actions */}
        <section className="space-y-6">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 shadow-xl">
            <h3 className="text-sm font-bold text-slate-300 mb-4 uppercase tracking-wider">
              Active Agent Nodes
            </h3>
            <div className="space-y-3 text-xs">
              <div className="flex items-center justify-between bg-slate-950 p-3 rounded border border-slate-800">
                <span className="text-slate-300">agent_q_node_01</span>
                <span className="text-emerald-400 flex items-center">
                  <span className="w-2 h-2 rounded-full bg-emerald-500 mr-2 animate-pulse" />
                  Streaming
                </span>
              </div>
              <div className="flex items-center justify-between bg-slate-950 p-3 rounded border border-slate-800">
                <span className="text-slate-300">benchmark_agent_2</span>
                <span className="text-amber-400 flex items-center">
                  <span className="w-2 h-2 rounded-full bg-amber-500 mr-2" />
                  Idle
                </span>
              </div>
            </div>
          </div>

          <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 shadow-xl">
            <h3 className="text-sm font-bold text-slate-300 mb-3 uppercase tracking-wider">
              Gateway Controls
            </h3>
            <p className="text-xs text-slate-400 mb-4">
              Trigger manual EAP state injection or flush the semantic tree cache.
            </p>
            <button 
              onClick={() => alert('Manual injection trigger ready.')}
              className="w-full bg-cyan-600 hover:bg-cyan-500 text-slate-950 font-bold py-2 px-4 rounded text-xs transition-colors uppercase tracking-wider"
            >
              Inject Test EAP Payload
            </button>
          </div>
        </section>

      </main>
    </div>
  );
}

export default App;