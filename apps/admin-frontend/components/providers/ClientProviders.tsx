'use client';

import { AdminAuthWrapper } from './AdminAuthWrapper';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  return (
    <AdminAuthWrapper>
      {children}
    </AdminAuthWrapper>
  );
}