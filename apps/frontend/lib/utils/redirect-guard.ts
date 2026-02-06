/**
 * Redirect Loop Detection and Prevention Utility
 * ROOT CAUSE #6 FIX - Prevents infinite redirect loops
 */

interface RedirectAttempt {
  path: string;
  timestamp: number;
  count: number;
}

class RedirectGuard {
  private attempts: Map<string, RedirectAttempt> = new Map();
  private readonly MAX_ATTEMPTS = 3;
  private readonly TIME_WINDOW = 5000; // 5 seconds

  /**
   * Check if redirect is safe (not in a loop)
   */
  isSafeToRedirect(path: string): boolean {
    const now = Date.now();
    const attempt = this.attempts.get(path);

    // No previous attempt - safe
    if (!attempt) {
      this.recordAttempt(path, now);
      return true;
    }

    // Outside time window - reset and allow
    if (now - attempt.timestamp > this.TIME_WINDOW) {
      this.recordAttempt(path, now);
      return true;
    }

    // Within time window - check count
    if (attempt.count >= this.MAX_ATTEMPTS) {
      console.error('🚨 REDIRECT LOOP DETECTED', {
        path,
        attempts: attempt.count,
        timeWindow: `${this.TIME_WINDOW}ms`,
        suggestion: 'Breaking redirect loop to prevent infinite loop'
      });
      return false;
    }

    // Increment attempt count
    this.recordAttempt(path, now, attempt.count + 1);
    return true;
  }

  /**
   * Record a redirect attempt
   */
  private recordAttempt(path: string, timestamp: number, count: number = 1) {
    this.attempts.set(path, { path, timestamp, count });
  }

  /**
   * Clear all attempts (call after successful navigation)
   */
  clear() {
    this.attempts.clear();
  }

  /**
   * Clear attempts for specific path
   */
  clearPath(path: string) {
    this.attempts.delete(path);
  }

  /**
   * Get current attempts (for debugging)
   */
  getAttempts() {
    return Array.from(this.attempts.entries());
  }
}

// Singleton instance
export const redirectGuard = new RedirectGuard();

/**
 * Safe redirect wrapper with loop detection
 */
export function safeRedirect(
  router: { push: (path: string) => void; replace: (path: string) => void },
  path: string,
  options: {
    replace?: boolean;
    delay?: number;
    onLoop?: () => void;
  } = {}
): boolean {
  const { replace = false, delay = 0, onLoop } = options;

  // Check if safe to redirect
  if (!redirectGuard.isSafeToRedirect(path)) {
    if (onLoop) {
      onLoop();
    }
    return false;
  }

  // Perform redirect
  const redirectFn = replace ? router.replace : router.push;

  if (delay > 0) {
    setTimeout(() => {
      redirectFn(path);
    }, delay);
  } else {
    redirectFn(path);
  }

  return true;
}

/**
 * Clear redirect guard on successful navigation
 * Call this in your layout or app component
 */
export function useRedirectGuardCleanup() {
  if (typeof window !== 'undefined') {
    // Clear on route change
    const handleRouteChange = () => {
      redirectGuard.clear();
    };

    window.addEventListener('popstate', handleRouteChange);
    return () => {
      window.removeEventListener('popstate', handleRouteChange);
    };
  }
}

/**
 * Validate returnUrl to prevent auth page loops
 */
const AUTH_PAGES = ['/auth', '/auth/signin', '/auth/login'];

export function validateReturnUrl(returnUrl: string, fallback: string = '/dashboard'): string {
  // Prevent auth pages from being returnUrl
  if (AUTH_PAGES.some(page => returnUrl.startsWith(page))) {
    console.warn('⚠️ Prevented auth page loop', {
      attemptedReturnUrl: returnUrl,
      fallback
    });
    return fallback;
  }

  // Ensure it's a valid path (starts with /)
  if (!returnUrl.startsWith('/')) {
    console.warn('⚠️ Invalid returnUrl (must start with /)', {
      attemptedReturnUrl: returnUrl,
      fallback
    });
    return fallback;
  }

  return returnUrl;
}
