import { cookies } from 'next/headers';
import { type User } from '@/shared/types/auth';
import { getBackendUrl } from '@/shared/utils/url-resolver';
import { COOKIES } from '@/shared/auth/cookies';

export interface SessionData {
  isAuthenticated: boolean;
  user?: User;
  expiresAt?: number;
}

/**
 * Get server session for frontend app
 */
export async function getServerSession(): Promise<SessionData | null> {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get(COOKIES.access_token)?.value;

    if (accessToken == null || accessToken === '') {
      return { isAuthenticated: false };
    }

    // Validate session with backend
    const response = await fetch(`${getBackendUrl()}/api/auth/session`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      return { isAuthenticated: false };
    }

    const sessionData = await response.json() as { user?: User; expiresAt?: number };

    return {
      isAuthenticated: true,
      user: sessionData.user,
      expiresAt: sessionData.expiresAt,
    };

  } catch (_error) {
    return { isAuthenticated: false };
  }
}