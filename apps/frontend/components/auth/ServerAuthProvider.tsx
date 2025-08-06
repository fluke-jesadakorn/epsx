import { getInitialAuthState } from '@/lib/server-auth';
import { AuthProvider } from '@/context/auth-context';
import { PackageTier as _PackageTier } from '@epsx/types';

interface ServerAuthProviderProps {
  children: React.ReactNode;
}

/**
 * Server-side wrapper for AuthProvider that hydrates client state with server state
 * Uses server actions for improved performance and compatibility
 */
export async function ServerAuthProvider({ children }: ServerAuthProviderProps) {
  try {
    // Get server-side auth state using new server actions approach
    const initialAuthState = await getInitialAuthState();
    
    // Pass the complete auth state to client
    const clientAuthState = initialAuthState.isAuthenticated ? {
      user: initialAuthState.user,
      permissions: initialAuthState.permissions,
      packageTier: initialAuthState.packageTier,
    } : undefined;

    return (
      <AuthProvider initialAuthState={clientAuthState}>
        {children}
      </AuthProvider>
    );
  } catch (_error) {
    // Fallback to client-only auth on server error (unauthenticated state)
    return (
      <AuthProvider>
        {children}
      </AuthProvider>
    );
  }
}