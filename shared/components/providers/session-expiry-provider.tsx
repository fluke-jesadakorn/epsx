'use client';
import { useEffect } from 'react';
import { registerSessionExpiredHandler } from '../../utils/api-client';

export function SessionExpiryProvider({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    registerSessionExpiredHandler(() => {
      // Don't redirect if already on auth page or root (gate overlay handles it)
      const p = window.location.pathname;
      if (p.startsWith('/auth') || p === '/') { return; }
      window.location.href = '/auth?clear=true';
    });
  }, []);
  return <>{children}</>;
}
