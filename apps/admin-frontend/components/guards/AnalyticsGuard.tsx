/**
 * Server-Side Analytics Guard
 * Protects analytics components based on JWT admin modules
 */
import { ReactNode } from 'react';
import { requireAnalytics, canAccessAnalytics, type EPSXJWTPayload } from '@/lib/server/auth';

interface AnalyticsGuardProps {
  children: ReactNode;
  redirectTo?: string;
}

export default async function AnalyticsGuard({ 
  children, 
  redirectTo 
}: AnalyticsGuardProps) {
  await requireAnalytics(redirectTo);
  return <>{children}</>;
}

interface WithAnalyticsProps extends AnalyticsGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithAnalytics({ 
  children, 
  redirectTo 
}: WithAnalyticsProps) {
  const user = await requireAnalytics(redirectTo);
  return <>{children(user)}</>;
}

interface ConditionalAnalyticsProps {
  children: ReactNode;
  fallback?: ReactNode;
}

export async function ConditionalAnalytics({ 
  children, 
  fallback = null 
}: ConditionalAnalyticsProps) {
  const canAccess = await canAccessAnalytics();
  return <>{canAccess ? children : fallback}</>;
}