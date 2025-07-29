'use client';

import { PermissionProvider } from './PermissionProvider';

interface ClientPermissionWrapperProps {
  children: React.ReactNode;
  serverData: any;
}

export function ClientPermissionWrapper({ children, serverData }: ClientPermissionWrapperProps) {
  return (
    <PermissionProvider serverData={serverData}>
      {children}
    </PermissionProvider>
  );
}