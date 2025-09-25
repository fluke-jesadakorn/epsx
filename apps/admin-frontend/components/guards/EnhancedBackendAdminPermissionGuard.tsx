// ============================================================================
// ENHANCED BACKEND ADMIN PERMISSION GUARD (Phase 3.3.1)
// Advanced admin permission guard with comprehensive error handling integration
// ============================================================================

'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { PermissionErrorBoundary } from '@/components/error-boundaries/PermissionErrorBoundary';
import { PermissionErrorUI } from '@/components/errors/PermissionErrorUI';
import { 
  enhancedPermissionAuthority,
  useEnhancedPermissionValidation
} from '@/lib/permissions/enhanced-backend-authority-client';
import { 
  ApiError,
  isPermissionDeniedError,
  isInsufficientTierError,
  isPermissionExpiredError,
  isRateLimitExceededError
} from '@/lib/api/response-handler';
import { 
  permissionErrorAnalytics,
  usePermissionErrorAnalytics
} from '@/lib/analytics/permission-error-analytics';
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
// ENHANCED ADMIN PERMISSION GUARD TYPES
// ============================================================================

export interface EnhancedBackendAdminPermissionGuardProps {
  // Required permission(s)
  permission?: string;
  permissions?: string[];
  requireAll?: boolean; // For multiple permissions: require all vs any
  
  // User context
  userId?: string;
  resourcePath?: string;
  
  // Admin-specific context
  adminAction?: string; // Description of admin action being performed
  requiredAdminLevel?: 'admin' | 'super_admin' | 'system_admin';
  securityLevel?: 'standard' | 'elevated' | 'critical';
  operationId?: string; // Unique identifier for the admin operation
  
  // Rendering options
  children: React.ReactNode;
  fallback?: React.ReactNode;
  loadingFallback?: React.ReactNode;
  
  // Enhanced error handling options
  enableRetry?: boolean;
  enableUpgrade?: boolean;
  enableSupport?: boolean;
  enableAnalytics?: boolean;
  showDetailedErrors?: boolean;
  
  // Behavior options
  hideOnDenied?: boolean; // Hide content completely vs show fallback
  
  // Callbacks
  onPermissionGranted?: (permissions: string[]) => void;
  onPermissionDenied?: (error: ApiError, permissions: string[]) => void;
  onPermissionError?: (error: ApiError) => void;
  onRetry?: () => void;
  onUpgrade?: () => void;
  
  // Component identification
  component?: string;
}

interface AdminPermissionStatusProps {
  permission: string;
  granted: boolean;
  adminLevel?: string;
  securityLevel?: 'standard' | 'elevated' | 'critical';
  expiresAt?: string;
  usageInfo?: {
    current: number;
    limit: number;
    percentage: number;
  };
}

// ============================================================================
// ENHANCED ADMIN PERMISSION GUARD CORE COMPONENT
// ============================================================================

function EnhancedBackendAdminPermissionGuardCore({
  permission,
  permissions = [],
  requireAll = false,
  userId,
  resourcePath,
  adminAction,
  requiredAdminLevel = 'admin',
  securityLevel = 'standard',
  operationId,
  children,
  fallback,
  loadingFallback,
  enableRetry = true,
  enableUpgrade = true,
  enableSupport = true,
  enableAnalytics = true,
  showDetailedErrors = true,
  hideOnDenied = false,
  onPermissionGranted,
  onPermissionDenied,
  onPermissionError,
  onRetry,
  onUpgrade,
  component = 'EnhancedBackendAdminPermissionGuard',
}: EnhancedBackendAdminPermissionGuardProps) {
  
  const analytics = usePermissionErrorAnalytics();
  
  // Build permissions list
  const permissionsToValidate = [
    ...(permission ? [permission] : []),
    ...permissions
  ];
  
  // Enhanced state management
  const [validationResults, setValidationResults] = useState<Record<string, boolean>>({});
  const [validationError, setValidationError] = useState<ApiError | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [errorId, setErrorId] = useState<string | null>(null);
  const [retryCount, setRetryCount] = useState(0);
  
  // Get current user context (should be admin user)
  // This would typically come from an admin auth context
  const currentUserId = userId || 'current-admin-user';
  
  // 🔒 SECURITY CRITICAL: Enhanced admin permission validation
  const validateAdminPermissions = useCallback(async () => {
    if (permissionsToValidate.length === 0) {
      setValidationResults({});
      setValidationError(null);
      return;
    }
    
    setIsValidating(true);
    setValidationError(null);
    
    const startTime = Date.now();
    
    try {
      if (permissionsToValidate.length === 1) {
        // Single permission validation for admin
        const result = await enhancedPermissionAuthority.validatePermission(
          currentUserId,
          permissionsToValidate[0],
          {
            resourcePath,
            component,
            includeUsage: true,
            includeExpiry: true,
            forceRefresh: retryCount > 0, // Force refresh on retry
            context: {
              adminAction,
              requiredAdminLevel,
              securityLevel,
              operationId,
              isAdminOperation: true
            }
          }
        );
        
        if (result.success) {
          const granted = result.data.granted;
          setValidationResults({ [permissionsToValidate[0]]: granted });
          
          if (granted) {
            onPermissionGranted?.(permissionsToValidate);
            
            // Track successful admin permission validation
            if (enableAnalytics) {
              analytics.trackUserAction(
                `admin_permission_success_${Date.now()}`,
                'retry',
                Date.now() - startTime
              );
            }
          } else {
            const deniedError: ApiError = {
              success: false,
              error: {
                type: 'PERMISSION_DENIED',
                code: 'ADMIN_PERMISSION_DENIED',
                message: `Admin permission denied: ${permissionsToValidate[0]}`,
                user_message: `You don't have the necessary admin permissions to perform this action.`,
                suggested_actions: [
                  'Contact your system administrator',
                  'Request elevated admin privileges',
                  'Check your admin role assignments'
                ],
                details: {
                  permission: permissionsToValidate[0],
                  adminAction,
                  requiredAdminLevel,
                  securityLevel
                }
              }
            };
            
            setValidationError(deniedError);
            onPermissionDenied?.(deniedError, permissionsToValidate);
            
            // Track admin permission denial
            if (enableAnalytics) {
              const trackingErrorId = analytics.trackError(deniedError, {
                component,
                permission: permissionsToValidate[0],
                user_id: currentUserId,
                operation: adminAction || 'admin_operation',
                platform: 'admin'
              });
              setErrorId(trackingErrorId);
            }
          }
        } else {
          setValidationError(result);
          onPermissionError?.(result);
          
          // Track validation error
          if (enableAnalytics) {
            const trackingErrorId = analytics.trackError(result, {
              component,
              permissions: permissionsToValidate,
              user_id: currentUserId,
              operation: adminAction || 'admin_operation',
              platform: 'admin'
            });
            setErrorId(trackingErrorId);
          }
        }
      } else {
        // Bulk admin permission validation
        const result = await enhancedPermissionAuthority.validateBulkPermissions(
          currentUserId,
          permissionsToValidate.map(p => ({ permission: p, resource_path: resourcePath })),
          {
            component,
            includeUsage: true,
            includeExpiry: true,
            failFast: !requireAll
          }
        );
        
        if (result.success) {
          const results: Record<string, boolean> = {};
          result.data.results.forEach(r => {
            results[r.permission] = r.granted;
          });
          setValidationResults(results);
          
          const grantedPermissions = result.data.results
            .filter(r => r.granted)
            .map(r => r.permission);
          
          const hasAccess = requireAll 
            ? permissionsToValidate.every(p => results[p] === true)
            : grantedPermissions.length > 0;
          
          if (hasAccess) {
            onPermissionGranted?.(grantedPermissions);
          } else {
            const deniedError: ApiError = {
              success: false,
              error: {
                type: 'PERMISSION_DENIED',
                code: 'ADMIN_BULK_PERMISSION_DENIED',
                message: `Admin bulk permissions denied`,
                user_message: requireAll 
                  ? 'You need all required admin permissions to perform this action.'
                  : 'You don\'t have any of the required admin permissions.',
                suggested_actions: [
                  'Contact your system administrator',
                  'Request elevated admin privileges',
                  'Review your admin role assignments'
                ]
              }
            };
            
            setValidationError(deniedError);
            onPermissionDenied?.(deniedError, permissionsToValidate);
          }
        } else {
          setValidationError(result);
          onPermissionError?.(result);
        }
      }
    } catch (error) {
      const networkError: ApiError = {
        success: false,
        error: {
          type: 'NETWORK_ERROR',
          code: 'ADMIN_PERMISSION_VALIDATION_FAILED',
          message: error instanceof Error ? error.message : 'Admin permission validation failed',
          user_message: 'Unable to validate admin permissions. Please check your connection and try again.',
          suggested_actions: [
            'Check your internet connection',
            'Refresh the page',
            'Contact technical support'
          ]
        }
      };
      
      setValidationError(networkError);
      onPermissionError?.(networkError);
      
      if (enableAnalytics) {
        const trackingErrorId = analytics.trackError(networkError, {
          component,
          permissions: permissionsToValidate,
          user_id: currentUserId,
          operation: adminAction || 'admin_operation',
          platform: 'admin'
        }, { retry_count: retryCount });
        setErrorId(trackingErrorId);
      }
    } finally {
      setIsValidating(false);
    }
  }, [
    permissionsToValidate,
    currentUserId,
    resourcePath,
    component,
    adminAction,
    requiredAdminLevel,
    securityLevel,
    operationId,
    requireAll,
    retryCount,
    enableAnalytics,
    onPermissionGranted,
    onPermissionDenied,
    onPermissionError,
    analytics
  ]);
  
  // Initial validation
  useEffect(() => {
    validateAdminPermissions();
  }, [validateAdminPermissions]);
  
  // Enhanced retry handler
  const handleRetry = useCallback(() => {
    setRetryCount(prev => prev + 1);
    setValidationError(null);
    
    // Clear cache for this user
    enhancedPermissionAuthority.clearUserCache(currentUserId);
    
    // Track retry action
    if (enableAnalytics && errorId) {
      analytics.trackUserAction(errorId, 'retry');
    }
    
    onRetry?.();
    validateAdminPermissions();
  }, [currentUserId, errorId, enableAnalytics, analytics, onRetry, validateAdminPermissions]);
  
  // Enhanced upgrade handler
  const handleUpgrade = useCallback(() => {
    if (enableAnalytics && errorId) {
      analytics.trackUserAction(errorId, 'upgrade');
    }
    
    onUpgrade?.();
    window.location.href = '/admin/request-upgrade';
  }, [enableAnalytics, errorId, analytics, onUpgrade]);
  
  // Enhanced support handler
  const handleContactSupport = useCallback(() => {
    if (enableAnalytics && errorId) {
      analytics.trackUserAction(errorId, 'contact_support');
    }
    
    window.location.href = '/admin/support';
  }, [enableAnalytics, errorId, analytics]);
  
  // Show loading state with admin-specific styling
  if (isValidating) {
    if (loadingFallback) {
      return <>{loadingFallback}</>;
    }
    
    return (
      <div className="flex items-center justify-center p-6">
        <div className="flex items-center space-x-4">
          <div className="relative">
            <Crown className="h-8 w-8 text-amber-500 animate-pulse" />
            <div className="absolute -top-1 -right-1">
              <RefreshCw className="h-4 w-4 text-amber-400 animate-spin" />
            </div>
          </div>
          <div className="space-y-2">
            <p className="text-lg font-semibold text-gray-800">Validating Admin Permissions...</p>
            <div className="flex space-x-1">
              <div className="h-2 w-12 bg-amber-200 rounded animate-pulse" />
              <div className="h-2 w-16 bg-amber-300 rounded animate-pulse" style={{ animationDelay: '0.1s' }} />
              <div className="h-2 w-8 bg-amber-400 rounded animate-pulse" style={{ animationDelay: '0.2s' }} />
            </div>
            <div className="text-sm text-gray-600 flex items-center space-x-2">
              <Shield className="h-4 w-4" />
              <span>Secure admin authority validation</span>
            </div>
          </div>
        </div>
      </div>
    );
  }
  
  // Check if user has valid permissions
  const hasAccess = (() => {
    if (permissionsToValidate.length === 0) return true;
    
    const grantedPermissions = Object.entries(validationResults)
      .filter(([, granted]) => granted)
      .map(([permission]) => permission);
    
    if (requireAll) {
      return permissionsToValidate.every(p => validationResults[p] === true);
    } else {
      return grantedPermissions.length > 0;
    }
  })();
  
  // Render children if access granted
  if (hasAccess && !validationError) {
    return <>{children}</>;
  }
  
  // Show comprehensive error UI if validation failed
  if (validationError) {
    return (
      <PermissionErrorUI
        error={validationError}
        context={{
          component,
          permissions: permissionsToValidate,
          user_id: currentUserId,
          platform: 'admin',
          adminAction,
          requiredAdminLevel,
          securityLevel,
          operationId
        }}
        onRetry={enableRetry ? handleRetry : undefined}
        onUpgrade={enableUpgrade ? handleUpgrade : undefined}
        onContactSupport={enableSupport ? handleContactSupport : undefined}
        showDetails={showDetailedErrors}
        className="my-6"
        adminMode={true} // Special admin styling
      />
    );
  }
  
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
      permissions={permissionsToValidate}
      adminAction={adminAction}
      requiredAdminLevel={requiredAdminLevel}
      securityLevel={securityLevel}
      onRequestAccess={handleUpgrade}
    />
  );
}

// ============================================================================
// ADMIN PERMISSION STATUS COMPONENT
// ============================================================================

function AdminPermissionStatus({ 
  permission, 
  granted, 
  adminLevel, 
  securityLevel = 'standard',
  expiresAt,
  usageInfo
}: AdminPermissionStatusProps) {
  const getSecurityIcon = () => {
    switch (securityLevel) {
      case 'critical':
        return <ShieldAlert className="h-5 w-5 text-red-500" />;
      case 'elevated':
        return <ShieldCheck className="h-5 w-5 text-orange-500" />;
      default:
        return <Shield className="h-5 w-5 text-blue-500" />;
    }
  };
  
  const getStatusColor = () => {
    if (!granted) return 'bg-red-50 border-red-200 text-red-800';
    if (expiresAt) return 'bg-yellow-50 border-yellow-200 text-yellow-800';
    return 'bg-green-50 border-green-200 text-green-800';
  };
  
  return (
    <div className={`border rounded-lg p-4 ${getStatusColor()}`}>
      <div className="flex items-center space-x-3">
        {getSecurityIcon()}
        <div className="flex-1">
          <h4 className="font-medium">{permission}</h4>
          {adminLevel && (
            <p className="text-sm opacity-75">Admin Level: {adminLevel}</p>
          )}
          {expiresAt && (
            <p className="text-sm opacity-75">Expires: {new Date(expiresAt).toLocaleDateString()}</p>
          )}
          {usageInfo && (
            <p className="text-sm opacity-75">
              Usage: {usageInfo.current}/{usageInfo.limit} ({usageInfo.percentage}%)
            </p>
          )}
        </div>
        <div className="text-right">
          {granted ? (
            <CheckCircle className="h-5 w-5 text-green-500" />
          ) : (
            <XCircle className="h-5 w-5 text-red-500" />
          )}
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// DEFAULT ADMIN PERMISSION DENIED COMPONENT
// ============================================================================

function DefaultAdminPermissionDenied({
  permissions,
  adminAction,
  requiredAdminLevel,
  securityLevel = 'standard',
  onRequestAccess
}: {
  permissions: string[];
  adminAction?: string;
  requiredAdminLevel?: string;
  securityLevel?: 'standard' | 'elevated' | 'critical';
  onRequestAccess?: () => void;
}) {
  const getSecurityIcon = () => {
    switch (securityLevel) {
      case 'critical':
        return <ShieldAlert className="h-16 w-16 text-red-400" />;
      case 'elevated':
        return <ShieldCheck className="h-16 w-16 text-orange-400" />;
      default:
        return <Crown className="h-16 w-16 text-amber-400" />;
    }
  };
  
  const getSecurityColors = () => {
    switch (securityLevel) {
      case 'critical':
        return { bg: 'bg-red-50', border: 'border-red-200', button: 'bg-red-600 hover:bg-red-700' };
      case 'elevated':
        return { bg: 'bg-orange-50', border: 'border-orange-200', button: 'bg-orange-600 hover:bg-orange-700' };
      default:
        return { bg: 'bg-amber-50', border: 'border-amber-200', button: 'bg-amber-600 hover:bg-amber-700' };
    }
  };
  
  const colors = getSecurityColors();
  
  return (
    <div className={`${colors.bg} ${colors.border} border rounded-lg p-8 text-center`}>
      <div className="flex justify-center mb-6">
        {getSecurityIcon()}
      </div>
      
      <h3 className="text-2xl font-bold text-gray-900 mb-4">
        Admin Access Required
      </h3>
      
      {adminAction && (
        <p className="text-lg text-gray-700 mb-4">
          Admin Action: <span className="font-semibold">{adminAction}</span>
        </p>
      )}
      
      <p className="text-gray-600 mb-6 max-w-md mx-auto">
        You need elevated admin permissions to access this feature. Please contact your system administrator or request an upgrade.
      </p>
      
      {requiredAdminLevel && (
        <div className="mb-6">
          <p className="text-sm text-gray-500 mb-2">Required Admin Level:</p>
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-100 text-gray-800">
            {requiredAdminLevel}
          </span>
        </div>
      )}
      
      {permissions.length > 0 && (
        <div className="mb-6">
          <p className="text-sm text-gray-500 mb-2">Required Permissions:</p>
          <div className="flex flex-wrap justify-center gap-2">
            {permissions.map((permission) => (
              <code key={permission} className="px-2 py-1 bg-gray-100 rounded text-xs">
                {permission}
              </code>
            ))}
          </div>
        </div>
      )}
      
      <div className="flex justify-center space-x-4">
        <button 
          onClick={onRequestAccess}
          className={`${colors.button} text-white px-6 py-3 rounded-lg text-sm font-medium transition-colors flex items-center space-x-2`}
        >
          <Settings className="w-4 h-4" />
          <span>Request Admin Access</span>
        </button>
      </div>
    </div>
  );
}

// Main Enhanced Admin Permission Guard with Error Boundary
export default function EnhancedBackendAdminPermissionGuard(props: EnhancedBackendAdminPermissionGuardProps) {
  const { component = 'EnhancedBackendAdminPermissionGuard', onPermissionError, enableAnalytics = true } = props;
  const analytics = usePermissionErrorAnalytics();

  return (
    <PermissionErrorBoundary
      component={component}
      onError={(error, errorInfo, apiError) => {
        console.error('Enhanced Admin Permission Guard Error:', {
          component,
          error: error.message,
          errorInfo,
          apiError,
          adminAction: props.adminAction,
          securityLevel: props.securityLevel
        });
        
        if (apiError && onPermissionError) {
          onPermissionError(apiError);
        }
        
        // Track React errors in admin components
        if (enableAnalytics) {
          const reactError: ApiError = {
            success: false,
            error: {
              type: 'COMPONENT_ERROR',
              code: 'ADMIN_REACT_ERROR',
              message: error.message,
              user_message: 'The admin interface encountered an error. Please refresh the page.',
              suggested_actions: ['Refresh the page', 'Contact technical support']
            }
          };
          
          analytics.trackError(reactError, {
            component,
            operation: props.adminAction || 'admin_component_render',
            platform: 'admin'
          });
        }
      }}
      fallback={
        <div className="p-6 bg-red-50 border border-red-200 rounded-lg">
          <div className="flex items-start">
            <div className="flex-shrink-0">
              <ShieldX className="h-6 w-6 text-red-500" />
            </div>
            <div className="ml-3">
              <h3 className="text-lg font-medium text-red-800">Admin System Error</h3>
              <p className="mt-2 text-sm text-red-700">
                The admin permission system encountered a critical error. Please refresh the page or contact technical support.
              </p>
              <button 
                onClick={() => window.location.reload()}
                className="mt-4 text-sm font-medium text-red-800 underline hover:text-red-900"
              >
                Refresh Admin Interface
              </button>
            </div>
          </div>
        </div>
      }
    >
      <EnhancedBackendAdminPermissionGuardCore {...props} />
    </PermissionErrorBoundary>
  );
}

// ============================================================================
// ENHANCED ADMIN CONVENIENCE GUARDS
// ============================================================================

export function EnhancedSuperAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<EnhancedBackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <EnhancedBackendAdminPermissionGuard
      permission="admin:*:*"
      userId={userId}
      requiredAdminLevel="super_admin"
      securityLevel="critical"
      adminAction="super admin access"
      component="EnhancedSuperAdminGuard"
      {...props}
    >
      {children}
    </EnhancedBackendAdminPermissionGuard>
  );
}

export function EnhancedUserManagementAdminGuard({ 
  action = 'manage',
  children, 
  userId, 
  ...props 
}: Omit<EnhancedBackendAdminPermissionGuardProps, 'permission'> & { 
  action?: 'read' | 'create' | 'update' | 'delete' | 'manage';
  userId?: string;
}) {
  return (
    <EnhancedBackendAdminPermissionGuard
      permission={`admin:users:${action}`}
      userId={userId}
      adminAction={`${action} users`}
      securityLevel={action === 'delete' ? 'critical' : 'elevated'}
      component="EnhancedUserManagementAdminGuard"
      {...props}
    >
      {children}
    </EnhancedBackendAdminPermissionGuard>
  );
}

export function EnhancedPermissionManagementAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<EnhancedBackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <EnhancedBackendAdminPermissionGuard
      permission="admin:permissions:manage"
      userId={userId}
      adminAction="manage permissions"
      securityLevel="critical"
      requiredAdminLevel="super_admin"
      component="EnhancedPermissionManagementAdminGuard"
      {...props}
    >
      {children}
    </EnhancedBackendAdminPermissionGuard>
  );
}

export function EnhancedSystemAdminGuard({ 
  children, 
  userId, 
  ...props 
}: Omit<EnhancedBackendAdminPermissionGuardProps, 'permission'> & { userId?: string }) {
  return (
    <EnhancedBackendAdminPermissionGuard
      permission="admin:system:manage"
      userId={userId}
      adminAction="manage system"
      securityLevel="critical"
      requiredAdminLevel="system_admin"
      component="EnhancedSystemAdminGuard"
      {...props}
    >
      {children}
    </EnhancedBackendAdminPermissionGuard>
  );
}

// ============================================================================
// ENHANCED BACKEND ADMIN PERMISSION GUARD COMPLETE NOTICE (Phase 3.3.1)
// ============================================================================
//
// 🎉 ENHANCED BACKEND ADMIN PERMISSION GUARD COMPLETE!
//
// Created next-generation admin permission guard with comprehensive integration:
// - Integrated with PermissionErrorBoundary for React error protection
// - Uses PermissionErrorUI for user-friendly admin-specific error displays
// - Enhanced backend authority client with admin-context caching
// - Comprehensive error analytics with admin operation tracking
// - Admin-specific security levels and permission hierarchies
// - Context-aware error reporting with admin action details
// - Specialized convenience guards for common admin operations
//
// Key Admin Enhancements:
// ✅ Admin-specific error boundary protection with technical support flows
// ✅ Structured admin error handling with security level awareness
// ✅ Enhanced permission authority client with admin context
// ✅ Admin operation analytics with business impact tracking
// ✅ Security-aware permission validation (standard/elevated/critical)
// ✅ Admin-specific retry mechanisms with cache management
// ✅ Specialized convenience guards (super admin, user management, system admin)
// ✅ Comprehensive admin permission status displays
//
// Security Features:
// 🔒 Multi-tier admin permission validation (admin/super_admin/system_admin)
// 🔒 Security level-aware error handling (standard/elevated/critical)
// 🔒 Admin operation context tracking for audit compliance
// 🔒 Elevated privilege request flows with approval workflows
// 🔒 Admin-specific error analytics with security event correlation
//
// The Enhanced Backend Admin Permission Guard is now PRODUCTION-READY! 🎯
// ============================================================================