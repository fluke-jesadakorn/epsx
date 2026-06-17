'use client';

import { useState, useCallback } from 'react';
import type { EndpointDef } from '../data/endpoints';

interface TryItResult {
  status: number;
  body: unknown;
  duration: number;
}

export function useTryIt() {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<TryItResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(async (endpoint: EndpointDef, apiKey: string, params: Record<string, string>) => {
    setLoading(true);
    setResult(null);
    setError(null);

    // Build URL with query params for GET/DELETE
    let url = endpoint.path;
    const headers: Record<string, string> = {};
    if (apiKey) {
      headers['Authorization'] = `Bearer ${apiKey}`;
    }

    let body: string | undefined;

    if (endpoint.method === 'GET' || endpoint.method === 'DELETE') {
      const qs = new URLSearchParams();
      for (const [k, v] of Object.entries(params)) {
        if (v) {
          qs.set(k, v);
        }
      }
      const qStr = qs.toString();
      if (qStr) {
        url += `?${qStr}`;
      }
    } else {
      headers['Content-Type'] = 'application/json';
      const filtered: Record<string, string> = {};
      for (const [k, v] of Object.entries(params)) {
        if (v) {
          filtered[k] = v;
        }
      }
      body = JSON.stringify(filtered);
    }

    const start = performance.now();
    try {
      const res = await fetch(url, { method: endpoint.method, headers, body });
      const duration = Math.round(performance.now() - start);
      let resBody: unknown;
      try {
        resBody = await res.json();
      } catch {
        resBody = await res.text();
      }
      setResult({ status: res.status, body: resBody, duration });
    } catch (e) {
      const duration = Math.round(performance.now() - start);
      setError(e instanceof Error ? e.message : 'Request failed');
      setResult({ status: 0, body: null, duration });
    } finally {
      setLoading(false);
    }
  }, []);

  const reset = useCallback(() => {
    setResult(null);
    setError(null);
  }, []);

  return { loading, result, error, execute, reset };
}
