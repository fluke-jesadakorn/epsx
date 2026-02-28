// ============================================================================
// SHARED USER DISPLAY COMPONENTS
// Ultra-simple display-only components for both frontend and admin-frontend
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - ZERO permission logic in components
 * - Only display what backend tells us to display
 * - Backend makes ALL authorization decisions
 * - Components are "dumb" - just render data
 */

'use client';

import React from 'react';
import { useSharedAuth } from '../auth/provider';

// Simple user info display component
export function UserWalletDisplay({
  className = '',
  showFullAddress = false,
}: {
  className?: string;
  showFullAddress?: boolean;
}) {
  const { user, isAuthenticated } = useSharedAuth();

  if (!isAuthenticated || !user) {
    return null;
  }

  const formatAddress = (address: string) => {
    if (showFullAddress) { return address; }
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  return (
    <div className={`flex items-center space-x-2 ${className}`}>
      <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
        <svg
          className="w-4 h-4 text-white"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v2a2 2 0 002 2z"
          />
        </svg>
      </div>
      <div className="flex flex-col">
        <span className="text-sm font-medium text-gray-900">
          {formatAddress(user.wallet_address)}
        </span>
      </div>
    </div>
  );
}

// Simple authenticated user status component
export function UserAuthStatus({ className = '' }: { className?: string }) {
  const { isAuthenticated, isLoading, error, user } = useSharedAuth();

  if (isLoading) {
    return (
      <div className={`flex items-center space-x-2 ${className}`}>
        <div className="w-4 h-4 bg-gray-300 rounded-full animate-pulse" />
        <span className="text-sm text-gray-500">Loading...</span>
      </div>
    );
  }

  if (error !== null) {
    return (
      <div className={`flex items-center space-x-2 ${className}`}>
        <div className="w-4 h-4 bg-red-500 rounded-full" />
        <span className="text-sm text-red-600">Auth Error</span>
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className={`flex items-center space-x-2 ${className}`}>
        <div className="w-4 h-4 bg-gray-400 rounded-full" />
        <span className="text-sm text-gray-600">Not Authenticated</span>
      </div>
    );
  }

  return (
    <div className={`flex items-center space-x-2 ${className}`}>
      <div className="w-4 h-4 bg-green-500 rounded-full" />
      <span className="text-sm text-green-600">
        Authenticated via {user?.auth_method ?? 'unknown'}
      </span>
    </div>
  );
}

// Simple permissions list component (FOR DISPLAY ONLY)
export function UserPermissionsDisplay({
  className = '',
  maxDisplay = 5,
}: {
  className?: string;
  maxDisplay?: number;
}) {
  const { user, isAuthenticated } = useSharedAuth();

  if (!isAuthenticated || !user) {
    return null;
  }

  if (user.permissions.length === 0) {
    return null;
  }

  const permissions = user.permissions.slice(0, maxDisplay);
  const hasMore = user.permissions.length > maxDisplay;

  return (
    <div className={`${className}`}>
      <h4 className="text-sm font-medium text-gray-900 mb-2">
        Permissions (Display Only)
      </h4>
      <div className="space-y-1">
        {permissions.map((permission) => (
          <span
            key={permission}
            className="inline-block bg-gray-100 text-gray-700 text-xs px-2 py-1 rounded mr-1 mb-1"
          >
            {permission}
          </span>
        ))}
        {hasMore && (
          <span className="inline-block bg-gray-200 text-gray-600 text-xs px-2 py-1 rounded">
            +{user.permissions.length - maxDisplay} more
          </span>
        )}
      </div>
      <p className="text-xs text-gray-500 mt-2">
        Note: Authorization is handled by backend only
      </p>
    </div>
  );
}

// Simple logout button component
export function UserLogoutButton({
  className = '',
  onLogout,
}: {
  className?: string;
  onLogout?: () => void;
}) {
  const { logout, isLoading } = useSharedAuth();

  const handleLogout = async () => {
    try {
      await logout();
      onLogout?.();
    } catch (_error) {
      // console.error('Logout failed:', _error);
    }
  };

  return (
    <button
      onClick={() => { void handleLogout(); }}
      disabled={isLoading}
      className={`inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50 ${className}`}
    >
      {isLoading ? (
        <>
          <svg
            className="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          Logging out...
        </>
      ) : (
        <>
          <svg
            className="-ml-1 mr-2 h-4 w-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
            />
          </svg>
          Logout
        </>
      )}
    </button>
  );
}

// Conditional display wrapper (display helper only - NOT for authorization)
export function ConditionalDisplay({
  condition,
  children,
  fallback,
}: {
  condition: boolean;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}) {
  // WARNING: This is for display purposes only
  // Backend makes all authorization decisions

  if (condition) {
    return <>{children}</>;
  }

  return fallback !== undefined && fallback !== null ? <>{fallback}</> : null;
}

// Loading state component
export function LoadingDisplay({
  className = '',
  message = 'Loading...',
}: {
  className?: string;
  message?: string;
}) {
  return (
    <div className={`flex items-center justify-center p-4 ${className}`}>
      <div className="flex items-center space-x-2">
        <svg
          className="animate-spin h-5 w-5 text-gray-500"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
        <span className="text-sm text-gray-600">{message}</span>
      </div>
    </div>
  );
}

// User tier badge component
export function UserTierBadge({ className = '' }: { className?: string }) {
  const { user, isAuthenticated } = useSharedAuth();

  if (!isAuthenticated || !user) {
    return null;
  }

  // Simple tier display based on permissions or user data
  const getTierInfo = () => {
    // You can customize this logic based on your user tier system
    if (user.permissions.includes('admin:*:*')) {
      return { tier: 'Admin', color: 'bg-purple-100 text-purple-800' };
    }
    if (user.permissions.includes('epsx:premium:access')) {
      return { tier: 'Premium', color: 'bg-yellow-100 text-yellow-800' };
    }
    return { tier: 'Basic', color: 'bg-gray-100 text-gray-800' };
  };

  const { tier, color } = getTierInfo();

  return (
    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${color} ${className}`}>
      {tier}
    </span>
  );
}

// Error display component
export function ErrorDisplay({
  error,
  className = '',
  onRetry,
}: {
  error: string;
  className?: string;
  onRetry?: () => void;
}) {
  return (
    <div
      className={`bg-red-50 border border-red-200 rounded-lg p-4 ${className}`}
    >
      <div className="flex">
        <div className="flex-shrink-0">
          <svg
            className="h-5 w-5 text-red-400"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
              clipRule="evenodd"
            />
          </svg>
        </div>
        <div className="ml-3 flex-1">
          <h3 className="text-sm font-medium text-red-800">Error</h3>
          <p className="text-sm text-red-700 mt-1">{error}</p>
          {onRetry && (
            <button
              onClick={onRetry}
              className="mt-2 text-sm bg-red-100 hover:bg-red-200 text-red-800 px-3 py-1 rounded"
            >
              Try Again
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
