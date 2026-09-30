import { useState, useEffect } from 'react';

export interface GatewayStateResponse {
  daemon_status: string;
  consensus_score: number;
  timestamp: string;
}

export function useHilbertLedger(endpoint: string = 'http://127.0.0.1:8081/api/v1/ledger/status', pollIntervalMs: number = 500) {
  const [data, setData] = useState<GatewayStateResponse | null>(null);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    const fetchStatus = async () => {
      try {
        const response = await fetch(endpoint);
        if (!response.ok) {
          throw new Error(`Gateway returned HTTP ${response.status}`);
        }
        const json: GatewayStateResponse = await response.json();
        
        if (isMounted) {
          setData(json);
          setIsConnected(json.daemon_status === 'OK');
          setError(null);
        }
      } catch (err: any) {
        if (isMounted) {
          setIsConnected(false);
          setError(err.message || 'Failed to connect to REST Gateway');
        }
      }
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, pollIntervalMs);

    return () => {
      isMounted = false;
      clearInterval(interval);
    };
  }, [endpoint, pollIntervalMs]);

  return { data, isConnected, error };
}