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

import { ReactNode } from 'react';
import { getAuthHook, type AuthLevel } from './UnifiedAuthAdapter';

export type Platform = 'admin' | 'frontend';

export interface UnifiedProgressiveAuthGateProps {
  /**
   * Platform context - determines which auth system to use
   */
  platform: Platform;
  
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

// Auth level hierarchy for comparison
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

// Get auth level description
function getAuthLevelDescription(level: AuthLevel): string {
  switch (level) {
    case 'AUTHENTICATED': return 'Please sign in to continue';
    case 'PROGRESSIVE': return 'Please complete additional authentication steps';
    case 'FULL': return 'Please complete full account verification';
    default: return 'Authentication required';
  }
}

export default function UnifiedProgressiveAuthGate({
  platform,
  children,
  requiredLevel,
  actionName,
  fallback,
  customMessage,
  showAuthPrompt = true,
  showWalletOptions = false
}: UnifiedProgressiveAuthGateProps) {
  const auth = getAuthHook(platform);
  
  // Check if user meets the required auth level
  const currentLevel = auth.level || 'ANONYMOUS';
  const currentLevelValue = AUTH_LEVEL_HIERARCHY[currentLevel] || 0;
  const requiredLevelValue = AUTH_LEVEL_HIERARCHY[requiredLevel] || 0;
  
  // If user meets or exceeds required level, show content
  if (currentLevelValue >= requiredLevelValue) {
    return <>{children}</>;
  }
  
  // Show custom fallback if provided
  if (fallback) {
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
          <div className="text-center space-y-4">
            {/* Icon based on required level */}
            <div className="flex justify-center">
              {needsSignIn && (
                <span className="text-3xl" role="img" aria-label="Sign in required">
                  🔐
                </span>
              )}
              {needsProgressive && (
                <span className="text-3xl" role="img" aria-label="Enhanced auth required">
                  🛡️
                </span>
              )}
              {needsFullAuth && (
                <span className="text-3xl" role="img" aria-label="Full verification required">
                  ✅
                </span>
              )}
            </div>
            
            {/* Title */}
            <div>
              <h3 className="text-lg font-medium text-blue-900 dark:text-blue-100">
                {needsSignIn && 'Sign In Required'}
                {needsProgressive && 'Enhanced Authentication Required'}
                {needsFullAuth && 'Account Verification Required'}
              </h3>
            </div>
            
            {/* Description */}
            <div>
              <p className="text-sm text-blue-700 dark:text-blue-300">
                {customMessage || (
                  <>
                    {actionName ? `To ${actionName}, you` : 'You'} need {getAuthLevelDisplayName(requiredLevel).toLowerCase()}.
                  </>
                )}
              </p>
            </div>
            
            {/* Current vs Required level info (admin only) */}
            {platform === 'admin' && (
              <div className="text-xs text-blue-600 dark:text-blue-400 bg-blue-100 dark:bg-blue-800/30 rounded p-2">
                <div>Current: {getAuthLevelDisplayName(currentLevel)}</div>
                <div>Required: {getAuthLevelDisplayName(requiredLevel)}</div>
              </div>
            )}
            
            {/* Action buttons */}
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
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// CONVENIENCE COMPONENTS FOR SPECIFIC AUTH LEVELS
// ============================================================================

export function RequireSignIn({ 
  platform,
  children, 
  fallback,
  actionName,
  showWalletOptions = false
}: {
  platform: Platform;
  children: ReactNode;
  fallback?: ReactNode;
  actionName?: string;
  showWalletOptions?: boolean;
}) {
  return (
    <UnifiedProgressiveAuthGate
      platform={platform}
      requiredLevel="AUTHENTICATED"
      actionName={actionName}
      fallback={fallback}
      showWalletOptions={showWalletOptions}
    >
      {children}
    </UnifiedProgressiveAuthGate>
  );
}

export function RequireProgressiveAuth({ 
  platform,
  children, 
  fallback,
  actionName,
  showWalletOptions = true
}: {
  platform: Platform;
  children: ReactNode;
  fallback?: ReactNode;
  actionName?: string;
  showWalletOptions?: boolean;
}) {
  return (
    <UnifiedProgressiveAuthGate
      platform={platform}
      requiredLevel="PROGRESSIVE"
      actionName={actionName}
      fallback={fallback}
      showWalletOptions={showWalletOptions}
    >
      {children}
    </UnifiedProgressiveAuthGate>
  );
}

export function RequireFullAuth({ 
  platform,
  children, 
  fallback,
  actionName
}: {
  platform: Platform;
  children: ReactNode;
  fallback?: ReactNode;
  actionName?: string;
}) {
  return (
    <UnifiedProgressiveAuthGate
      platform={platform}
      requiredLevel="FULL"
      actionName={actionName}
      fallback={fallback}
    >
      {children}
    </UnifiedProgressiveAuthGate>
  );
}

// ============================================================================
// HIGHER-ORDER COMPONENT FOR PROGRESSIVE AUTH
// ============================================================================

export function withProgressiveAuth<P extends object>(
  Component: React.ComponentType<P>,
  platform: Platform,
  requiredLevel: AuthLevel,
  options?: {
    actionName?: string;
    showWalletOptions?: boolean;
  }
) {
  return function ProtectedComponent(props: P) {
    return (
      <UnifiedProgressiveAuthGate
        platform={platform}
        requiredLevel={requiredLevel}
        actionName={options?.actionName}
        showWalletOptions={options?.showWalletOptions}
      >
        <Component {...props} />
      </UnifiedProgressiveAuthGate>
    );
  };
}

// ============================================================================
// PROGRESSIVE AUTH STATUS HOOK
// ============================================================================

export function useProgressiveAuthStatus(platform: Platform) {
  const auth = getAuthHook(platform);
  const currentLevel = auth.level || 'ANONYMOUS';
  const currentLevelValue = AUTH_LEVEL_HIERARCHY[currentLevel];
  
  return {
    currentLevel,
    currentLevelValue,
    isAnonymous: currentLevel === 'ANONYMOUS',
    isAuthenticated: currentLevelValue >= AUTH_LEVEL_HIERARCHY.AUTHENTICATED,
    isProgressive: currentLevelValue >= AUTH_LEVEL_HIERARCHY.PROGRESSIVE,
    isFull: currentLevelValue >= AUTH_LEVEL_HIERARCHY.FULL,
    canAccess: (level: AuthLevel) => currentLevelValue >= AUTH_LEVEL_HIERARCHY[level],
    needsUpgrade: (level: AuthLevel) => currentLevelValue < AUTH_LEVEL_HIERARCHY[level]
  };
}