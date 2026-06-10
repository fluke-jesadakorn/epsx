'use client';

import { Badge } from '@/components/ui/badge';
import type { AuthUser } from '@/lib/server/actions';
import { useUsageData } from './hooks/use-usage-data';

interface UsageMonitorProps {
  currentUser: AuthUser;
}

const RANGES = [7, 30, 90] as const;

const methodColor: Record<string, string> = {
  GET: 'bg-blue-500/10 text-blue-400',
  POST: 'bg-green-500/10 text-green-400',
  DELETE: 'bg-red-500/10 text-red-400',
};
 
export function UsageMonitor({ currentUser: _currentUser }: UsageMonitorProps) {
  const { keys, stats, history, topEndpoints, loading, days, setRange } = useUsageData();
  const totalKeys = keys.length;
  const activeKeys = keys.filter((k) => k.status === 'active').length;

  return (
    <div className="space-y-6">
      {/* Stats grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {[
          { label: 'Total Requests', value: stats.total_requests.toLocaleString(), sub: 'All time', color: 'text-emerald-400' },
          { label: 'Requests (24h)', value: stats.requests_24h.toLocaleString(), sub: `${totalKeys} keys (${activeKeys} active)`, color: 'text-blue-400' },
          { label: 'Error Rate (24h)', value: `${stats.error_rate_24h.toFixed(2)}%`, sub: 'Failed requests', color: 'text-purple-400' },
          { label: 'Success Rate', value: `${stats.average_success_rate.toFixed(1)}%`, sub: 'Global average', color: 'text-amber-400' },
        ].map((s) => (
          <div key={s.label} className="rounded-2xl border border-border/20 bg-card p-5 shadow-xl">
            <p className="text-xs font-medium text-muted-foreground">{s.label}</p>
            <p className={`mt-2 text-2xl font-bold ${s.color}`}>{loading ? '...' : s.value}</p>
            <p className="mt-1 text-[11px] text-muted-foreground/60">{s.sub}</p>
          </div>
        ))}
      </div>

      {/* Per-key usage */}
      <div className="rounded-2xl border border-border/20 bg-card shadow-xl">
        <div className="border-b border-border/10 px-5 py-4">
          <h3 className="text-sm font-semibold text-foreground">Usage by API Key</h3>
        </div>
        <div className="p-5">
          {loading ? (
            <div className="flex justify-center py-8">
              <div className="h-6 w-6 animate-spin rounded-full border-2 border-t-[#7645d9] border-border/20" />
            </div>
          ) : keys.length === 0 ? (
            <p className="py-8 text-center text-sm text-muted-foreground">No API keys yet</p>
          ) : (
            <div className="space-y-2">
              {keys.map((k) => (
                <div key={k.id} className="flex items-center justify-between rounded-xl bg-background p-3">
                  <div>
                    <span className="text-sm font-medium text-foreground">{k.name}</span>
                    <Badge className={`ml-2 text-[10px] ${k.status === 'active' ? 'border-green-500/30 bg-green-500/10 text-green-400' : 'border-border bg-background text-muted-foreground'}`}>
                      {k.status}
                    </Badge>
                  </div>
                  <span className="text-lg font-bold text-emerald-400">{k.total_requests.toLocaleString()}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* History chart */}
      <div className="rounded-2xl border border-border/20 bg-card shadow-xl">
        <div className="flex items-center justify-between border-b border-border/10 px-5 py-4">
          <h3 className="text-sm font-semibold text-foreground">Usage History</h3>
          <div className="flex gap-1">
            {RANGES.map((r) => (
              <button
                key={r}
                type="button"
                onClick={() => setRange(r)}
                className={`rounded-md px-2.5 py-1 text-xs font-medium transition-colors ${
                  days === r ? 'bg-[#7645d9]/20 text-[#7645d9]' : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {r}d
              </button>
            ))}
          </div>
        </div>
        <div className="flex h-52 items-end gap-1 px-5 pb-2 pt-8">
          {loading ? (
            <div className="flex h-full w-full items-center justify-center">
              <div className="h-6 w-6 animate-spin rounded-full border-2 border-t-[#7645d9] border-border/20" />
            </div>
          ) : history.length === 0 ? (
            <p className="w-full text-center text-sm text-muted-foreground">No data</p>
          ) : (
            history.map((pt) => {
              const max = Math.max(...history.map((h) => h.count), 1);
              const pct = (pt.count / max) * 100;
              const d = new Date(pt.bucket);
              return (
                <div key={pt.bucket} className="group relative flex flex-1 flex-col items-center">
                  <div className="relative w-full rounded-t bg-emerald-500/15 transition-colors hover:bg-emerald-500/30" style={{ height: `${Math.max(pct, 4)}%` }}>
                    <div className="absolute -top-7 left-1/2 -translate-x-1/2 rounded bg-black/80 px-1.5 py-0.5 text-[10px] text-white opacity-0 group-hover:opacity-100 whitespace-nowrap">
                      {pt.count}
                    </div>
                    <div className="absolute bottom-0 left-0 right-0 rounded-t bg-emerald-500" style={{ height: `${pct}%` }} />
                  </div>
                  <span className="mt-1.5 text-[9px] text-muted-foreground/60 truncate w-full text-center">
                    {d.toLocaleDateString(undefined, { day: 'numeric', month: 'short' })}
                  </span>
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Top endpoints */}
      <div className="rounded-2xl border border-border/20 bg-card shadow-xl">
        <div className="border-b border-border/10 px-5 py-4">
          <h3 className="text-sm font-semibold text-foreground">Top Endpoints ({days}d)</h3>
        </div>
        <div className="p-5">
          {loading ? (
            <p className="py-4 text-center text-sm text-muted-foreground">Loading...</p>
          ) : topEndpoints.length === 0 ? (
            <p className="py-4 text-center text-sm text-muted-foreground">No endpoint data</p>
          ) : (
            <div className="space-y-2">
              {topEndpoints.map((ep) => (
                <div key={ep.endpoint} className="flex items-center justify-between rounded-xl bg-background p-3">
                  <div className="flex items-center gap-2">
                    <span className={`rounded-md px-2 py-0.5 text-xs font-bold ${methodColor[ep.method] ?? 'text-muted-foreground'}`}>
                      {ep.method}
                    </span>
                    <span className="font-mono text-xs text-muted-foreground">{ep.endpoint}</span>
                  </div>
                  <span className="text-sm font-bold text-emerald-400">{ep.count.toLocaleString()}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
