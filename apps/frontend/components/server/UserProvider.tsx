import { getCurrentUser } from '@/lib/server-actions';
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
  name: string;
}

/**
 * Server component that provides user data to child components
 * Uses getCurrentUser for server-side user data access
 */
export async function UserProvider({ children }: UserProviderProps) {
  const user = await getCurrentUser();

  const userData: UserData | null = user ? {
    user_id: user.id || '',
    email: user.email || '',
    role: user.role || 'user',
    permissions: Array.isArray(user.permissions)
      ? user.permissions
      : (user.permissions ? Object.keys(user.permissions) : ['user:read']),
    package_tier: user.package_tier || 'FREE',
    name: user.name || user.email || '',
  } : null;

  return <>{children(userData)}</>;
}

export default UserProvider;