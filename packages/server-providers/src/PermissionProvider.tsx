'use client';

import { createContext, useContext } from 'react';

interface PermissionContextData {
  paymentStatus: any;
  permissions: any;
  featureAccess: any;
  rankingAccess: any;
  error: string | null;
}

interface PermissionProviderProps {
  children: React.ReactNode;
  serverData: PermissionContextData;
}

const PermissionContext = createContext<PermissionContextData | null>(null);

export function PermissionProvider({ children, serverData }: PermissionProviderProps) {
  return (
    <PermissionContext.Provider value={serverData}>
      {children}
    </PermissionContext.Provider>
  );
}

export function usePermissionContext() {
  const context = useContext(PermissionContext);
  if (!context) {
    throw new Error('usePermissionContext must be used within a PermissionProvider');
  }
  return context;
}