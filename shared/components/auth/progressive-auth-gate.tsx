/**
 * UNIFIED PROGRESSIVE AUTH GATE COMPONENT
 * 
 * Consolidates progressive authentication logic for both admin-frontend and frontend apps.
 * Replaces AdminProgressiveAuthGate and frontend progressive auth with a single,
 * platform-aware component that handles multi-level authentication flows.
 * 
 * Features:
 * - Multi-level authentication requirements (Anonymous -> Authenticated -> Progressive -> Full)
 * - Platform-aware messaging and flows  
 * - Web3 wallet integration support
 * - Graceful degradation for missing auth levels
 * - Customizable upgrade prompts and messaging
 */
'use client';

import type { ReactNode } from 'react';
import React from 'react';
import { useSharedAuth } from './Provider';

export type Platform = 'admin' | 'frontend';

export type AuthLevel = 'ANONYMOUS' | 'AUTHENTICATED' | 'PROGRESSIVE' | 'FULL';

export interface UnifiedProgressiveAuthGateProps {
  /**
   * Platform context - DETERMINED AUTOMATICALLY from hook
   */
  platform?: Platform;

  /**
   * Content to show when user meets the required auth level
   */
  children: ReactNode;

  /**
   * Required authentication level
   */
  requiredLevel: AuthLevel;

  /**
   * Action name for better UX messaging
   */
  actionName?: string;

  /**
   * Custom fallback content for insufficient auth level
   */
  fallback?: ReactNode;

  /**
   * Custom message for auth level requirement
   */
  customMessage?: string;

  /**
   * Whether to show upgrade/auth prompts
   */
  showAuthPrompt?: boolean;

  /**
   * Whether to show Web3 wallet connection options
   */
  showWalletOptions?: boolean;
}

// Auth level hierarchy for comparison (Legacy compatibility)
const AUTH_LEVEL_HIERARCHY: Record<AuthLevel, number> = {
  'ANONYMOUS': 0,
  'AUTHENTICATED': 1,
  'PROGRESSIVE': 2,
  'FULL': 3
};

// Get human-readable auth level names
function getAuthLevelDisplayName(level: AuthLevel): string {
  switch (level) {
    case 'ANONYMOUS': return 'No authentication';
    case 'AUTHENTICATED': return 'Basic authentication';
    case 'PROGRESSIVE': return 'Enhanced authentication';
    case 'FULL': return 'Complete authentication';
    default: return level;
  }
}

// Auth level description (deleted unused function)

export default function UnifiedProgressiveAuthGate({
  platform: _platform, // Platform is now inferred from context
  children,
  requiredLevel: _requiredLevel,
  actionName,
  fallback,
  customMessage,
  showAuthPrompt = true,
  showWalletOptions = false
}: UnifiedProgressiveAuthGateProps) {
  const auth = useSharedAuth();

  // PERMISSION REFACTOR: Client-side is permissive for all authenticated users.
  // Backend (Rust) enforces actual resource access and auth level requirements.
  const currentLevel: AuthLevel = auth.isAuthenticated ? 'AUTHENTICATED' : 'ANONYMOUS';
  const requiredLevel: AuthLevel = _requiredLevel;
  const platform: Platform = _platform ?? 'frontend';

  if (auth.isAuthenticated) {
    return <>{children}</>;
  }

  // Show custom fallback if provided
  if (fallback !== null && fallback !== undefined) {
    return <>{fallback}</>;
  }

  // Don't show auth prompt if disabled
  if (!showAuthPrompt) {
    return null;
  }

  // Determine what kind of auth prompt to show based on current vs required level
  const needsSignIn = currentLevel === 'ANONYMOUS';
  const needsProgressive = currentLevel === 'AUTHENTICATED' && requiredLevel !== 'AUTHENTICATED';
  const needsFullAuth = requiredLevel === 'FULL';

  return (
    <div className="flex items-center justify-center min-h-[300px] p-6">
      <div className="max-w-md w-full">
        <div className="p-6 bg-blue-50 border border-blue-200 rounded-lg dark:bg-blue-900/20 dark:border-blue-700">
          <AuthGateUI
            needsSignIn={needsSignIn}
            needsProgressive={needsProgressive}
            needsFullAuth={needsFullAuth}
            customMessage={customMessage}
            actionName={actionName}
            requiredLevel={requiredLevel}
            platform={platform}
            showWalletOptions={showWalletOptions}
          />
        </div>
      </div>
    </div>
  );
}

interface AuthGateUIProps {
  needsSignIn: boolean;
  needsProgressive: boolean;
  needsFullAuth: boolean;
  customMessage?: string;
  actionName?: string;
  requiredLevel: AuthLevel;
  platform: Platform;
  showWalletOptions: boolean;
}

// ... types remain same ...

// Refactored AuthGateUI components to drastically reduce complexity
function AuthIcon({ type }: { type: 'signin' | 'progressive' | 'full' }) {
  if (type === 'signin') {
    return (
      <span className="text-3xl" role="img" aria-label="Sign in required">
        🔐
      </span>
    );
  }
  if (type === 'progressive') {
    return (
      <span className="text-3xl" role="img" aria-label="Enhanced auth required">
        🛡️
      </span>
    );
  }
  return (
    <span className="text-3xl" role="img" aria-label="Full verification required">
      ✅
    </span>
  );
}

function AuthActions({
  needsSignIn,
  needsProgressive,
  needsFullAuth,
  showWalletOptions
}: {
  needsSignIn: boolean;
  needsProgressive: boolean;
  needsFullAuth: boolean;
  showWalletOptions: boolean;
}) {
  return (
    <div className="space-y-3">
      {needsSignIn && (
        <div className="space-y-2">
          <button className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium">
            Sign In
          </button>

          {showWalletOptions && (
            <button className="w-full px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 font-medium">
              Connect Wallet
            </button>
          )}
        </div>
      )}

      {needsProgressive && (
        <div className="space-y-2">
          <button className="w-full px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 font-medium">
            Complete Authentication
          </button>

          {showWalletOptions && (
            <button className="w-full px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 font-medium">
              Verify with Wallet
            </button>
          )}
        </div>
      )}

      {needsFullAuth && (
        <div className="space-y-2">
          <button className="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 font-medium">
            Complete Verification
          </button>
        </div>
      )}

      {/* Secondary action */}
      <button className="w-full px-4 py-2 text-blue-600 hover:text-blue-800 font-medium text-sm">
        Learn More
      </button>
    </div>
  );
}

function AuthTitle({ needsSignIn, needsProgressive, needsFullAuth }: { needsSignIn: boolean; needsProgressive: boolean; needsFullAuth: boolean }) {
  return (
    <h3 className="text-lg font-medium text-blue-900 dark:text-blue-100">
      {needsSignIn && 'Sign In Required'}
      {needsProgressive && 'Enhanced Authentication Required'}
      {needsFullAuth && 'Account Verification Required'}
    </h3>
  );
}

function AuthDescription({ customMessage, actionName, requiredLevel }: { customMessage?: string; actionName?: string; requiredLevel: AuthLevel }) {
  return (
    <p className="text-sm text-blue-700 dark:text-blue-300">
      {typeof customMessage === 'string' && customMessage !== '' ? customMessage : (
        <>
          {typeof actionName === 'string' && actionName !== '' ? `To ${actionName}, you` : 'You'} need {getAuthLevelDisplayName(requiredLevel).toLowerCase()}.
        </>
      )}
    </p>
  );
}

function AuthGateUI({
  needsSignIn,
  needsProgressive,
  needsFullAuth,
  customMessage,
  actionName,
  requiredLevel,
  platform,
  showWalletOptions
}: AuthGateUIProps) {
  return (
    <div className="text-center space-y-4">
      {/* Icon based on required level */}
      <div className="flex justify-center">
        {needsSignIn && <AuthIcon type="signin" />}
        {needsProgressive && <AuthIcon type="progressive" />}
        {needsFullAuth && <AuthIcon type="full" />}
      </div>

      {/* Title */}
      <div>
        <AuthTitle
          needsSignIn={needsSignIn}
          needsProgressive={needsProgressive}
          needsFullAuth={needsFullAuth}
        />
      </div>

      {/* Description */}
      <div>
        <AuthDescription
          customMessage={customMessage}
          actionName={actionName}
          requiredLevel={requiredLevel}
        />
      </div>

      {/* Current vs Required level info (admin only) */}
      {platform === 'admin' && (
        <div className="text-xs text-blue-600 dark:text-blue-400 bg-blue-100 dark:bg-blue-800/30 rounded p-2">
          <div>Required: {getAuthLevelDisplayName(requiredLevel)}</div>
        </div>
      )}

      {/* Action buttons */}
      <AuthActions
        needsSignIn={needsSignIn}
        needsProgressive={needsProgressive}
        needsFullAuth={needsFullAuth}
        showWalletOptions={showWalletOptions}
      />

      {/* Platform-specific help text */}
      <div className="text-xs text-blue-600 dark:text-blue-400">
        {platform === 'admin' && (
          <p>Contact your system administrator if you need assistance.</p>
        )}
        {platform === 'frontend' && (
          <p>Need help? Check our support documentation or contact support.</p>
        )}
      </div>
    </div>
  );
}

// Progressive auth gate convenience components
export function RequireSignIn({ children, ...props }: Omit<UnifiedProgressiveAuthGateProps, 'requiredLevel'>) {
  return <UnifiedProgressiveAuthGate requiredLevel="AUTHENTICATED" {...props}>{children}</UnifiedProgressiveAuthGate>;
}

export function RequireProgressiveAuth({ children, ...props }: Omit<UnifiedProgressiveAuthGateProps, 'requiredLevel'>) {
  return <UnifiedProgressiveAuthGate requiredLevel="PROGRESSIVE" {...props}>{children}</UnifiedProgressiveAuthGate>;
}

export function RequireFullAuth({ children, ...props }: Omit<UnifiedProgressiveAuthGateProps, 'requiredLevel'>) {
  return <UnifiedProgressiveAuthGate requiredLevel="FULL" {...props}>{children}</UnifiedProgressiveAuthGate>;
}

// ============================================================================
// HIGHER-ORDER COMPONENT FOR PROGRESSIVE AUTH
// ============================================================================

export interface WithProgressiveAuthConfig {
  platform: Platform;
  requiredLevel: AuthLevel;
  actionName?: string;
  showWalletOptions?: boolean;
}

export function withProgressiveAuth<P extends object>(
  Component: React.ComponentType<P>,
  config: WithProgressiveAuthConfig
) {
  // Use generic type P directly without casting to any
  const WrappedComponent = React.forwardRef<unknown, P>((props, ref) => {
    return (
      <UnifiedProgressiveAuthGate
        platform={config.platform}
        requiredLevel={config.requiredLevel}
        actionName={config.actionName}
        showWalletOptions={config.showWalletOptions}
      >
        <Component {...props} {...(ref ? { ref } : {}) as P} />
      </UnifiedProgressiveAuthGate>
    );
  });
  WrappedComponent.displayName = `withProgressiveAuth(${Component.displayName ?? Component.name})`;
  return WrappedComponent;
}

// ============================================================================
// PROGRESSIVE AUTH STATUS HOOK
// ============================================================================

export function useProgressiveAuthStatus() {
  const auth = useSharedAuth();
  const currentLevel: AuthLevel = auth.isAuthenticated ? 'AUTHENTICATED' : 'ANONYMOUS';
  const currentLevelValue = AUTH_LEVEL_HIERARCHY[currentLevel];

  return {
    currentLevel,
    currentLevelValue,
    isAnonymous: currentLevel === 'ANONYMOUS',
    isAuthenticated: auth.isAuthenticated,
    isProgressive: auth.isAuthenticated, // Permissive on client
    isFull: auth.isAuthenticated,        // Permissive on client
    canAccess: (_level: AuthLevel) => auth.isAuthenticated,
    needsUpgrade: (_level: AuthLevel) => !auth.isAuthenticated
  };
}