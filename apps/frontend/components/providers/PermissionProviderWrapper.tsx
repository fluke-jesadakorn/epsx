'use client';

interface PermissionProviderWrapperProps {
  children: React.ReactNode;
  serverData?: any;
}

export function PermissionProviderWrapper({ children, serverData }: PermissionProviderWrapperProps) {
  // Simple wrapper that just passes through children
  // ServerData can be used for future enhancements
  return <>{children}</>;
}