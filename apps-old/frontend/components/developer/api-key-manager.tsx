'use client';

import { useState, useCallback } from 'react';
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel,
  AlertDialogContent, AlertDialogDescription, AlertDialogFooter,
  AlertDialogHeader, AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import type { AuthUser } from '@/lib/server/actions';
import type { UserApiKey } from '@/shared/api/users';
import { useApiKeys } from './hooks/use-api-keys';
import { ApiKeyCreateForm } from './api-key-create-form';
import { ApiKeyCard } from './api-key-card';

interface APIKeyManagerProps {
  currentUser: AuthUser;
  onStatsChange?: () => void;
}

export function APIKeyManager({ currentUser, onStatsChange }: APIKeyManagerProps) {
  const {
    keys, permissions, loading, hasPlans,
    generatedKey, showNewKey, dismissNewKey,
    createKey, revokeKey, refresh,
  } = useApiKeys({ walletAddress: currentUser.walletAddress ?? '', onStatsChange });

  const [revokeTarget, setRevokeTarget] = useState<UserApiKey | null>(null);

  const confirmRevoke = useCallback(() => {
    if (revokeTarget) {
      void revokeKey(revokeTarget.id);
      setRevokeTarget(null);
    }
  }, [revokeTarget, revokeKey]);

  return (
    <div className="space-y-6">
      {/* Create form */}
      <ApiKeyCreateForm
        permissions={permissions}
        hasPlans={hasPlans}
        loading={loading}
        generatedKey={generatedKey}
        showNewKey={showNewKey}
        onDismissNewKey={dismissNewKey}
        onCreateKey={createKey}
        isAdmin={currentUser.role === 'admin'}
      />

      {/* Key list */}
      <div className="rounded-2xl border border-border/20 bg-card shadow-xl">
        <div className="flex items-center justify-between border-b border-border/10 px-5 py-4">
          <h3 className="text-lg font-semibold text-foreground">Your API Keys</h3>
          <button
            type="button"
            onClick={() => void refresh()}
            disabled={loading}
            className="rounded-lg bg-background px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground disabled:opacity-50"
          >
            {loading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
        <div className="p-5">
          {loading && keys.length === 0 ? (
            <div className="flex justify-center py-10">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-t-[#7645d9] border-border/20" />
            </div>
          ) : keys.length === 0 ? (
            <p className="py-10 text-center text-sm text-muted-foreground">
              No API keys yet. Create your first key above.
            </p>
          ) : (
            <div className="space-y-3">
              {keys.map((k) => (
                <ApiKeyCard key={k.id} apiKey={k} onRevoke={setRevokeTarget} disabled={loading} />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Security tips */}
      <div className="rounded-2xl border border-blue-500/20 bg-blue-500/5 p-5">
        <h4 className="mb-2 text-sm font-semibold text-blue-300">Security Best Practices</h4>
        <ul className="list-inside list-disc space-y-1 text-xs text-blue-300/80">
          <li>Never commit API keys to version control</li>
          <li>Use environment variables for key storage</li>
          <li>Rotate keys regularly for production apps</li>
          <li>Revoke unused keys promptly</li>
        </ul>
      </div>

      {/* Revoke dialog */}
      <AlertDialog open={revokeTarget !== null} onOpenChange={(open) => { if (!open) { setRevokeTarget(null); } }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Revoke API Key</AlertDialogTitle>
            <AlertDialogDescription>
              Revoke <span className="font-mono text-xs">{revokeTarget?.name}</span>? Applications using this key will stop working immediately.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={loading}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={confirmRevoke}
              disabled={loading}
              className="bg-red-600 text-white hover:bg-red-700"
            >
              {loading ? 'Revoking...' : 'Revoke Key'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
