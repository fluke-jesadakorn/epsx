'use client';

import { useCallback, useEffect, useState } from 'react';
import { useFrontendApiClient } from '@/shared/hooks/use-api-client';
import type { UserApiKey } from '@/shared/api/users';
import { toast } from 'sonner';

interface UseApiKeysCtx {
  walletAddress: string;
  onStatsChange?: () => void;
}
 
export function useApiKeys({ walletAddress, onStatsChange }: UseApiKeysCtx) {
  const { users, plans } = useFrontendApiClient();
  const [keys, setKeys] = useState<UserApiKey[]>([]);
  const [permissions, setPermissions] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasPlans, setHasPlans] = useState(false);
  const [generatedKey, setGeneratedKey] = useState('');
  const [showNewKey, setShowNewKey] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const keysRes = await users.getApiKeys();
      if (keysRes.success && keysRes.data) {
        setKeys(keysRes.data.api_keys);
      }
      if (walletAddress) {
        const memRes = await plans.getWalletMemberships(walletAddress);
        if (memRes.success && memRes.data) {
          const ms = Array.isArray(memRes.data) ? memRes.data : [];
          setHasPlans(ms.length > 0);
          const perms = new Set<string>();
          ms.forEach((m) => {
            if (Array.isArray(m.permissions)) {
              m.permissions.forEach((p: string) => perms.add(p));
            }
          });
          setPermissions(Array.from(perms).sort());
        }
      }
    } catch {
      // silent
    } finally {
      setLoading(false);
    }
  }, [users, plans, walletAddress]);

  useEffect(() => { void refresh(); }, [refresh]);

  const createKey = useCallback(async (name: string, selectedPerms: string[]) => {
    setLoading(true);
    try {
      const res = await users.createApiKey({ client_name: name, permissions: selectedPerms });
      if (res.success && res.data) {
        const key = res.data.key ?? '';
        setGeneratedKey(key);
        setShowNewKey(true);
        toast.success('API key created');
        await refresh();
        onStatsChange?.();
        return true;
      }
      toast.error('Failed to create API key');
      return false;
    } catch {
      toast.error('Failed to create API key');
      return false;
    } finally {
      setLoading(false);
    }
  }, [users, refresh, onStatsChange]);

  const revokeKey = useCallback(async (id: string) => {
    setLoading(true);
    try {
      const res = await users.deleteApiKey(id);
      if (res.success) {
        toast.success('API key revoked');
        await refresh();
        onStatsChange?.();
      } else {
        toast.error('Failed to revoke API key');
      }
    } catch {
      toast.error('Failed to revoke API key');
    } finally {
      setLoading(false);
    }
  }, [users, refresh, onStatsChange]);

  const dismissNewKey = useCallback(() => setShowNewKey(false), []);

  return {
    keys, permissions, loading, hasPlans,
    generatedKey, showNewKey, dismissNewKey,
    createKey, revokeKey, refresh,
  };
}
