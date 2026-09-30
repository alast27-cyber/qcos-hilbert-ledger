import { useState, useEffect, useCallback } from 'react';

export interface LedgerMetrics {
  status: string;
  total_injections: number;
  total_branches_pruned: number;
  active_nodes_count: number;
  average_prune_rate: number;
  timestamp: string;
}

const WS_URL = 'ws://127.0.0.1:8081/api/v1/ledger/ws';

export function useLedgerStream() {
  const [metrics, setMetrics] = useState<LedgerMetrics | null>(null);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let ws: WebSocket | null = null;
    let reconnectTimeout: NodeJS.Timeout;

    const connect = () => {
      ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        setIsConnected(true);
        setError(null);
      };

      ws.onmessage = (event) => {
        try {
          const data: LedgerMetrics = JSON.parse(event.data);
          setMetrics(data);
        } catch (err) {
          console.error('Failed to parse ledger telemetry frame:', err);
        }
      };

      ws.onerror = (event) => {
        setError('WebSocket connection error');
        setIsConnected(false);
      };

      ws.onclose = () => {
        setIsConnected(false);
        // Attempt reconnection after 3 seconds
        reconnectTimeout = setTimeout(connect, 3000);
      };
    };

    connect();

    return () => {
      if (ws) ws.close();
      clearTimeout(reconnectTimeout);
    };
  }, []);

  const injectState = useCallback(async (payload: {
    client_id: string;
    mode_index: number;
    amplitude_re: number;
    amplitude_im: number;
  }) => {
    try {
      const response = await fetch('http://127.0.0.1:8081/api/v1/ledger/inject', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      return await response.json();
    } catch (err) {
      console.error('State injection failed:', err);
      throw err;
    }
  }, []);

  return { metrics, isConnected, error, injectState };
}