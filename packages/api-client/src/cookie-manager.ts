export const COOKIE_NAMES = {
  SESSION: 'auth-token',
  ADMIN_SESSION: 'admin-auth-token',
  ID_TOKEN: 'id-token',
  REFRESH: 'refresh-token',
} as const;

export const COOKIE_CONFIG = {
  SESSION: {
    maxAge: 60 * 60 * 24 * 5, // 5 days
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
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
  adminSession?: string;
  idToken?: string;
  refresh?: string;
}

export class CookieManager {
  /**
   * Get Bearer token from server cookies (simplified OIDC pattern)
   */
  static async getBearerToken(): Promise<string | null> {
    try {
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      return cookieStore.get(COOKIE_NAMES.ADMIN_SESSION)?.value || 
             cookieStore.get(COOKIE_NAMES.SESSION)?.value || 
             null;
    } catch (error) {
      console.error('Failed to get bearer token:', error);
      return null;
    }
  }

  /**
   * @deprecated Use getBearerToken() instead
   */
  static async getSessionToken(): Promise<string | null> {
    return this.getBearerToken();
  }


  /**
   * Get all auth cookies for API requests
   */
  static async getAuthCookies(): Promise<AuthCookies> {
    try {
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      
      // Get OIDC authentication cookies
      const authCookies: Record<string, string | undefined> = {
        session: cookieStore.get(COOKIE_NAMES.SESSION)?.value,
        adminSession: cookieStore.get(COOKIE_NAMES.ADMIN_SESSION)?.value,
        idToken: cookieStore.get(COOKIE_NAMES.ID_TOKEN)?.value,
        refresh: cookieStore.get(COOKIE_NAMES.REFRESH)?.value,
      };
      
      
      return authCookies;
    } catch (error) {
      console.error('❌ [CookieManager] Failed to get auth cookies:', error);
      return {};
    }
  }

  /**
   * Build authorization headers (clean OIDC pattern)
   * Backend expects: Authorization: Bearer <access_token>
   */
  static async buildAuthHeaders(): Promise<Record<string, string>> {
    const headers: Record<string, string> = {};
    
    try {
      const bearerToken = await this.getBearerToken();
      
      if (bearerToken) {
        headers['Authorization'] = `Bearer ${bearerToken}`;
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
    getBearerToken(): string | null {
      if (typeof document === 'undefined') return null;
      
      const cookies = document.cookie.split(';');
      
      // Check for both admin and regular session tokens
      for (const cookie of cookies) {
        const [key, value] = cookie.trim().split('=');
        if ((key === COOKIE_NAMES.ADMIN_SESSION || key === COOKIE_NAMES.SESSION) && value) {
          return decodeURIComponent(value);
        }
      }
      return null;
    },

    /**
     * @deprecated Use getBearerToken() instead
     */
    getSessionToken(): string | null {
      return this.getBearerToken();
    },

    getAllCookies(): Record<string, string> {
      if (typeof document === 'undefined') return {};
      
      const cookies: Record<string, string> = {};
      document.cookie.split(';').forEach(cookie => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) {
          cookies[key] = decodeURIComponent(value);
        }
      });
      
      return cookies;
    },

    buildAuthHeaders(): Record<string, string> {
      const headers: Record<string, string> = {};
      
      const bearerToken = this.getBearerToken();
      if (bearerToken) {
        headers['Authorization'] = `Bearer ${bearerToken}`;
      }
      
      return headers;
    }
  };
}