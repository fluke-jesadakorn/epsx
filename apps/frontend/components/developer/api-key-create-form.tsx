'use client';

import { useState, useCallback } from 'react';
import { Button } from '@/components/ui';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { copyToClipboard as copyUtil } from '@/utils/clipboard';
import { toast } from 'sonner';
import { PlanTransferList } from './plan-transfer-list';

interface CreateFormProps {
  permissions: string[];
  hasPlans: boolean;
  loading: boolean;
  generatedKey: string;
  showNewKey: boolean;
  onDismissNewKey: () => void;
  onCreateKey: (name: string, perms: string[]) => Promise<boolean>;
  isAdmin: boolean;
}

const EXPIRY_PRESETS = [
  { label: '30 Days', days: 30 },
  { label: '90 Days', days: 90 },
  { label: '1 Year', days: 365 },
  { label: 'Never', days: 0 },
];
 
export function ApiKeyCreateForm({
  permissions, hasPlans, loading, generatedKey, showNewKey,
  onDismissNewKey, onCreateKey, isAdmin,
}: CreateFormProps) {
  const [name, setName] = useState('');
  const [selected, setSelected] = useState<string[]>([]);
  const [_expiresAt, setExpiresAt] = useState('');
  const canCreate = hasPlans || isAdmin;

  const handleCreate = useCallback(async () => {
    if (!name.trim()) { toast.error('Enter a key name'); return; }
    const ok = await onCreateKey(name, selected);
    if (ok) { setName(''); setSelected([]); setExpiresAt(''); }
  }, [name, selected, onCreateKey]);

  if (!canCreate && !loading) {
    return (
      <div className="rounded-2xl border border-border/20 bg-card p-6 text-center shadow-xl">
        <Badge className="mb-3 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white border-0">Premium Required</Badge>
        <p className="text-sm text-muted-foreground">API key generation requires a plan subscription.</p>
      </div>
    );
  }

  return (
    <div className="rounded-2xl border border-border/20 bg-card shadow-xl">
      <div className="flex items-center gap-3 border-b border-border/10 px-5 py-4">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-[#7645d9] to-[#5a33b8] shadow-lg shadow-purple-500/25">
          <svg className="h-5 w-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
        </div>
        <h3 className="text-lg font-semibold text-foreground">Create API Key</h3>
      </div>

      <div className="space-y-5 p-5">
        {/* Name */}
        <div>
          <label className="mb-1 block text-sm font-medium text-muted-foreground">Key Name</label>
          <Input
            placeholder="e.g. Production Server"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="bg-background"
          />
        </div>

        {/* Permissions */}
        <PlanTransferList available={permissions} selected={selected} onChange={setSelected} />
        {permissions.length === 0 && (
          <p className="text-xs text-amber-400">No permissions available — check your plan assignments.</p>
        )}

        {/* Expiry presets */}
        <div>
          <label className="mb-2 block text-sm font-medium text-muted-foreground">Expiration</label>
          <div className="flex flex-wrap gap-2">
            {EXPIRY_PRESETS.map((p) => (
              <button
                key={p.label}
                type="button"
                onClick={() => {
                  if (p.days === 0) { setExpiresAt(''); }
                  else {
                    const d = new Date();
                    d.setDate(d.getDate() + p.days);
                    setExpiresAt(d.toISOString());
                  }
                }}
                className="rounded-lg border border-border/30 bg-background px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-accent"
              >
                {p.label}
              </button>
            ))}
          </div>
        </div>

        <Button
          onClick={() => void handleCreate()}
          disabled={loading || !name.trim() || (permissions.length > 0 && selected.length === 0)}
          className="w-full bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white hover:opacity-90"
        >
          {loading ? 'Creating...' : 'Create API Key'}
        </Button>

        {/* New key reveal */}
        {showNewKey && generatedKey && (
          <div className="rounded-xl border border-green-500/30 bg-green-500/5 p-4">
            <p className="mb-2 text-sm font-semibold text-green-400">Key Created — Copy it now!</p>
            <div className="flex items-center gap-2">
              <input
                readOnly
                value={generatedKey}
                className="flex-1 rounded-lg bg-slate-900 px-3 py-2 font-mono text-sm text-green-400 truncate"
              />
              <Button
                size="icon"
                className="shrink-0 bg-green-600 hover:bg-green-700"
                onClick={async () => {
                  const ok = await copyUtil(generatedKey);
                  if (ok) {toast.success('Copied');}
                }}
              >
                <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </Button>
            </div>
            <button type="button" onClick={onDismissNewKey} className="mt-2 text-xs text-green-400 hover:text-green-300">
              Dismiss
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
