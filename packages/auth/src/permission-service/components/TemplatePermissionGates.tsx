import React, { ReactNode } from 'react';
import { 
  useTemplatePermissions, 
  useEffectivePermissions, 
  useResourcePermission,
  useTemplateManagement 
} from '../hooks/useTemplatePermissions';

export interface TemplatePermissionGateProps {
  userId?: string;
  permission?: string;
  resource?: string;
  action?: string;
  children: ReactNode;
  fallback?: ReactNode;
  showSource?: boolean;
}

/**
 * Permission gate that uses template-aware permission checking
 */
export function TemplatePermissionGate({
  userId,
  permission,
  resource,
  action,
  children,
  fallback = null,
  showSource = false,
}: TemplatePermissionGateProps) {
  // Use different hooks based on what props are provided
  const shouldUseResourcePermission = resource && action;
  const shouldUseTemplatePermission = permission && !shouldUseResourcePermission;

  const resourcePermissionResult = useResourcePermission(
    shouldUseResourcePermission ? userId : undefined,
    resource || '',
    action || ''
  );

  const templatePermissionResult = useTemplatePermissions(
    shouldUseTemplatePermission ? userId : undefined
  );

  let hasPermission = false;
  let source = '';

  if (shouldUseResourcePermission) {
    hasPermission = resourcePermissionResult.hasPermission;
    source = resourcePermissionResult.source;
  } else if (shouldUseTemplatePermission && permission) {
    hasPermission = templatePermissionResult.hasTemplatePermission(permission);
    source = 'Template permissions';
  }

  if (!hasPermission) {
    return <>{fallback}</>;
  }

  return (
    <>
      {children}
      {showSource && source && (
        <small className="text-xs text-gray-500 ml-2">
          Source: {source}
        </small>
      )}
    </>
  );
}

/**
 * Gate that shows template conflict warnings
 */
export function TemplateConflictWarning({ 
  userId, 
  children 
}: { 
  userId?: string; 
  children?: ReactNode;
}) {
  const { conflicts, hasConflicts, loading } = useTemplateManagement(userId);

  if (loading || !hasConflicts) {
    return <>{children}</>;
  }

  return (
    <div>
      <div className="bg-yellow-50 border border-yellow-200 rounded-md p-3 mb-4">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
            </svg>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-yellow-800">
              Permission Conflicts Detected
            </h3>
            <div className="mt-2 text-sm text-yellow-700">
              <ul className="list-disc list-inside space-y-1">
                {conflicts.map((conflict, index) => (
                  <li key={index}>
                    <strong>{conflict.permission}</strong>: {conflict.conflict}
                    <br />
                    <span className="text-xs">Resolution: {conflict.resolution}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </div>
      {children}
    </div>
  );
}

/**
 * Display active templates for a user
 */
export function ActiveTemplatesDisplay({ 
  userId,
  className = ''
}: { 
  userId?: string; 
  className?: string;
}) {
  const { activeTemplates, loading, hasActiveTemplates } = useTemplateManagement(userId);

  if (loading) {
    return (
      <div className={`animate-pulse ${className}`}>
        <div className="h-4 bg-gray-200 rounded w-1/2"></div>
      </div>
    );
  }

  if (!hasActiveTemplates) {
    return (
      <div className={`text-sm text-gray-500 ${className}`}>
        No active templates
      </div>
    );
  }

  return (
    <div className={className}>
      <h4 className="text-sm font-medium text-gray-900 mb-2">Active Templates</h4>
      <div className="space-y-2">
        {activeTemplates.map((template) => (
          <div key={template.templateId} className="bg-blue-50 rounded-md p-2">
            <div className="text-sm font-medium text-blue-900">
              {template.templateName}
            </div>
            <div className="text-xs text-blue-700">
              {template.contributedPermissions.length} permissions
            </div>
            <div className="flex flex-wrap gap-1 mt-1">
              {template.contributedPermissions.slice(0, 3).map((permission) => (
                <span 
                  key={permission}
                  className="inline-block px-1.5 py-0.5 bg-blue-100 text-blue-800 text-xs rounded"
                >
                  {permission}
                </span>
              ))}
              {template.contributedPermissions.length > 3 && (
                <span className="inline-block px-1.5 py-0.5 bg-blue-100 text-blue-800 text-xs rounded">
                  +{template.contributedPermissions.length - 3} more
                </span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * Combined gate that shows permission status with template information
 */
export function EnhancedPermissionGate({
  userId,
  permission,
  resource,
  action,
  children,
  fallback = null,
  showDetails = false,
}: TemplatePermissionGateProps & { showDetails?: boolean }) {
  const { effectivePermissions, loading } = useEffectivePermissions(userId);

  if (loading) {
    return (
      <div className="animate-pulse">
        <div className="h-8 bg-gray-200 rounded w-full"></div>
      </div>
    );
  }

  const permissionId = permission || (resource && action ? `${resource}.${action}` : '');
  const hasPermission = effectivePermissions?.templatePermissions.templatePermissions.includes(permissionId) || false;

  if (!hasPermission) {
    if (showDetails) {
      return (
        <div className="bg-red-50 border border-red-200 rounded-md p-3">
          <div className="text-sm text-red-800">
            Permission denied: <code className="font-mono">{permissionId}</code>
          </div>
          {effectivePermissions && (
            <div className="mt-2 text-xs text-red-700">
              Available permissions: {effectivePermissions.templatePermissions.templatePermissions.length}
            </div>
          )}
        </div>
      );
    }
    return <>{fallback}</>;
  }

  const sources = effectivePermissions?.templatePermissions.templateSources.filter(
    source => source.contributedPermissions.includes(permissionId)
  ) || [];

  return (
    <div>
      {showDetails && sources.length > 0 && (
        <div className="bg-green-50 border border-green-200 rounded-md p-2 mb-2">
          <div className="text-xs text-green-800">
            Permission granted by: {sources.map(s => s.templateName).join(', ')}
          </div>
        </div>
      )}
      {children}
    </div>
  );
}

/**
 * Higher-order component that provides template permission context
 */
export function withTemplatePermissions<P extends object>(
  Component: React.ComponentType<P & { hasPermission: (permission: string) => boolean }>
) {
  return function TemplatePermissionWrapper(
    props: P & { userId?: string }
  ) {
    const { hasTemplatePermission } = useTemplatePermissions(props.userId);

    return (
      <Component
        {...props}
        hasPermission={hasTemplatePermission}
      />
    );
  };
}

/**
 * Debug component to show all permission information
 */
export function PermissionDebugInfo({ 
  userId,
  className = ''
}: { 
  userId?: string; 
  className?: string;
}) {
  const { effectivePermissions, loading } = useEffectivePermissions(userId);

  if (loading) {
    return <div className={`animate-pulse h-32 bg-gray-200 rounded ${className}`}></div>;
  }

  if (!effectivePermissions) {
    return <div className={`text-gray-500 ${className}`}>No permission data available</div>;
  }

  return (
    <div className={`bg-gray-50 border rounded-md p-4 ${className}`}>
      <h4 className="font-medium text-gray-900 mb-3">Permission Debug Info</h4>
      
      <div className="space-y-3 text-sm">
        <div>
          <span className="font-medium">Template Permissions:</span>
          <span className="ml-2">{effectivePermissions.templatePermissions.templatePermissions.length}</span>
        </div>
        
        <div>
          <span className="font-medium">Active Templates:</span>
          <span className="ml-2">{effectivePermissions.templatePermissions.templateSources.length}</span>
        </div>
        
        <div>
          <span className="font-medium">Conflicts:</span>
          <span className="ml-2">{effectivePermissions.templatePermissions.conflicts.length}</span>
        </div>
        
        <div>
          <span className="font-medium">Combined Policies:</span>
          <span className="ml-2">{effectivePermissions.combinedPolicies.length}</span>
        </div>

        {effectivePermissions.templatePermissions.conflicts.length > 0 && (
          <div className="mt-3 p-2 bg-yellow-100 rounded">
            <div className="font-medium text-yellow-800 mb-1">Conflicts:</div>
            {effectivePermissions.templatePermissions.conflicts.map((conflict, index) => (
              <div key={index} className="text-xs text-yellow-700">
                {conflict.permission}: {conflict.resolution}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}