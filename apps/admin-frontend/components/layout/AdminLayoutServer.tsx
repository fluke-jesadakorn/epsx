/**
 * Server-Side Admin Layout
 * Fetches session data and passes it to the client layout
 */

import dynamic from 'next/dynamic';
import { ReactNode, Suspense } from 'react';

import { getSessionFromJWT } from '@/lib/server/jwt';

// Dynamically import AdminLayoutClient with error boundary
const AdminLayoutClient = dynamic(
  () => import('./AdminLayoutClient').then(mod => ({ default: mod.AdminLayoutClient })),
  {
    loading: () => <AdminLayoutFallback />,
    ssr: true
  }
);

interface AdminLayoutServerProps {
  children: ReactNode;
}

interface Session {
  user?: {
    id: string;
    email: string;
    name?: string;
    role: string;
    permissions: string[];
    packageTier: string;
  };
  isLoggedIn: boolean;
}

/**
 * Fallback UI when AdminLayoutClient fails to load
 */
function AdminLayoutFallback() {
  return (
    <div className="flex min-h-screen bg-gray-900">
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center space-y-4">
          <div className="w-8 h-8 border-4 border-yellow-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
          <p className="text-white">Loading admin interface...</p>
        </div>
      </div>
    </div>
  );
}

/**
 * Error boundary fallback for critical layout errors
 * @param root0
 * @param root0.children
 */
function AdminLayoutError({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <div className="p-6">
        <h1 className="text-xl font-bold mb-4">Admin Layout Error</h1>
        <p className="text-gray-300 mb-6">There was an error loading the admin interface.</p>
        <div className="bg-gray-800 p-4 rounded">
          {children}
        </div>
      </div>
    </div>
  );
}

/**
 * Server component that fetches session and passes it to the client layout
 * @param root0
 * @param root0.children
 */
export async function AdminLayoutServer({ children }: AdminLayoutServerProps) {
  // Fetch session on the server with enhanced error handling
  let session: Session | null = null;
  let sessionError: string | null = null;
  
  try {
    const sessionData = await getSessionFromJWT();
    if (sessionData?.isAuthenticated && sessionData.user) {
      session = {
        user: {
          id: sessionData.user.sub,
          email: sessionData.user.email,
          name: sessionData.user.name,
          role: 'admin', // Default role for admin frontend
          permissions: sessionData.user.permissions || [],
          packageTier: 'ENTERPRISE' // Default tier for admin users
        },
        isLoggedIn: sessionData.isAuthenticated
      };
    }
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to get session data in layout:', _error);
    sessionError = _error instanceof Error ? _error.message : 'Unknown session error';
  }

  // If there's a critical session error, render error fallback
  if (sessionError) {
    return (
      <AdminLayoutError>
        <p>Session Error: {sessionError}</p>
        <div className="mt-4">
          {children}
        </div>
      </AdminLayoutError>
    );
  }

  try {
    return (
      <Suspense fallback={<AdminLayoutFallback />}>
        <AdminLayoutClient session={session}>
          {children}
        </AdminLayoutClient>
      </Suspense>
    );
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Critical AdminLayoutClient error:', _error);
    return (
      <AdminLayoutError>
        <p>Layout Component Error: {_error instanceof Error ? _error.message : 'Unknown error'}</p>
        <div className="mt-4">
          {children}
        </div>
      </AdminLayoutError>
    );
  }
}