/**
 * Server-Side Admin Layout
 * Fetches session data and passes it to the client layout
 */

import { ReactNode } from 'react';
import { getSessionFromJWT } from '@/lib/server/jwt';
import { AdminLayoutClient } from './AdminLayoutClient';

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
 * Server component that fetches session and passes it to the client layout
 */
export async function AdminLayoutServer({ children }: AdminLayoutServerProps) {
  // Fetch session on the server
  let session: Session | null = null;
  try {
    const sessionData = await getSessionFromJWT();
    if (sessionData?.isAuthenticated && sessionData.user) {
      session = {
        user: sessionData.user,
        isLoggedIn: sessionData.isAuthenticated
      };
    }
  } catch (error) {
    console.error('Failed to get session data in layout:', error);
  }

  return (
    <AdminLayoutClient session={session}>
      {children}
    </AdminLayoutClient>
  );
}