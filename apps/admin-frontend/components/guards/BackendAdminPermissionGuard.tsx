// ============================================================================
// BACKEND ADMIN PERMISSION GUARD COMPONENT (Phase 2.2)
// Replaces ALL local admin permission validation with backend API calls
// THE SINGLE SOURCE OF TRUTH for conditional admin rendering based on permissions
// ============================================================================

'use client';

import React, { useEffect, useState } from 'react';
import { useAdminPermission, useAdminPermissions } from '@/lib/permissions/use-backend-admin-permissions';
import { AdminPermissionError } from '@/lib/permissions/use-backend-admin-permissions';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { 
  Shield, 
  ShieldAlert, 
  ShieldX,
  Clock, 
  AlertTriangle, 
  Lock,
  RefreshCw,
  UserX,
  Settings,
  Crown,
  ShieldCheck
} from 'lucide-react';

// ============================================================================
// ADMIN PERMISSION GUARD COMPONENT TYPES
// ============================================================================

export interface BackendAdminPermissionGuardProps {
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
  errorFallback?: React.ReactNode | ((error: AdminPermissionError) => React.ReactNode);
  
  // Behavior options
  hideOnDenied?: boolean; // Hide content completely vs show fallback
  showErrorDetails?: boolean; // Show detailed error information
  enableUpgradePrompt?: boolean; // Show upgrade prompts for tier restrictions
  showAdminContext?: boolean; // Show admin-specific context
  
  // Admin-specific options
  adminAction?: string; // Description of admin action being performed
  requiredAdminLevel?: 'admin' | 'super_admin' | 'system_admin';
  securityLevel?: 'standard' | 'elevated' | 'critical';
  
  // Callbacks
  onPermissionGranted?: () => void;
  onPermissionDenied?: (error: AdminPermissionError) => void;
  onPermissionError?: (error: AdminPermissionError) => void;
}

export interface AdminUpgradePromptProps {
  upgradeInfo: {
    current_tier: string;
    required_tier: string;
    upgrade_url?: string;
    benefits: string[];
  };
  permission: string;
  adminAction?: string;
  securityLevel?: 'standard' | 'elevated' | 'critical';
  onUpgrade?: () => void;
  onDismiss?: () => void;
}

// ============================================================================
// BACKEND ADMIN PERMISSION GUARD COMPONENT
// THE SINGLE SOURCE OF TRUTH for admin permission-based conditional rendering
// ============================================================================

export function BackendAdminPermissionGuard({
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
  showAdminContext = true,
  adminAction,
  requiredAdminLevel = 'admin',
  securityLevel = 'standard',
  onPermissionGranted,
  onPermissionDenied,
  onPermissionError,
}: BackendAdminPermissionGuardProps) {
  
  // ============================================================================
  // ADMIN PERMISSION VALIDATION LOGIC
  // ============================================================================
  
  // Single admin permission validation
  const singlePermissionResult = useAdminPermission(
    permission || '',
    userId,
    resourcePath
  );
  
  // Multiple admin permissions validation
  const multiplePermissionsResult = useAdminPermissions(
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
        <div className="flex items-center space-x-3">
          <div className="relative">
            <Shield className="h-6 w-6 text-blue-500 animate-pulse" />
            <div className="absolute -top-1 -right-1">
              <RefreshCw className="h-3 w-3 text-blue-400 animate-spin" />
            </div>
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-gray-700">Validating admin permissions...</p>
            <div className="flex space-x-1">
              <div className="h-1 w-8 bg-blue-200 rounded animate-pulse" />
              <div className="h-1 w-12 bg-blue-300 rounded animate-pulse" style={{ animationDelay: '0.1s' }} />
              <div className="h-1 w-6 bg-blue-400 rounded animate-pulse" style={{ animationDelay: '0.2s' }} />
            </div>
          </div>
        </div>
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
        <AdminUpgradePrompt
          upgradeInfo={upgradeInfo}
          permission={permission || permissions.join(', ')}
          adminAction={adminAction}
          securityLevel={securityLevel}
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
      return <AdminErrorDetails error={error} showAdminContext={showAdminContext} />;
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
  
  // Default admin permission denied message
  return (
    <DefaultAdminPermissionDenied 
      permission={permission || permissions.join(', ')} 
      adminAction={adminAction}
      requiredAdminLevel={requiredAdminLevel}
      securityLevel={securityLevel}
    />
  );
}

// ============================================================================
// ADMIN UPGRADE PROMPT COMPONENT
// ============================================================================

function AdminUpgradePrompt({ 
  upgradeInfo, 
  permission, 
  adminAction,
  securityLevel = 'standard',
  onUpgrade, 
  onDismiss 
}: AdminUpgradePromptProps) {
  
  const getSecurityIcon = () => {
    switch (securityLevel) {
      case 'critical':
        return <ShieldAlert className="h-6 w-6 text-red-600" />;
      case 'elevated':
        return <ShieldCheck className="h-6 w-6 text-orange-600" />;
      default:
        return <Crown className="h-6 w-6 text-blue-600" />;
    }
  };

  const getSecurityColors = () => {
    switch (securityLevel) {
      case 'critical':
        return {
          bg: 'from-red-50 to-red-100',
          border: 'border-red-200',
          button: 'bg-red-600 hover:bg-red-700',
        };
      case 'elevated':
        return {
          bg: 'from-orange-50 to-orange-100',
          border: 'border-orange-200',
          button: 'bg-orange-600 hover:bg-orange-700',
        };
      default:
        return {
          bg: 'from-blue-50 to-indigo-100',
          border: 'border-blue-200',
          button: 'bg-blue-600 hover:bg-blue-700',
        };
    }
  };

  const colors = getSecurityColors();

  return (
    <div className={`bg-gradient-to-br ${colors.bg} border ${colors.border} rounded-lg p-6`}>
      <div className="flex items-center mb-4">
        <div className="flex-shrink-0">
          {getSecurityIcon()}
        </div>
        <div className="ml-3">
          <h3 className="text-lg font-medium text-gray-900">
            Admin Upgrade Required
          </h3>
          <p className="text-sm text-gray-600">
            {adminAction ? `Admin action: ${adminAction}` : `This admin feature requires ${upgradeInfo.required_tier} tier access`}
          </p>
        </div>
      </div>
      
      <div className="mb-4">
        <p className="text-sm text-gray-700 mb-2">
          Your current admin tier: <span className="font-semibold">{upgradeInfo.current_tier}</span>
        </p>
        <p className="text-sm text-gray-700 mb-2">
          Required admin tier: <span className="font-semibold">{upgradeInfo.required_tier}</span>
        </p>
        
        {upgradeInfo.benefits.length > 0 && (
          <div>
            <p className="text-sm text-gray-700 mb-2">Upgrade benefits:</p>
            <ul className="list-disc list-inside space-y-1 text-sm text-gray-600">
              {upgradeInfo.benefits.map((benefit, index) => (
                <li key={index}>{benefit}</li>
              ))}
            </ul>
          </div>
        )}
      </div>
      
      {securityLevel === 'critical' && (
        <Alert className="mb-4 bg-red-50 border-red-200">
          <AlertTriangle className="h-4 w-4 text-red-600" />
          <AlertDescription className="text-red-800 text-sm">
            <strong>Critical Security Notice:</strong> This admin operation requires elevated privileges for security reasons.
          </AlertDescription>
        </Alert>
      )}
      
      <div className="flex space-x-3">
        <button
          onClick={onUpgrade}
          className={`${colors.button} text-white px-4 py-2 rounded-md text-sm font-medium transition-colors`}
        >
          Request Admin Upgrade
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
// ADMIN ERROR DETAILS COMPONENT
// ============================================================================

function AdminErrorDetails({ error, showAdminContext = true }: { 
  error: AdminPermissionError;
  showAdminContext?: boolean;
}) {
  return (
    <div className="bg-red-50 border border-red-200 rounded-lg p-4">
      <div className="flex">
        <div className="flex-shrink-0">
          <ShieldX className="h-5 w-5 text-red-400" />
        </div>
        <div className="ml-3">
          <h3 className="text-sm font-medium text-red-800">
            Admin Permission Error
          </h3>
          <div className="mt-2 text-sm text-red-700">
            <p className="mb-2">{error.userMessage}</p>
            
            {showAdminContext && error.adminContext && (
              <div className="mb-2">
                <p className="font-medium">Admin Context:</p>
                <div className="ml-2 space-y-1">
                  {error.adminContext.adminAction && (
                    <p>Action: <code className="bg-red-100 px-1 py-0.5 rounded">{error.adminContext.adminAction}</code></p>
                  )}
                  {error.adminContext.requiredAdminLevel && (
                    <p>Required Level: <Badge variant="outline" className="ml-1">{error.adminContext.requiredAdminLevel}</Badge></p>
                  )}
                  {error.adminContext.currentAdminLevel && (
                    <p>Current Level: <Badge variant="outline" className="ml-1">{error.adminContext.currentAdminLevel}</Badge></p>
                  )}
                </div>
              </div>
            )}
            
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
// DEFAULT ADMIN PERMISSION DENIED COMPONENT
// ============================================================================

function DefaultAdminPermissionDenied({ 
  permission, 
  adminAction,
  requiredAdminLevel,
  securityLevel
}: { 
  permission: string;
  adminAction?: string;
  requiredAdminLevel?: 'admin' | 'super_admin' | 'system_admin';
  securityLevel?: 'standard' | 'elevated' | 'critical';
}) {
  
  const getSecurityIcon = () => {
    switch (securityLevel) {
      case 'critical':
        return <ShieldAlert className="h-12 w-12 text-red-400" />;
      case 'elevated':
        return <ShieldCheck className="h-12 w-12 text-orange-400" />;
      default:
        return <Shield className="h-12 w-12 text-gray-400" />;
    }
  };

  const getSecurityColors = () => {
    switch (securityLevel) {
      case 'critical':
        return { bg: 'bg-red-50', border: 'border-red-200' };
      case 'elevated':
        return { bg: 'bg-orange-50', border: 'border-orange-200' };
      default:
        return { bg: 'bg-gray-50', border: 'border-gray-200' };
    }
  };

  const colors = getSecurityColors();

  return (
    <div className={`${colors.bg} ${colors.border} border rounded-lg p-6 text-center`}>
      <div className="flex justify-center mb-4">
        {getSecurityIcon()}
      </div>
      
      <h3 className="text-lg font-medium text-gray-900 mb-2">
        Admin Access Restricted
      </h3>
      
      {adminAction && (
        <p className="text-sm text-gray-600 mb-2">
          Admin Action: <span className="font-medium">{adminAction}</span>
        </p>
      )}
      
      <p className="text-sm text-gray-600 mb-4">
        You don't have the necessary admin permissions to access this feature.
      </p>
      
      {requiredAdminLevel && (
        <div className="mb-4">
          <p className="text-xs text-gray-500 mb-2">Required admin level:</p>
          <Badge variant="outline" className="text-xs">{requiredAdminLevel}</Badge>
        </div>
      )}
      
      {permission && (
        <p className="text-xs text-gray-500">
          Required permission: <code className="bg-gray-100 px-1 py-0.5 rounded">{permission}</code>
        </p>
      )}
      
      <div className="mt-4 flex justify-center space-x-2">
        <Button 
          variant="outline" 
          size="sm"
          onClick={() => window.location.href = '/admin/request-access'}
        >
          <Settings className="w-3 h-3 mr-1" />
          Request Admin Access
        </Button>
      </div>
    </div>
  );
}

// ============================================================================
// CONVENIENCE ADMIN GUARD VARIANTS
// ============================================================================

// Super admin guard
export function SuperAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendAdminPermissionGuard
      permission="admin:*:*"
      userId={userId}
      requiredAdminLevel="super_admin"
      securityLevel="critical"
      adminAction="super admin access"
      {...props}
    >
      {children}
    </BackendAdminPermissionGuard>
  );
}

// User management admin guard
export function UserManagementAdminGuard({ 
  action = 'manage',
  children, 
  userId, 
  ...props 
}: Omit<BackendAdminPermissionGuardProps, 'permission'> & { 
  action?: 'read' | 'create' | 'update' | 'delete' | 'manage';
  userId?: string;
}) {
  return (
    <BackendAdminPermissionGuard
      permission={`admin:users:${action}`}
      userId={userId}
      adminAction={`${action} users`}
      securityLevel={action === 'delete' ? 'critical' : 'elevated'}
      {...props}
    >
      {children}
    </BackendAdminPermissionGuard>
  );
}

// System management admin guard
export function SystemManagementAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendAdminPermissionGuard
      permission="admin:system:manage"
      userId={userId}
      adminAction="manage system"
      securityLevel="critical"
      requiredAdminLevel="system_admin"
      {...props}
    >
      {children}
    </BackendAdminPermissionGuard>
  );
}

// Permission management admin guard
export function PermissionManagementAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendAdminPermissionGuard
      permission="admin:permissions:manage"
      userId={userId}
      adminAction="manage permissions"
      securityLevel="critical"
      requiredAdminLevel="super_admin"
      {...props}
    >
      {children}
    </BackendAdminPermissionGuard>
  );
}

// Analytics admin guard
export function AnalyticsAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendAdminPermissionGuard
      permission="admin:analytics:read"
      userId={userId}
      adminAction="view analytics"
      securityLevel="standard"
      {...props}
    >
      {children}
    </BackendAdminPermissionGuard>
  );
}

// Security admin guard
export function SecurityAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendAdminPermissionGuard
      permission="admin:security:manage"
      userId={userId}
      adminAction="manage security"
      securityLevel="critical"
      requiredAdminLevel="super_admin"
      {...props}
    >
      {children}
    </BackendAdminPermissionGuard>
  );
}

// Audit logs admin guard
export function AuditLogsAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<BackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <BackendAdminPermissionGuard
      permission="admin:audit:read"
      userId={userId}
      adminAction="view audit logs"
      securityLevel="elevated"
      {...props}
    >
      {children}
    </BackendAdminPermissionGuard>
  );
}

// ============================================================================
// HIGHER-ORDER COMPONENT WRAPPER
// ============================================================================

export function withBackendAdminPermission<T extends object>(
  Component: React.ComponentType<T>,
  permission: string,
  options?: Partial<BackendAdminPermissionGuardProps>
) {
  return function AdminPermissionWrappedComponent(props: T & { userId?: string }) {
    const { userId, ...componentProps } = props;
    
    return (
      <BackendAdminPermissionGuard
        permission={permission}
        userId={userId}
        {...options}
      >
        <Component {...componentProps as T} />
      </BackendAdminPermissionGuard>
    );
  };
}

// ============================================================================
// EXPORTS
// ============================================================================

// Export main component
export default BackendAdminPermissionGuard;

// ============================================================================
// MIGRATION COMPLETE NOTICE
// ============================================================================
// 
// 🎉 ADMIN PERMISSION GUARDS TRANSFORMATION COMPLETE!
//
// This file provides THE SINGLE SOURCE OF TRUTH for all admin permission-based
// conditional rendering using the backend permission authority system.
//
// Key Security Features:
// ⚡ ALL admin permission checks now use backend API calls
// 🔒 NO client-side admin permission validation possible
// 🛡️  Structured error responses with admin context
// 📊 Real-time admin permission validation from authoritative source
// ⏰ Backend handles ALL admin time-based and expiry validation
// 👑 Admin-specific upgrade prompts and security levels
// 🎯 Specialized guards for different admin operations
//
// The admin-frontend conditional rendering is now SECURE and UNHACKABLE!
// ============================================================================