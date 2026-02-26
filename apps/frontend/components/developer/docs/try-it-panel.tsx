'use client';

import { useState, useCallback } from 'react';
import type { EndpointDef } from './data/endpoints';
import { useTryIt } from './hooks/use-try-it';
import { ResponseExample } from './response-example';

interface TryItPanelProps {
  endpoint: EndpointDef;
  apiKeys: { id: string; name: string; full_key?: string; key_prefix: string }[];
}

function StatusBadge({ status }: { status: number }) {
  const color = status >= 200 && status < 300
    ? 'bg-green-500/10 text-green-400'
    : status >= 400
      ? 'bg-red-500/10 text-red-400'
      : 'bg-yellow-500/10 text-yellow-400';
  return (
    <span className={`rounded-full px-2 py-0.5 text-xs font-mono font-semibold ${color}`}>
      {status}
    </span>
  );
}
 
export function TryItPanel({ endpoint, apiKeys }: TryItPanelProps) {
  const [selectedKey, setSelectedKey] = useState('');
  const [params, setParams] = useState<Record<string, string>>({});
  const { loading, result, error, execute, reset } = useTryIt();

  const updateParam = useCallback((name: string, value: string) => {
    setParams((prev) => ({ ...prev, [name]: value }));
  }, []);

  const handleSend = useCallback(() => {
    void execute(endpoint, selectedKey, params);
  }, [endpoint, selectedKey, params, execute]);

  return (
    <div className="rounded-2xl border border-border/20 bg-card shadow-xl">
      <div className="flex items-center justify-between border-b border-border/10 px-4 py-3">
        <h4 className="text-sm font-semibold text-foreground">Try it</h4>
        {result !== null && (
          <button type="button" onClick={reset} className="text-xs text-muted-foreground hover:text-foreground">
            Reset
          </button>
        )}
      </div>

      <div className="space-y-3 p-4">
        {/* API Key selector */}
        <div>
          <label className="mb-1 block text-xs font-medium text-muted-foreground">API Key</label>
          <select
            value={selectedKey}
            onChange={(e) => setSelectedKey(e.target.value)}
            className="w-full rounded-lg border border-border/30 bg-background px-3 py-2 text-sm text-foreground"
          >
            <option value="">No key (anonymous)</option>
            {apiKeys.map((k) => (
              <option key={k.id} value={k.full_key ?? k.key_prefix}>
                {k.name} ({k.key_prefix}...)
              </option>
            ))}
          </select>
        </div>

        {/* Params */}
        {endpoint.params?.map((p) => (
          <div key={p.name}>
            <label className="mb-1 flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
              {p.name}
              {p.required && <span className="text-red-400">*</span>}
              <span className="text-[10px] text-muted-foreground/60">{p.type}</span>
            </label>
            <input
              type="text"
              placeholder={p.default ?? p.desc}
              value={params[p.name] ?? ''}
              onChange={(e) => updateParam(p.name, e.target.value)}
              className="w-full rounded-lg border border-border/30 bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/40"
            />
          </div>
        ))}

        {/* Send */}
        <button
          type="button"
          onClick={handleSend}
          disabled={loading}
          className="w-full rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] px-4 py-2.5 text-sm font-semibold text-white transition-opacity hover:opacity-90 disabled:opacity-50"
        >
          {loading ? 'Sending...' : 'Send Request'}
        </button>

        {/* Error */}
        {error !== null && (
          <p className="text-xs text-red-400">{error}</p>
        )}

        {/* Result */}
        {result !== null && (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <StatusBadge status={result.status} />
              <span className="text-xs text-muted-foreground">{result.duration}ms</span>
            </div>
            {result.body !== null && (
              <ResponseExample data={result.body as Record<string, unknown>} />
            )}
          </div>
        )}
      </div>
    </div>
  );
}
