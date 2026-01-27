'use client';

import { QueryClientProvider } from '@tanstack/react-query';
import { useState, type ReactNode } from 'react';
import { createQueryClient, type Platform } from './query-client';

export interface UnifiedStateProviderProps {
  children: ReactNode;
  platform: Platform;
}

/**
 * Unified state provider for both admin and frontend apps.
 * Wraps QueryClientProvider with platform-specific configuration.
 */
export function UnifiedStateProvider({ children, platform }: UnifiedStateProviderProps) {
  const [queryClient] = useState(() => createQueryClient(platform));

  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}
