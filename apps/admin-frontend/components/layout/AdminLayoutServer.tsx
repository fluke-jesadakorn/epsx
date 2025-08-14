/**
 * Server-Side Admin Layout
 * Renders the layout with our custom session management
 */

import { ReactNode } from 'react';
import { getSession } from '@/lib/auth/session';
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
    const sessionData = await getSession();
    if (sessionData?.isLoggedIn && sessionData.user) {
      session = {
        user: sessionData.user,
        isLoggedIn: sessionData.isLoggedIn
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