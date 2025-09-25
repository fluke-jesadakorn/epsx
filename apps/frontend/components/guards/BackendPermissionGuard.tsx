// ============================================================================
// BACKEND PERMISSION GUARD COMPONENT (Phase 2.1)
// Replaces ALL local permission validation with backend API calls
// THE SINGLE SOURCE OF TRUTH for conditional rendering based on permissions
// ============================================================================

'use client';

import React, { useEffect, useState } from 'react';
import { usePermission, usePermissions } from '@/lib/permissions/use-backend-permissions';
import { PermissionError } from '@/lib/permissions/use-backend-permissions';

// ============================================================================
// PERMISSION GUARD COMPONENT TYPES
// ============================================================================

export interface BackendPermissionGuardProps {
  // Required permission(s)
  permission?: string;
  permissions?: string[];
  requireAll?: boolean; // For multiple permissions: require all vs any
  
  // User context
  userId?: string;
  resourcePath?: string;
  
  // Rendering options
  children: React.ReactNode;
  fallback?: React.ReactNode;
  loadingFallback?: React.ReactNode;
  errorFallback?: React.ReactNode | ((error: PermissionError) => React.ReactNode);
  
  // Behavior options
  hideOnDenied?: boolean; // Hide content completely vs show fallback
  showErrorDetails?: boolean; // Show detailed error information
  enableUpgradePrompt?: boolean; // Show upgrade prompts for tier restrictions
  
  // Callbacks
  onPermissionGranted?: () => void;
  onPermissionDenied?: (error: PermissionError) => void;
  onPermissionError?: (error: PermissionError) => void;
}

export interface UpgradePromptProps {
  upgradeInfo: {
    current_tier: string;
    required_tier: string;
    upgrade_url?: string;
    benefits: string[];
  };
  permission: string;
  onUpgrade?: () => void;
  onDismiss?: () => void;
}

// ============================================================================
// BACKEND PERMISSION GUARD COMPONENT
// THE SINGLE SOURCE OF TRUTH for permission-based conditional rendering
// ============================================================================

export function BackendPermissionGuard({
  permission,
  permissions = [],
  requireAll = false,
  userId,
  resourcePath,
  children,
  fallback,
  loadingFallback,
  errorFallback,
  hideOnDenied = false,
  showErrorDetails = false,
  enableUpgradePrompt = true,
  onPermissionGranted,
  onPermissionDenied,
  onPermissionError,
}: BackendPermissionGuardProps) {
  
  // ============================================================================
  // PERMISSION VALIDATION LOGIC
  // ============================================================================
  
  // Single permission validation
  const singlePermissionResult = usePermission(
    permission || '',
    userId,
    resourcePath
  );
  
  // Multiple permissions validation
  const multiplePermissionsResult = usePermissions(
    permissions,
    userId,
    requireAll
  );
  
  // Determine which result to use
  const permissionResult = permission 
    ? singlePermissionResult 
    : multiplePermissionsResult;
  
  const { granted, loading, error, requiresUpgrade, upgradeInfo } = permissionResult;

  // ============================================================================
  // EFFECT HANDLERS
  // ============================================================================
  
  useEffect(() => {
    if (loading) return;
    
    if (granted) {
      onPermissionGranted?.();
    } else if (error) {
      onPermissionDenied?.(error);
      onPermissionError?.(error);
    }
  }, [granted, loading, error, onPermissionGranted, onPermissionDenied, onPermissionError]);

  // ============================================================================
  // LOADING STATE
  // ============================================================================
  
  if (loading) {
    if (loadingFallback) {
      return <>{loadingFallback}</>;
    }
    
    return (
      <div className="flex items-center justify-center p-4">
        <div className="animate-pulse flex space-x-2">
          <div className="h-2 w-2 bg-gray-400 rounded-full animate-bounce" />
          <div className="h-2 w-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0.1s' }} />
          <div className="h-2 w-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }} />
        </div>
        <span className="ml-2 text-sm text-gray-500">Validating permissions...</span>
      </div>
    );
  }

  // ============================================================================
  // PERMISSION GRANTED - RENDER CHILDREN
  // ============================================================================
  
  if (granted) {
    return <>{children}</>;
  }

  // ============================================================================
  // PERMISSION DENIED - HANDLE ERRORS AND FALLBACKS
  // ============================================================================
  
  // Handle specific error types with custom fallbacks
  if (error) {
    // If we have a custom error fallback function, use it
    if (typeof errorFallback === 'function') {
      return <>{errorFallback(error)}</>;
    }
    
    // If we have a custom error fallback component, use it
    if (errorFallback) {
      return <>{errorFallback}</>;
    }
    
    // Show upgrade prompt for tier restrictions
    if (enableUpgradePrompt && requiresUpgrade && upgradeInfo) {
      return (
        <UpgradePrompt
          upgradeInfo={upgradeInfo}
          permission={permission || permissions.join(', ')}
          onUpgrade={() => {
            if (upgradeInfo.upgrade_url) {
              window.location.href = upgradeInfo.upgrade_url;
            }
          }}
        />
      );
    }
    
    // Show detailed error information if enabled
    if (showErrorDetails) {
      return <ErrorDetails error={error} />;
    }
  }

  // ============================================================================
  // DEFAULT FALLBACK HANDLING
  // ============================================================================
  
  // Hide content completely if hideOnDenied is true
  if (hideOnDenied) {
    return null;
  }
  
  // Show custom fallback if provided
  if (fallback) {
    return <>{fallback}</>;
  }
  
  // Default permission denied message
  return <DefaultPermissionDenied permission={permission || permissions.join(', ')} />;
}

// ============================================================================
// UPGRADE PROMPT COMPONENT
// ============================================================================

function UpgradePrompt({ 
  upgradeInfo, 
  permission, 
  onUpgrade, 
  onDismiss 
}: UpgradePromptProps) {
  return (
    <div className="bg-gradient-to-br from-blue-50 to-indigo-100 border border-blue-200 rounded-lg p-6">
      <div className="flex items-center mb-4">
        <div className="flex-shrink-0">
          <svg className="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
              d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        <div className="ml-3">
          <h3 className="text-lg font-medium text-gray-900">
            Upgrade Required
          </h3>
          <p className="text-sm text-gray-600">
            This feature requires {upgradeInfo.required_tier} tier access
          </p>
        </div>
      </div>
      
      <div className="mb-4">
        <p className="text-sm text-gray-700 mb-2">
          You currently have <span className="font-semibold">{upgradeInfo.current_tier}</span> tier access.
          Upgrade to <span className="font-semibold">{upgradeInfo.required_tier}</span> to unlock:
        </p>
        
        <ul className="list-disc list-inside space-y-1 text-sm text-gray-600">
          {upgradeInfo.benefits.map((benefit, index) => (
            <li key={index}>{benefit}</li>
          ))}
        </ul>
      </div>
      
      <div className="flex space-x-3">
        <button
          onClick={onUpgrade}
          className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors"
        >
          Upgrade Now
        </button>
        
        {onDismiss && (
          <button
            onClick={onDismiss}
            className="bg-gray-200 hover:bg-gray-300 text-gray-800 px-4 py-2 rounded-md text-sm font-medium transition-colors"
          >
            Maybe Later
          </button>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// ERROR DETAILS COMPONENT
// ============================================================================

function ErrorDetails({ error }: { error: PermissionError }) {
  return (
    <div className="bg-red-50 border border-red-200 rounded-lg p-4">
      <div className="flex">
        <div className="flex-shrink-0">
          <svg className="h-5 w-5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <div className="ml-3">
          <h3 className="text-sm font-medium text-red-800">
            Permission Error
          </h3>
          <div className="mt-2 text-sm text-red-700">
            <p className="mb-2">{error.userMessage}</p>
            
            {error.suggestedActions.length > 0 && (
              <div>
                <p className="font-medium mb-1">Suggested actions:</p>
                <ul className="list-disc list-inside space-y-1">
                  {error.suggestedActions.map((action, index) => (
                    <li key={index}>{action}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// DEFAULT PERMISSION DENIED COMPONENT
// ============================================================================

function DefaultPermissionDenied({ permission }: { permission: string }) {
  return (
    <div className="bg-gray-50 border border-gray-200 rounded-lg p-6 text-center">
      <div className="flex justify-center mb-4">
        <svg className="h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} 
            d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>
      
      <h3 className="text-lg font-medium text-gray-900 mb-2">
        Access Restricted
      </h3>
      
      <p className="text-sm text-gray-600 mb-4">
        You don't have the necessary permissions to view this content.
      </p>
      
      {permission && (
        <p className="text-xs text-gray-500">
          Required permission: <code className="bg-gray-100 px-1 py-0.5 rounded">{permission}</code>
        </p>
      )}
    </div>
  );
}

// ============================================================================
// CONVENIENCE PERMISSION GUARD VARIANTS
// ============================================================================

// Admin-only guard
export function AdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendPermissionGuard
      permission="admin:general:access"
      userId={userId}
      {...props}
    >
      {children}
    </BackendPermissionGuard>
  );
}

// Feature guard for specific features
export function FeatureGuard({ 
  feature, 
  children, 
  userId, 
  ...props 
}: Omit<BackendPermissionGuardProps, 'permission'> & { 
  feature: string;
  userId?: string;
}) {
  return (
    <BackendPermissionGuard
      permission={`epsx:${feature}:access`}
      userId={userId}
      {...props}
    >
      {children}
    </BackendPermissionGuard>
  );
}

// Analytics guard
export function AnalyticsGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendPermissionGuard
      permission="epsx:analytics:read"
      userId={userId}
      {...props}
    >
      {children}
    </BackendPermissionGuard>
  );
}

// Premium features guard
export function PremiumGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendPermissionGuard
      permission="epsx:premium:access"
      userId={userId}
      enableUpgradePrompt={true}
      {...props}
    >
      {children}
    </BackendPermissionGuard>
  );
}

// User management guard
export function UserManagementGuard({ 
  action, 
  children, 
  userId, 
  ...props 
}: Omit<BackendPermissionGuardProps, 'permission'> & { 
  action: 'read' | 'create' | 'update' | 'delete';
  userId?: string;
}) {
  return (
    <BackendPermissionGuard
      permission={`admin:users:${action}`}
      userId={userId}
      {...props}
    >
      {children}
    </BackendPermissionGuard>
  );
}

// ============================================================================
// HIGHER-ORDER COMPONENT WRAPPER
// ============================================================================

export function withBackendPermission<T extends object>(
  Component: React.ComponentType<T>,
  permission: string,
  options?: Partial<BackendPermissionGuardProps>
) {
  return function PermissionWrappedComponent(props: T & { userId?: string }) {
    const { userId, ...componentProps } = props;
    
    return (
      <BackendPermissionGuard
        permission={permission}
        userId={userId}
        {...options}
      >
        <Component {...componentProps as T} />
      </BackendPermissionGuard>
    );
  };
}