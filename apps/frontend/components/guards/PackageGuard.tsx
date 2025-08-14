/**
 * Server-Side Package Tier Guard
 * Protects premium features based on JWT package tier
 */
import { ReactNode } from 'react';
import { requirePackageTier, hasPackageTier, type EPSXJWTPayload } from '@/lib/server/auth';

interface PackageGuardProps {
  tier: string;
  children: ReactNode;
  redirectTo?: string;
}

export default async function PackageGuard({ 
  tier, 
  children, 
  redirectTo 
}: PackageGuardProps) {
  await requirePackageTier(tier, redirectTo);
  return <>{children}</>;
}

interface WithPackageProps extends PackageGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithPackage({ 
  tier, 
  children, 
  redirectTo 
}: WithPackageProps) {
  const user = await requirePackageTier(tier, redirectTo);
  return <>{children(user)}</>;
}

interface ConditionalPackageProps {
  tier: string;
  children: ReactNode;
  fallback?: ReactNode;
}

export async function ConditionalPackage({ 
  tier, 
  children, 
  fallback = null 
}: ConditionalPackageProps) {
  const hasRequiredTier = await hasPackageTier(tier);
  return <>{hasRequiredTier ? children : fallback}</>;
}