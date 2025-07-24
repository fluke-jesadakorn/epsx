import { getSessionInfo } from '@/lib/auth-server';
import { AuthProvider } from '@/context/auth-context';
import { PackageTier } from '@epsx/types';

interface ServerAuthProviderProps {
  children: React.ReactNode;
}

/**
 * Server-side wrapper for AuthProvider that hydrates client state with server state
 */
export async function ServerAuthProvider({ children }: ServerAuthProviderProps) {
  try {
    // Get server-side auth state
    const sessionInfo = await getSessionInfo();
    
    // Transform server auth state for client consumption
    const initialAuthState = sessionInfo.isAuthenticated ? {
      user: {
        user_id: sessionInfo.email?.split('@')[0] || 'unknown',
        email: sessionInfo.email || '',
        role: 'user', // Default role, will be updated by client
        permissions: sessionInfo.permissions || [],
        subscription_tier: sessionInfo.packageTier?.toLowerCase() || 'free',
        package_tier: sessionInfo.packageTier || 'FREE',
        expires_at: new Date(Date.now() + 86400000).toISOString(), // 24 hours
        session_type: 'server_hydrated',
        // Firebase compatibility
        uid: sessionInfo.email?.split('@')[0] || 'unknown',
        emailVerified: true,
        displayName: sessionInfo.displayName,
        photoURL: null,
        providerData: [],
        isAnonymous: false,
        metadata: { creationTime: '', lastSignInTime: '' },
        refreshToken: null,
        tenantId: null,
        phoneNumber: null,
        providerId: 'server-auth',
      },
      permissions: sessionInfo.permissions || [],
      packageTier: (sessionInfo.packageTier as PackageTier) || PackageTier.FREE,
    } : undefined;

    return (
      <AuthProvider initialAuthState={initialAuthState}>
        {children}
      </AuthProvider>
    );
  } catch (error) {
    console.error('Server auth initialization failed:', error);
    
    // Fallback to client-only auth on server error
    return (
      <AuthProvider>
        {children}
      </AuthProvider>
    );
  }
}