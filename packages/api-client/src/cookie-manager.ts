export const COOKIE_NAMES = {
  SESSION: 'sess_id',
  ADMIN_SESSION: 'admin_sess_id',
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
  adminSession?: string;
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
      const adminSession = cookieStore.get(COOKIE_NAMES.ADMIN_SESSION);
      const session = cookieStore.get(COOKIE_NAMES.SESSION);
      return adminSession?.value || session?.value || null;
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
      console.log('🍪 [CookieManager] Getting auth cookies...');
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      
      // Debug: Log all available cookies
      const allCookies = cookieStore.getAll();
      console.log('🍪 [CookieManager] All cookies available:', 
        allCookies.map((c: { name: string; value: string }) => ({ name: c.name, hasValue: !!c.value, valueLength: c.value?.length || 0 }))
      );
      
      // Get both custom and NextAuth cookies
      const nextAuthCookieNames = [
        'next-auth.session-token',
        '__Secure-next-auth.session-token', 
        'authjs.session-token',
        '__Secure-authjs.session-token'
      ];
      
      const authCookies: Record<string, string | undefined> = {
        session: cookieStore.get(COOKIE_NAMES.SESSION)?.value,
        adminSession: cookieStore.get(COOKIE_NAMES.ADMIN_SESSION)?.value,
        csrf: cookieStore.get(COOKIE_NAMES.CSRF)?.value,
        refresh: cookieStore.get(COOKIE_NAMES.REFRESH)?.value,
      };
      
      // Add NextAuth cookies
      for (const cookieName of nextAuthCookieNames) {
        const cookie = cookieStore.get(cookieName);
        if (cookie?.value) {
          authCookies[cookieName] = cookie.value;
        }
      }
      
      console.log('🔍 [CookieManager] Looking for specific auth cookies:', {
        expectedNames: COOKIE_NAMES,
        found: {
          session: !!authCookies.session,
          adminSession: !!authCookies.adminSession,
          csrf: !!authCookies.csrf,
          refresh: !!authCookies.refresh,
        }
      });
      
      return authCookies;
    } catch (error) {
      console.error('❌ [CookieManager] Failed to get auth cookies:', error);
      return {};
    }
  }

  /**
   * Build authorization headers for the new backend session system
   * Backend expects: Authorization: Bearer <session_id>
   */
  static async buildAuthHeaders(): Promise<Record<string, string>> {
    const headers: Record<string, string> = {};
    
    try {
      // Try multiple sources for session token
      let sessionToken: string | null = null;
      
      // 1. Check if NextAuth session token was passed via environment/context
      if (typeof globalThis !== 'undefined' && (globalThis as any).__NEXTAUTH_SESSION_TOKEN) {
        sessionToken = (globalThis as any).__NEXTAUTH_SESSION_TOKEN;
        console.log('🔐 [CookieManager] Found NextAuth session token from global context');
      }
      
      // 2. Try to get from NextAuth cookies (standard NextAuth cookie names)
      if (!sessionToken) {
        try {
          const { cookies } = await import('next/headers');
          const cookieStore = await cookies();
          
          // NextAuth.js standard cookie names
          const nextAuthCookieNames = [
            'next-auth.session-token',
            '__Secure-next-auth.session-token',
            'authjs.session-token',
            '__Secure-authjs.session-token'
          ];
          
          console.log('🔍 [CookieManager] Checking for NextAuth cookies:', {
            expectedCookieNames: nextAuthCookieNames,
            availableCookies: cookieStore.getAll().map(c => c.name)
          });
          
          for (const cookieName of nextAuthCookieNames) {
            const cookie = cookieStore.get(cookieName);
            if (cookie?.value) {
              sessionToken = cookie.value;
              console.log(`🔐 [CookieManager] Found NextAuth session token from cookie: ${cookieName}`);
              console.log('🔍 [CookieManager] Session token details:', {
                cookieName,
                tokenLength: cookie.value.length,
                tokenStart: cookie.value.substring(0, 50) + '...',
                isJWT: cookie.value.includes('.') && cookie.value.split('.').length === 3
              });
              break;
            }
          }
        } catch (_cookieError) {
          console.debug('🔍 [CookieManager] NextAuth cookies not available');
        }
      }
      
      // 3. Fallback to our custom session cookies
      if (!sessionToken) {
        const allCookies = await this.getAuthCookies();
        sessionToken = allCookies.adminSession || allCookies.session || null;
        if (sessionToken) {
          console.log('🔐 [CookieManager] Found session token from custom cookies');
        }
      }
      
      const csrfToken = await this.getCSRFToken();
      
      console.log('🍪 [CookieManager] Building auth headers for backend:', {
        nextAuthSessionFound: !!sessionToken,
        cookieSessionFound: !sessionToken && !!(await this.getAuthCookies()).session,
        csrfTokenFound: !!csrfToken,
      });
      
      // Add Bearer token authorization if session is available
      if (sessionToken) {
        // Check if this is a NextAuth JWE token that needs to be decrypted
        try {
          // NextAuth uses JWE tokens which look different from JWTs
          // JWE format: header.encrypted_key.initialization_vector.ciphertext.authentication_tag
          if (sessionToken.includes('.') && sessionToken.split('.').length === 5) {
            // This is likely a JWE token from NextAuth
            // Decrypt NextAuth JWE token
            
            try {
              // Import NextAuth's getToken function which can decrypt JWE tokens
              const { getToken } = await import('next-auth/jwt');
              
              // We need a mock request object to use with getToken
              // Since we're in a server context, we can create a minimal request
              const authCookies = await this.getAuthCookies();
              const cookieHeader = Object.entries(authCookies)
                .filter(([_, value]) => value)
                .map(([name, value]) => `${name}=${value}`)
                .join('; ');
                
              const mockReq = {
                headers: {
                  cookie: cookieHeader
                },
                cookies: authCookies
              } as any;
              
              // Attempt token decryption with NextAuth
              
              // Try to decrypt the JWE token using NextAuth
              const decryptedToken = await getToken({ 
                req: mockReq, 
                secret: process.env.NEXTAUTH_SECRET || process.env.AUTH_SECRET,
                secureCookie: process.env.NODE_ENV === 'production'
              });
              
              if (decryptedToken) {
                
                // Look for the backend session ID in various fields
                const possibleSessionId = (decryptedToken.accessToken || 
                                         decryptedToken.sessionId || 
                                         decryptedToken.sub) as string | undefined;
                
                if (possibleSessionId && typeof possibleSessionId === 'string') {
                  headers['Authorization'] = `Bearer ${possibleSessionId}`;
                } else {
                  headers['Authorization'] = `Bearer ${sessionToken}`;
                }
              } else {
                headers['Authorization'] = `Bearer ${sessionToken}`;
              }
            } catch (decryptError: any) {
              console.warn('NextAuth token decryption failed:', decryptError?.message);
              headers['Authorization'] = `Bearer ${sessionToken}`;
            }
          } else {
            // Use token directly (JWT or session ID)
            headers['Authorization'] = `Bearer ${sessionToken}`;
          }
        } catch (importError) {
          console.warn('⚠️ [CookieManager] Failed to import next-auth/jwt, using raw token');
          headers['Authorization'] = `Bearer ${sessionToken}`;
        }
        
        console.log('🔑 [CookieManager] Added Bearer token authorization header');
      } else {
        console.warn('⚠️ [CookieManager] No session token found for authorization');
      }
      
      // Still add CSRF token if available for additional security
      if (csrfToken) {
        headers['X-CSRF-Token'] = csrfToken;
        console.log('🔑 [CookieManager] Added CSRF token to headers');
      }
    } catch (error) {
      console.error('❌ [CookieManager] Failed to build auth headers:', error);
    }

    console.log('📋 [CookieManager] Final auth headers:', headers);
    return headers;
  }

  /**
   * Client-side cookie utilities
   */
  static client = {
    getSessionToken(): string | null {
      if (typeof document === 'undefined') return null;
      
      const cookies = document.cookie.split(';');
      console.log('🍪 [CookieManager.client] All browser cookies:', 
        cookies.map(c => {
          const [key] = c.trim().split('=');
          return key;
        })
      );
      
      for (const cookie of cookies) {
        const [key, value] = cookie.trim().split('=');
        if (key === COOKIE_NAMES.SESSION && value) {
          console.log('🔍 [CookieManager.client] Found session cookie');
          return decodeURIComponent(value);
        }
      }
      
      console.log('⚠️ [CookieManager.client] No session cookie found');
      return null;
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
      
      console.log('🍪 [CookieManager.client] Building client-side auth headers...');
      
      if (typeof document !== 'undefined') {
        const allCookies = this.getAllCookies();
        const relevantCookies = {
          session: allCookies[COOKIE_NAMES.SESSION],
          adminSession: allCookies[COOKIE_NAMES.ADMIN_SESSION],
          csrf: allCookies[COOKIE_NAMES.CSRF],
          refresh: allCookies[COOKIE_NAMES.REFRESH],
        };
        
        // Also check localStorage for session token (new backend format)
        let sessionToken: string | null = relevantCookies.adminSession || relevantCookies.session || null;
        
        // Fallback to localStorage if no cookie session found
        if (!sessionToken && typeof localStorage !== 'undefined') {
          const storedToken = localStorage.getItem('session_id');
          sessionToken = storedToken || null;
        }
        
        console.log('🔍 [CookieManager.client] Client auth tokens found:', {
          cookieNames: COOKIE_NAMES,
          cookiesAvailable: Object.keys(allCookies),
          cookieSessionFound: !!(relevantCookies.adminSession || relevantCookies.session),
          localStorageSessionFound: !!(typeof localStorage !== 'undefined' && localStorage.getItem('session_id')),
          csrfFound: !!relevantCookies.csrf,
          finalSessionToken: !!sessionToken,
        });
        
        // Add Bearer authorization header for new backend
        if (sessionToken && sessionToken !== null) {
          headers['Authorization'] = `Bearer ${sessionToken}`;
          console.log('🔑 [CookieManager.client] Added Bearer authorization header');
        } else {
          console.log('⚠️ [CookieManager.client] No session token found for authorization');
        }
        
        // Add CSRF token for additional security
        if (relevantCookies.csrf) {
          headers['X-CSRF-Token'] = relevantCookies.csrf;
          console.log('🔑 [CookieManager.client] Added CSRF token to headers');
        }
      }
      
      console.log('📋 [CookieManager.client] Final client auth headers:', headers);
      return headers;
    }
  };
}