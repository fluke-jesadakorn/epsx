import { cookies } from 'next/headers';
import { type User } from '@/shared/types/auth';
import { getBackendUrl } from '@/shared/utils/url-resolver';
import { COOKIES } from '@/shared/auth/cookies';

export interface AdminSessionData {
  isAuthenticated: boolean;
  user?: User;
  expiresAt?: number;
}

/**
 * Get server session for admin app
 */
export async function getServerSessionAdmin(): Promise<AdminSessionData | null> {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get(COOKIES.access)?.value;

    if (!accessToken) {
      return { isAuthenticated: false };
    }

    // Validate session with backend
    const response = await fetch(`${getBackendUrl()}/api/auth/session`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
        'X-App-Type': 'admin',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      return { isAuthenticated: false };
    }

    const sessionData = await response.json();

    // Check if user has admin permissions
    const hasAdminPermissions = sessionData.user?.permissions?.some((p: string) =>
      p.startsWith('admin:')
    ) || false;

    if (!hasAdminPermissions) {
      return { isAuthenticated: false };
    }

    return {
      isAuthenticated: true,
      user: sessionData.user,
      expiresAt: sessionData.expiresAt,
    };

  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to get admin server session:', _error);
    return { isAuthenticated: false };
  }
}