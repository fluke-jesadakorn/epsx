import { auth } from '@/lib/auth';
import { ReactNode } from 'react';

interface UserProviderProps {
  children: (user: UserData | null) => ReactNode;
}

interface UserData {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  package_tier: string;
  admin_modules: string[];
  name: string;
}

/**
 * Server component that provides user data to child components
 * Uses NextAuth.js session for server-side user data access
 */
export async function UserProvider({ children }: UserProviderProps) {
  const session = await auth();
  
  const user: UserData | null = session?.user ? {
    user_id: session.user.firebase_uid || session.user.id || '',
    email: session.user.email || '',
    role: session.user.role || 'user',
    permissions: session.user.permissions || ['user:read'],
    package_tier: session.user.package_tier || 'FREE',
    admin_modules: session.user.admin_modules || [],
    name: session.user.name || session.user.email || '',
  } : null;
  
  return <>{children(user)}</>;
}

export default UserProvider;