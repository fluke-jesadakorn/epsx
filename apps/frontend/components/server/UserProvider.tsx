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
    user_id: (user as any).id || '',
    email: (user as any).email || '',
    role: (user as any).role || 'user',
    permissions: (user as any).permissions ? Object.keys((user as any).permissions) : ['user:read'],
    package_tier: (user as any).package_tier || 'FREE',
    name: (user as any).name || (user as any).email || '',
  } : null;
  
  return <>{children(userData)}</>;
}

export default UserProvider;