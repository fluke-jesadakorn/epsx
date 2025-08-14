/**
 * Server-Side Admin Module Guard
 * Protects admin components/pages based on JWT admin modules
 */
import { ReactNode } from 'react';
import { requireAdminModule, hasAdminModule, type EPSXJWTPayload } from '@/lib/server/auth';

interface AdminModuleGuardProps {
  module: string;
  children: ReactNode;
  redirectTo?: string;
}

export default async function AdminModuleGuard({ 
  module, 
  children, 
  redirectTo 
}: AdminModuleGuardProps) {
  await requireAdminModule(module, redirectTo);
  return <>{children}</>;
}

interface WithAdminModuleProps extends AdminModuleGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithAdminModule({ 
  module, 
  children, 
  redirectTo 
}: WithAdminModuleProps) {
  const user = await requireAdminModule(module, redirectTo);
  return <>{children(user)}</>;
}

interface ConditionalAdminModuleProps {
  module: string;
  children: ReactNode;
  fallback?: ReactNode;
}

export async function ConditionalAdminModule({ 
  module, 
  children, 
  fallback = null 
}: ConditionalAdminModuleProps) {
  const hasRequiredModule = await hasAdminModule(module);
  return <>{hasRequiredModule ? children : fallback}</>;
}