import { Suspense } from 'react';
import { ClientAuthProvider } from '../client/ClientAuthProvider';
import type { SessionInfo } from '@/lib/auth-server';

interface OptimizedAuthProviderProps {
  children: React.ReactNode;
  initialSession: SessionInfo;
}

/**
 * Server-optimized auth provider with proper SSR hydration
 */
export function OptimizedAuthProvider({ children, initialSession }: OptimizedAuthProviderProps) {
  // Transform session for client consumption
  const initialAuthState = initialSession.isAuthenticated ? {
    user: {
      user_id: initialSession.email?.split('@')[0] || 'unknown',
      email: initialSession.email || '',
      role: 'user',
      permissions: initialSession.permissions || [],
      subscription_tier: initialSession.packageTier?.toLowerCase() || 'free',
      package_tier: initialSession.packageTier || 'FREE',
      expires_at: new Date(Date.now() + 86400000).toISOString(),
      session_type: 'server_hydrated',
      emailVerified: true,
      displayName: initialSession.displayName,
      photoURL: null,
      phoneNumber: null,
    },
    permissions: initialSession.permissions || [],
    packageTier: initialSession.packageTier || 'FREE',
  } : undefined;

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <ClientAuthProvider initialAuthState={initialAuthState}>
        {children}
      </ClientAuthProvider>
    </Suspense>
  );
}