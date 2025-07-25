import { cookies } from 'next/headers';
import { AppState } from './types';

// Server-side state hydration utilities
export interface SSRStateOptions {
  includeAuth?: boolean;
  includeUserPreferences?: boolean;
  includeCache?: boolean;
  cacheKeys?: string[];
}

// Extract server-side state for hydration
export async function getServerState(options: SSRStateOptions = {}): Promise<Partial<AppState>> {
  const {
    includeAuth = true,
    includeUserPreferences = true,
    includeCache = false,
    cacheKeys = []
  } = options;

  const serverState: Partial<AppState> = {
    ui: {
      theme: 'system',
      sidebar: {
        open: true,
        collapsed: false
      },
      modals: {},
      toasts: [],
      loading: {
        global: false,
        requests: {}
      },
      responsive: {
        isMobile: false,
        isTablet: false,
        breakpoint: 'lg'
      }
    }
  };

  try {
    // Get auth state from server
    if (includeAuth) {
      const authState = await getServerAuthState();
      if (authState) {
        serverState.user = {
          data: authState,
          loading: false,
          error: null,
          lastUpdated: Date.now(),
          optimisticUpdates: []
        };
      }
    }

    // Get user preferences from server or cookies
    if (includeUserPreferences) {
      const preferences = await getServerUserPreferences();
      if (preferences && serverState.user?.data) {
        serverState.user.data.preferences = preferences;
      }
    }

    // Get cached data if requested
    if (includeCache && cacheKeys.length > 0) {
      const cacheData = await getServerCacheData(cacheKeys);
      if (cacheData) {
        serverState.cache = cacheData;
      }
    }

  } catch (error) {
    console.error('Error getting server state:', error);
    // Return empty state on error to prevent SSR failures
  }

  return serverState;
}

// Get authentication state from server
async function getServerAuthState() {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('session');
    
    if (!sessionCookie) {
      return null;
    }

    // Verify session with backend
    const response = await fetch(`${process.env.API_URL || process.env.BACKEND_URL || 'http://localhost:8080'}/api/v1/auth/profile`, {
      headers: {
        'Cookie': `sess_id=${sessionCookie.value}`,
        'Content-Type': 'application/json'
      },
      cache: 'no-store'
    });

    if (!response.ok) {
      return null;
    }

    const userData = await response.json();
    return {
      profile: {
        id: userData.user_id,
        email: userData.email,
        displayName: userData.display_name || userData.email?.split('@')[0],
        photoURL: userData.photo_url,
        emailVerified: true,
        createdAt: userData.created_at,
        lastSignInAt: userData.last_sign_in,
        role: userData.role
      },
      permissions: userData.permissions || [],
      packageTier: userData.package_tier || 'FREE',
      subscription: userData.subscription ? {
        tier: userData.subscription.tier,
        validUntil: userData.subscription.valid_until,
        isActive: userData.subscription.is_active,
        features: userData.subscription.features || []
      } : null,
      preferences: {
        language: 'en',
        timezone: 'UTC',
        currency: 'USD',
        notifications: {
          email: true,
          push: true,
          tradingAlerts: true,
          priceAlerts: true
        },
        trading: {
          defaultView: 'grid' as const,
          riskTolerance: 'medium' as const,
          autoRefresh: true,
          refreshInterval: 30000
        }
      }
    };

  } catch (error) {
    console.error('Error getting server auth state:', error);
    return null;
  }
}

// Get user preferences from cookies or server
async function getServerUserPreferences() {
  try {
    const cookieStore = await cookies();
    
    // Try to get preferences from cookie first
    const prefsCookie = cookieStore.get('user-preferences');
    if (prefsCookie) {
      return JSON.parse(prefsCookie.value);
    }

    // If authenticated, get from server
    const sessionCookie = cookieStore.get('session');
    if (sessionCookie) {
      const response = await fetch(`${process.env.API_URL || process.env.BACKEND_URL || 'http://localhost:8080'}/api/user/preferences`, {
        headers: {
          'Cookie': `sess_id=${sessionCookie.value}`,
          'Content-Type': 'application/json'
        },
        cache: 'no-store'
      });

      if (response.ok) {
        return await response.json();
      }
    }

    // Return default preferences
    return {
      language: 'en',
      timezone: 'UTC',
      currency: 'USD',
      notifications: {
        email: true,
        push: true,
        tradingAlerts: true,
        priceAlerts: true
      },
      trading: {
        defaultView: 'grid',
        riskTolerance: 'medium',
        autoRefresh: true,
        refreshInterval: 30000
      }
    };

  } catch (error) {
    console.error('Error getting server user preferences:', error);
    return null;
  }
}

// Get cached data from server
async function getServerCacheData(keys: string[]) {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('session');
    
    if (!sessionCookie) {
      return null;
    }

    const response = await fetch(`${process.env.API_URL || process.env.BACKEND_URL || 'http://localhost:8080'}/api/cache/bulk`, {
      method: 'POST',
      headers: {
        'Cookie': `sess_id=${sessionCookie.value}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ keys }),
      cache: 'no-store'
    });

    if (!response.ok) {
      return null;
    }

    const cacheData = await response.json();
    return {
      stockData: cacheData.stockData || {},
      rankings: cacheData.rankings || {},
      analytics: cacheData.analytics || {}
    };

  } catch (error) {
    console.error('Error getting server cache data:', error);
    return null;
  }
}

// Hydrate client state with server state
export function hydrateClientState(serverState: Partial<AppState>, clientState: AppState): AppState {
  return {
    ui: {
      ...clientState.ui,
      ...serverState.ui,
      // Preserve client-only UI state
      responsive: clientState.ui.responsive,
      toasts: clientState.ui.toasts,
      loading: clientState.ui.loading
    },
    user: {
      ...clientState.user,
      ...serverState.user
    },
    trading: {
      ...clientState.trading,
      ...serverState.trading
    },
    notifications: {
      ...clientState.notifications,
      ...serverState.notifications
    },
    cache: {
      ...clientState.cache,
      ...serverState.cache
    }
  };
}

// SSR-safe hook for initial hydration
export function useSSRSafeHydration<T>(serverValue: T, clientValue: T, deps: any[] = []): T {
  const [isHydrated, setIsHydrated] = React.useState(false);
  
  React.useEffect(() => {
    setIsHydrated(true);
  }, []);

  return isHydrated ? clientValue : serverValue;
}

// Server-side theme detection
export async function getServerTheme(): Promise<'light' | 'dark' | 'system'> {
  try {
    const cookieStore = await cookies();
    const themeCookie = cookieStore.get('theme');
    
    if (themeCookie && ['light', 'dark', 'system'].includes(themeCookie.value)) {
      return themeCookie.value as 'light' | 'dark' | 'system';
    }
    
    return 'system';
  } catch {
    return 'system';
  }
}

// Server-side user agent detection for responsive state
export function getServerResponsive(userAgent?: string) {
  if (!userAgent) {
    return {
      isMobile: false,
      isTablet: false,
      breakpoint: 'lg' as const
    };
  }

  const isMobile = /iPhone|iPad|iPod|Android|BlackBerry|Opera Mini|IEMobile|WPDesktop/i.test(userAgent);
  const isTablet = /iPad|Android(?!.*Mobile)|Kindle|Silk/i.test(userAgent);

  return {
    isMobile: isMobile && !isTablet,
    isTablet,
    breakpoint: (isMobile && !isTablet) ? 'sm' : isTablet ? 'md' : 'lg' as const
  };
}

// Generate script tag for hydrating client state
export function generateHydrationScript(serverState: Partial<AppState>): string {
  const safeState = JSON.stringify(serverState).replace(/</g, '\\u003c');
  
  return `
    <script>
      window.__EPSX_INITIAL_STATE__ = ${safeState};
    </script>
  `;
}

// Client-side hydration from script tag
export function getHydrationState(): Partial<AppState> | null {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    const initialState = (window as any).__EPSX_INITIAL_STATE__;
    if (initialState) {
      // Clean up
      delete (window as any).__EPSX_INITIAL_STATE__;
      return initialState;
    }
  } catch (error) {
    console.error('Error parsing hydration state:', error);
  }

  return null;
}

// React import for useSSRSafeHydration
import React from 'react';