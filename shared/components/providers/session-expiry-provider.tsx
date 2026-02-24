'use client';
import { useEffect } from 'react';
import { registerSessionExpiredHandler } from '../../utils/api-client';

export function SessionExpiryProvider({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    registerSessionExpiredHandler(() => {
      // Immediate clean redirect without backend dependencies
      window.location.href = '/auth?clear=true';
    });
  }, []);
  return <>{children}</>;
}
