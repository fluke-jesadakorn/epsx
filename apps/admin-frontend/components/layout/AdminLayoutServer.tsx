/**
 * Server-Side Admin Layout
 * Renders the layout with server-side authentication context
 */

import { ReactNode } from 'react';
import { getCurrentUser } from '@/lib/actions/server-auth';
import { AdminLayoutClient } from './AdminLayoutClient';

interface AdminLayoutServerProps {
  children: ReactNode;
}

/**
 * Server component that fetches user data and passes it to the client layout
 */
export async function AdminLayoutServer({ children }: AdminLayoutServerProps) {
  // Fetch user data on the server
  let user = null;
  try {
    user = await getCurrentUser();
  } catch (error) {
    console.error('Failed to get user data in layout:', error);
  }

  // Pass server data to client component
  return (
    <AdminLayoutClient user={user}>
      {children}
    </AdminLayoutClient>
  );
}