'use client';

import { useState, useCallback } from 'react';
import { useQuery } from '@tanstack/react-query';
import { getDeveloperOverviewAction } from '@/app/actions/developer';

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

  const { data: res, isLoading } = useQuery({
    queryKey: ['dev-overview', days],
    queryFn: () => getDeveloperOverviewAction(days),
    staleTime: 60_000,
  });

  const overview = res?.success === true && res.data !== undefined ? res.data : null;

  const rawKeys = overview?.api_keys?.api_keys ?? [];
  const keys: KeySummary[] = (rawKeys as any[]).map((k: any) => ({
    id: k.id,
    name: k.client_name,
    total_requests: k.total_requests ?? 0,
    status: k.status ?? 'active',
    created_at: k.created_at ?? '',
    expires_at: k.expires_at ?? null,
  }));

  const rawStats = overview?.stats as any;
  const stats: UsageStats = rawStats ?? { total_requests: 0, average_success_rate: 100, requests_24h: 0, error_rate_24h: 0 };

  const history: HistoryPoint[] = (overview?.history as HistoryPoint[] | null) ?? [];
  const topEndpoints: TopEndpoint[] = (overview?.top_endpoints as TopEndpoint[] | null) ?? [];

  const setRange = useCallback((d: number) => setDays(d), []);

  return { keys, stats, history, topEndpoints, loading: isLoading, days, setRange };
}
