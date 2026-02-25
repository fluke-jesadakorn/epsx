'use client';
import { useEffect } from 'react';
import { registerSessionExpiredHandler } from '../../utils/api-client';

export function SessionExpiryProvider({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    registerSessionExpiredHandler(() => {
      // Don't redirect if already on auth page (prevents redirect loop)
      if (window.location.pathname.startsWith('/auth')) { return; }
      window.location.href = '/auth?clear=true';
    });
  }, []);
  return <>{children}</>;
}
