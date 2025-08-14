'use client';

import { ReactNode } from 'react';

interface ClientProvidersProps {
  children: ReactNode;
}

/**
 * Simplified client providers
 * Auth state is managed by Zustand store, no provider needed
 */
export function ClientProviders({ children }: ClientProvidersProps) {
  return <>{children}</>;
}