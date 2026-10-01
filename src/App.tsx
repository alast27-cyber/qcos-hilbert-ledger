import React from 'react';

export default function App() {
  return (
    <main className="min-h-screen flex flex-col items-center justify-center p-6 bg-slate-950 text-white">
      <div className="max-w-xl w-full bg-slate-900 border border-slate-800 rounded-xl p-8 shadow-2xl">
        <h1 className="text-2xl font-bold tracking-tight mb-2 text-cyan-400">QCOS Hilbert Ledger</h1>
        <p className="text-slate-400 text-sm mb-6">Telemetry & State Injection Dashboard</p>
        <div className="p-4 bg-slate-950 rounded-lg border border-slate-800 font-mono text-xs text-green-400">
          [SYSTEM] Dashboard online and synced with production Vercel pipeline.
        </div>
      </div>
    </main>
  );
}
