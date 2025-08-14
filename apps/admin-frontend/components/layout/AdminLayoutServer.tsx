/**
 * Server-Side Admin Layout
 * Renders the layout with our custom session management
 */

import { ReactNode } from 'react';
import { getSessionFromJWT } from '@/lib/server/jwt';
import { AdminLayoutClient } from './AdminLayoutClient';

interface AdminLayoutServerProps {
  children: ReactNode;
}

/**
 * Server component that fetches session and passes it to the client layout
 */
export async function AdminLayoutServer({ children }: AdminLayoutServerProps) {
  // Fetch session on the server
  let session = null;
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

  // Pass server data to client component
  return (
    <AdminLayoutClient session={session}>
      {children}
    </AdminLayoutClient>
  );
}