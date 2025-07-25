export const COOKIE_NAMES = {
  SESSION: '__session',
  CSRF: '__csrf_token',
  REFRESH: '__refresh_token',
} as const;

export const COOKIE_CONFIG = {
  SESSION: {
    maxAge: 60 * 60 * 24 * 5, // 5 days
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    path: '/',
  },
  CSRF: {
    maxAge: 60 * 60 * 24, // 1 day
    httpOnly: false,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict' as const,
    path: '/',
  },
  REFRESH: {
    maxAge: 60 * 60 * 24 * 30, // 30 days
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict' as const,
    path: '/api/v1/auth',
  },
} as const;

export interface AuthCookies {
  session?: string;
  csrf?: string;
  refresh?: string;
}

export class CookieManager {
  /**
   * Get session token from server cookies
   */
  static async getSessionToken(): Promise<string | null> {
    try {
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      const sessionCookie = cookieStore.get(COOKIE_NAMES.SESSION);
      return sessionCookie?.value || null;
    } catch (error) {
      console.error('Failed to get session token:', error);
      return null;
    }
  }

  /**
   * Get CSRF token from server cookies
   */
  static async getCSRFToken(): Promise<string | null> {
    try {
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      const csrfCookie = cookieStore.get(COOKIE_NAMES.CSRF);
      return csrfCookie?.value || null;
    } catch (error) {
      console.error('Failed to get CSRF token:', error);
      return null;
    }
  }

  /**
   * Get all auth cookies for API requests
   */
  static async getAuthCookies(): Promise<AuthCookies> {
    try {
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      return {
        session: cookieStore.get(COOKIE_NAMES.SESSION)?.value,
        csrf: cookieStore.get(COOKIE_NAMES.CSRF)?.value,
        refresh: cookieStore.get(COOKIE_NAMES.REFRESH)?.value,
      };
    } catch (error) {
      console.error('Failed to get auth cookies:', error);
      return {};
    }
  }

  /**
   * Build authorization headers from cookies
   */
  static async buildAuthHeaders(): Promise<Record<string, string>> {
    const headers: Record<string, string> = {};
    
    try {
      const sessionToken = await this.getSessionToken();
      if (sessionToken) {
        headers['Authorization'] = `Bearer ${sessionToken}`;
      }

      const csrfToken = await this.getCSRFToken();
      if (csrfToken) {
        headers['X-CSRF-Token'] = csrfToken;
      }
    } catch (error) {
      console.error('Failed to build auth headers:', error);
    }

    return headers;
  }

  /**
   * Client-side cookie utilities
   */
  static client = {
    getSessionToken(): string | null {
      if (typeof document === 'undefined') return null;
      
      const cookies = document.cookie.split(';');
      for (const cookie of cookies) {
        const [key, value] = cookie.trim().split('=');
        if (key === COOKIE_NAMES.SESSION) {
          return decodeURIComponent(value);
        }
      }
      return null;
    },

    buildAuthHeaders(): Record<string, string> {
      const headers: Record<string, string> = {};
      const sessionToken = this.getSessionToken();
      
      if (sessionToken) {
        headers['Authorization'] = `Bearer ${sessionToken}`;
      }
      
      return headers;
    }
  };
}