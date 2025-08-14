/**
 * Server-Side Authentication Guard
 * Protects components/pages with JWT-based authentication
 */
import { ReactNode } from 'react';
import { redirect } from 'next/navigation';
import { requireAuth, type EPSXJWTPayload } from '@/lib/server/auth';

interface AuthGuardProps {
  children: ReactNode;
  redirectTo?: string;
}

export default async function AuthGuard({ children, redirectTo }: AuthGuardProps) {
  await requireAuth(redirectTo);
  return <>{children}</>;
}

interface WithUserProps extends AuthGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithUser({ children, redirectTo }: WithUserProps) {
  const user = await requireAuth(redirectTo);
  return <>{children(user)}</>;
}