/**
 * JWT Server-Side Authentication Provider
 * Simplified server-side auth validation using custom JWT
 */

import { ReactNode } from 'react';
import { redirect } from 'next/navigation';
import { getServerSession } from '@/lib/auth';

interface ServerAuthProviderProps {
  children: ReactNode;
  requireAuth?: boolean;
  requireAdmin?: boolean;
}

/**
 * Custom JWT server-side authentication wrapper
 * This component runs on the server and validates authentication before rendering
 */
export async function ServerAuthProvider({
  children,
  requireAuth = true,
  requireAdmin = false,
}: ServerAuthProviderProps) {
  
  if (requireAuth) {
    const session = await getServerSession();
    
    if (!session?.user) {
      console.log('🔐 Server auth: User not authenticated, redirecting to backend Chef Kitchen login');
      const { redirectToBackendAdminLogin } = await import('@/lib/server/auth');
      redirectToBackendAdminLogin();
    }

    if (requireAdmin) {
      // Check if user has admin access
      const userAdminModules = (session.user as any).admin_modules as string[] || [];
      
      if (userAdminModules.length === 0) {
        console.log('🔐 Server auth: User lacks admin access', {
          admin_modules: userAdminModules,
          email: session.user.email
        });
        redirect('/access-denied?reason=insufficient_admin_access');
      }
      
      console.log('✅ Server auth: Admin access verified', {
        user_id: session.user.id,
        email: session.user.email,
        admin_modules: userAdminModules
      });
    }
  }

  // Children are rendered - client components will use custom auth hooks
  return <>{children}</>;
}

/**
 * Higher-order component for protecting routes with admin access
 */
export function withAdminAuth<T extends {}>(Component: React.ComponentType<T>) {
  return async function AdminProtectedComponent(props: T) {
    return (
      <ServerAuthProvider requireAuth={true} requireAdmin={true}>
        <Component {...props} />
      </ServerAuthProvider>
    );
  };
}