/**
 * Progressive Auth Gate Component
 * 
 * FRONTEND PERMISSION PHILOSOPHY:
 * - Frontend does NOT enforce permissions
 * - Backend handles all authorization via API responses
 * - Frontend only displays errors from backend (401/403 with reason)
 * - This component is a passthrough - it just renders children
 * 
 * For triggering sign-in, use JustInTimeAuth or check backend API responses
 */
'use client';

// Simple passthrough - no frontend permission checking
// Backend handles all authorization, frontend just shows the UI
export function ProgressiveAuthGate({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

// All these are passthroughs - no frontend enforcement
export function RequireSignIn({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

export function RequireProgressiveAuth({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

export function RequireFullAuth({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

export function PermissionGuard({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

export function RequirePermission({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

export function RequireRole({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

export function RequireTier({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

export function RequireAccess({ children }: { children: React.ReactNode;[key: string]: any }) {
  return <>{children}</>;
}

// Passthrough hooks - return empty/no-op values
export function useProgressiveAuthStatus(_platform?: string) {
  return {
    currentLevel: 'ANONYMOUS',
    currentLevelValue: 0,
    isAnonymous: true,
    isAuthenticated: false,
    isProgressive: false,
    isFull: false,
    canAccess: () => true, // Allow UI rendering, backend enforces
    needsUpgrade: () => false
  };
}

export function useUnifiedPermissionGuard(_platform?: string, _permissions?: string[]) {
  return {
    hasPermission: true, // Allow UI rendering, backend enforces
    userPermissions: [],
    canAccess: () => true
  };
}

// HOC passthrough
export function withProgressiveAuth<P extends object>(Component: React.ComponentType<P>) {
  return Component;
}

export type Platform = 'admin' | 'frontend';
export type UnifiedProgressiveAuthGateProps = { children: React.ReactNode };
