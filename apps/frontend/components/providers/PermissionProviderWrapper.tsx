'use client';

import dynamic from 'next/dynamic';

const ClientPermissionWrapper = dynamic(
  () => import('@epsx/server-providers/client').then(mod => ({ default: mod.ClientPermissionWrapper })),
  { ssr: false }
);

interface PermissionProviderWrapperProps {
  children: React.ReactNode;
  serverData: any;
}

export function PermissionProviderWrapper({ children, serverData }: PermissionProviderWrapperProps) {
  return (
    <ClientPermissionWrapper serverData={serverData}>
      {children}
    </ClientPermissionWrapper>
  );
}