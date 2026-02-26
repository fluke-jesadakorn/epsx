'use client';

import { useState, useCallback } from 'react';
import type { EndpointDef } from './data/endpoints';
import { TierBadge } from './tier-badge';
import { CodeSnippet } from './code-snippet';
import { ResponseExample } from './response-example';
import { TryItPanel } from './try-it-panel';

const methodColors: Record<string, string> = {
  GET: 'bg-blue-500/10 text-blue-500',
  POST: 'bg-green-500/10 text-green-500',
  DELETE: 'bg-red-500/10 text-red-500',
};

interface EndpointCardProps {
  endpoint: EndpointDef;
  apiKeys: { id: string; name: string; full_key?: string; key_prefix: string }[];
}
 
export function EndpointCard({ endpoint, apiKeys }: EndpointCardProps) {
  const [expanded, setExpanded] = useState(false);

  const toggle = useCallback(() => setExpanded((p) => !p), []);

  return (
    <div className="rounded-2xl border border-border/20 bg-card shadow-xl transition-shadow hover:shadow-2xl">
      {/* Header */}
      <button
        type="button"
        onClick={toggle}
        className="flex w-full items-center gap-3 px-5 py-4 text-left"
      >
        <span className={`rounded-lg px-2.5 py-1 text-xs font-bold ${methodColors[endpoint.method] ?? ''}`}>
          {endpoint.method}
        </span>
        <code className="flex-1 font-mono text-sm text-foreground">{endpoint.path}</code>
        <TierBadge tier={endpoint.tier} />
        <svg
          className={`h-4 w-4 text-muted-foreground transition-transform ${expanded ? 'rotate-180' : ''}`}
          fill="none" viewBox="0 0 24 24" stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Expanded content */}
      {expanded && (
        <div className="border-t border-border/10 px-5 py-4 space-y-5">
          <p className="text-sm text-muted-foreground">{endpoint.desc}</p>

          {/* Params table */}
          {endpoint.params && endpoint.params.length > 0 && (
            <div>
              <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">Parameters</h4>
              <div className="overflow-x-auto rounded-xl border border-border/10">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-border/10 text-left text-xs text-muted-foreground">
                      <th className="px-3 py-2 font-medium">Name</th>
                      <th className="px-3 py-2 font-medium">Type</th>
                      <th className="px-3 py-2 font-medium">Required</th>
                      <th className="px-3 py-2 font-medium">Description</th>
                    </tr>
                  </thead>
                  <tbody>
                    {endpoint.params.map((p) => (
                      <tr key={p.name} className="border-b border-border/5">
                        <td className="px-3 py-2 font-mono text-xs text-foreground">{p.name}</td>
                        <td className="px-3 py-2 text-xs text-muted-foreground">{p.type}</td>
                        <td className="px-3 py-2">
                          {p.required
                            ? <span className="text-xs text-red-400">yes</span>
                            : <span className="text-xs text-muted-foreground/50">no</span>
                          }
                        </td>
                        <td className="px-3 py-2 text-xs text-muted-foreground">
                          {p.desc}{p.default ? ` (default: ${p.default})` : ''}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Rate limits */}
          <div>
            <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">Rate Limits</h4>
            <div className="flex flex-wrap gap-2">
              {Object.entries(endpoint.rateLimits).map(([tier, limit]) => (
                <span key={tier} className="rounded-lg bg-background px-2.5 py-1 text-xs text-muted-foreground">
                  <span className="capitalize">{tier}</span>: <span className="font-mono">{limit}</span>
                </span>
              ))}
            </div>
          </div>

          {/* Code snippet */}
          <div>
            <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">Example</h4>
            <CodeSnippet endpoint={endpoint} />
          </div>

          {/* Response example */}
          <div>
            <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">Response</h4>
            <ResponseExample data={endpoint.responseExample} />
          </div>

          {/* Try it */}
          <TryItPanel endpoint={endpoint} apiKeys={apiKeys} />
        </div>
      )}
    </div>
  );
}
