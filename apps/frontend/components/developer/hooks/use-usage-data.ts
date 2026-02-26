'use client';

import { useState, useCallback } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  getApiKeysAction,
  getUsageStatsAction,
  getUsageHistoryAction,
  getTopEndpointsAction,
} from '@/app/actions/developer';

interface KeySummary {
  id: string;
  name: string;
  total_requests: number;
  status: string;
  created_at: string;
  expires_at: string | null;
}

interface UsageStats {
  total_requests: number;
  average_success_rate: number;
  requests_24h: number;
  error_rate_24h: number;
}

interface HistoryPoint {
  bucket: string;
  count: number;
}

interface TopEndpoint {
  endpoint: string;
  method: string;
  count: number;
}

export function useUsageData() {
  const [days, setDays] = useState(7);

  const { data: keysRes, isLoading: lk } = useQuery({
    queryKey: ['dev-keys'],
    queryFn: () => getApiKeysAction({ limit: 100 }),
  });

  const { data: statsRes, isLoading: ls } = useQuery({
    queryKey: ['dev-stats'],
    queryFn: getUsageStatsAction,
  });

  const { data: historyRes, isLoading: lh } = useQuery({
    queryKey: ['dev-history', days],
    queryFn: () => getUsageHistoryAction(days),
  });

  const { data: endpointsRes, isLoading: le } = useQuery({
    queryKey: ['dev-endpoints', days],
    queryFn: () => getTopEndpointsAction(days),
  });

  const loading = lk || ls || lh || le;

  // Process keys
   
  const rawKeys = (keysRes?.success && keysRes.data) ? ((keysRes.data as any).api_keys ?? []) : [];
   
  const keys: KeySummary[] = rawKeys.map((k: any) => ({
    id: k.id,
    name: k.name,
    total_requests: k.usage_count ?? 0,
    status: k.is_active ? 'active' : 'inactive',
    created_at: k.created_at ?? '',
    expires_at: null,
  }));

  // Process stats
   
  const rawStats = (statsRes?.success && statsRes.data) ? ((statsRes.data as any).data ?? statsRes.data) : null;
  const stats: UsageStats = rawStats ?? { total_requests: 0, average_success_rate: 100, requests_24h: 0, error_rate_24h: 0 };

  // Process history
   
  const history: HistoryPoint[] = (historyRes?.success && historyRes.data) ? ((historyRes.data as any).data ?? historyRes.data) : [];

  // Process endpoints
   
  const topEndpoints: TopEndpoint[] = (endpointsRes?.success && endpointsRes.data) ? ((endpointsRes.data as any).data ?? endpointsRes.data) : [];

  const setRange = useCallback((d: number) => setDays(d), []);

  return { keys, stats, history, topEndpoints, loading, days, setRange };
}
