'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui';
import { copyToClipboard as copyUtil } from '@/utils/clipboard';
import { toast } from 'sonner';
import type { UserApiKey } from '@/shared/api/users';

interface ApiKeyCardProps {
  apiKey: UserApiKey;
  onRevoke: (key: UserApiKey) => void;
  disabled: boolean;
}

export function ApiKeyCard({ apiKey, onRevoke, disabled }: ApiKeyCardProps) {
  return (
    <div className="rounded-2xl border border-border/20 bg-card transition-shadow hover:shadow-xl">
      <div className="p-5">
        {/* Header */}
        <div className="mb-4 flex items-start justify-between">
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold text-foreground">{apiKey.name}</h3>
              <Badge
                className={`text-xs ${apiKey.is_active
                  ? 'border-green-500/30 bg-green-500/10 text-green-400'
                  : 'border-red-500/30 bg-red-500/10 text-red-400'
                }`}
              >
                {apiKey.is_active ? 'Active' : 'Revoked'}
              </Badge>
            </div>
            <p className="mt-0.5 text-xs text-muted-foreground">
              Created {new Date(apiKey.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })}
            </p>
          </div>
        </div>

        {/* Key preview */}
        <div className="mb-4 flex items-center gap-2">
          <input
            readOnly
            value={apiKey.key ?? 'epsx_\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022'}
            className="flex-1 rounded-lg bg-slate-900 px-3 py-2 font-mono text-sm text-green-400 truncate border border-border/10"
          />
          {apiKey.key && (
            <Button
              size="icon"
              variant="ghost"
              className="shrink-0 h-9 w-9"
              onClick={async () => {
                if (apiKey.key) {
                  const ok = await copyUtil(apiKey.key);
                  if (ok) {toast.success('Copied');}
                }
              }}
            >
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </Button>
          )}
        </div>

        {/* Info grid */}
        <div className="grid grid-cols-2 gap-4 border-t border-border/10 pt-4">
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Permissions</p>
            <div className="mt-1.5 flex flex-wrap gap-1">
              {apiKey.scopes.length > 0 ? (
                apiKey.scopes.slice(0, 3).map((s: string) => (
                  <Badge key={s} variant="outline" className="border-purple-500/30 bg-purple-500/5 text-[10px] font-mono text-purple-400">
                    {s}
                  </Badge>
                ))
              ) : (
                <span className="text-xs text-muted-foreground/50">Read only</span>
              )}
              {apiKey.scopes.length > 3 && (
                <span className="text-[10px] text-muted-foreground">+{apiKey.scopes.length - 3}</span>
              )}
            </div>
          </div>
          <div className="text-right">
            <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Usage</p>
            <p className="mt-1.5 text-lg font-bold text-foreground">
              {(apiKey.usage_count ?? 0).toLocaleString()}
            </p>
          </div>
        </div>
      </div>

      {/* Revoke */}
      {apiKey.is_active && (
        <div className="border-t border-border/10 px-5 py-3 text-right">
          <button
            type="button"
            disabled={disabled}
            onClick={() => onRevoke(apiKey)}
            className="text-xs font-medium text-red-400/80 hover:text-red-400 disabled:opacity-50"
          >
            Revoke Key
          </button>
        </div>
      )}
    </div>
  );
}
