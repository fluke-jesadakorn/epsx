import { useState, useEffect, useCallback } from 'react';
import { PermissionService } from '../PermissionService';
import type { TemplateEvaluationResult, TemplateContext } from '../template-integration';

/**
 * Hook for working with template-aware permissions
 */
export function useTemplatePermissions(userId?: string) {
  const [templatePermissions, setTemplatePermissions] = useState<TemplateEvaluationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const permissionService = PermissionService.getInstance();

  const loadTemplatePermissions = useCallback(async () => {
    if (!userId) return;

    setLoading(true);
    setError(null);

    try {
      const result = await permissionService.evaluateUserTemplates(userId);
      setTemplatePermissions(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load template permissions');
      setTemplatePermissions(null);
    } finally {
      setLoading(false);
    }
  }, [userId, permissionService]);

  useEffect(() => {
    loadTemplatePermissions();
  }, [loadTemplatePermissions]);

  const hasTemplatePermission = useCallback(
    (permission: string): boolean => {
      if (!templatePermissions) return false;
      
      // Check exact match
      if (templatePermissions.templatePermissions.includes(permission)) {
        return true;
      }

      // Check wildcard permissions
      return templatePermissions.templatePermissions.some(userPermission => {
        if (userPermission.endsWith('.*')) {
          const prefix = userPermission.slice(0, -2);
          return permission.startsWith(prefix + '.');
        }
        return false;
      });
    },
    [templatePermissions]
  );

  const refresh = useCallback(() => {
    permissionService.refreshCache();
    loadTemplatePermissions();
  }, [permissionService, loadTemplatePermissions]);

  return {
    templatePermissions,
    loading,
    error,
    hasTemplatePermission,
    refresh,
  };
}

/**
 * Hook for combined static and template permissions
 */
export function useEffectivePermissions(userId?: string) {
  const [effectivePermissions, setEffectivePermissions] = useState<{
    staticPermissions: any;
    templatePermissions: TemplateEvaluationResult;
    combinedPolicies: any[];
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const permissionService = PermissionService.getInstance();

  const loadEffectivePermissions = useCallback(async () => {
    if (!userId) return;

    setLoading(true);
    setError(null);

    try {
      const result = await permissionService.getEffectivePermissions(userId);
      setEffectivePermissions(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load effective permissions');
      setEffectivePermissions(null);
    } finally {
      setLoading(false);
    }
  }, [userId, permissionService]);

  useEffect(() => {
    loadEffectivePermissions();
  }, [loadEffectivePermissions]);

  const hasPermission = useCallback(
    (resource: string, action: string): boolean => {
      if (!effectivePermissions) return false;

      // Check template permissions first
      const templatePermissionId = `${resource}.${action}`;
      if (effectivePermissions.templatePermissions.templatePermissions.includes(templatePermissionId)) {
        return true;
      }

      // Check for conflicts that might deny permission
      const conflict = effectivePermissions.templatePermissions.conflicts.find(
        c => c.permission === templatePermissionId && c.resolution.includes('Denied')
      );
      if (conflict) return false;

      // Fallback to policy-based evaluation would go here
      return false;
    },
    [effectivePermissions]
  );

  const getPermissionSource = useCallback(
    (permission: string): string[] => {
      if (!effectivePermissions) return [];

      const sources: string[] = [];
      
      // Check template sources
      effectivePermissions.templatePermissions.templateSources.forEach(source => {
        if (source.contributedPermissions.includes(permission)) {
          sources.push(`Template: ${source.templateName}`);
        }
      });

      return sources;
    },
    [effectivePermissions]
  );

  const refresh = useCallback(() => {
    permissionService.refreshCache();
    loadEffectivePermissions();
  }, [permissionService, loadEffectivePermissions]);

  return {
    effectivePermissions,
    loading,
    error,
    hasPermission,
    getPermissionSource,
    refresh,
  };
}

/**
 * Hook for checking specific resource permissions with templates
 */
export function useResourcePermission(
  userId: string | undefined,
  resource: string,
  action: string
) {
  const [hasPermission, setHasPermission] = useState(false);
  const [loading, setLoading] = useState(false);
  const [source, setSource] = useState<string>('');

  const permissionService = PermissionService.getInstance();

  const checkPermission = useCallback(async () => {
    if (!userId) {
      setHasPermission(false);
      setSource('');
      return;
    }

    setLoading(true);

    try {
      const allowed = await permissionService.hasPermissionWithTemplates(
        userId,
        resource,
        action
      );
      setHasPermission(allowed);

      // Get source information
      const effectivePermissions = await permissionService.getEffectivePermissions(userId);
      const permissionId = `${resource}.${action}`;
      const templateSource = effectivePermissions.templatePermissions.templateSources.find(
        source => source.contributedPermissions.includes(permissionId)
      );
      
      if (templateSource) {
        setSource(`Template: ${templateSource.templateName}`);
      } else {
        setSource('Static permissions');
      }
    } catch (error) {
      console.error('Error checking permission:', error);
      setHasPermission(false);
      setSource('Error');
    } finally {
      setLoading(false);
    }
  }, [userId, resource, action, permissionService]);

  useEffect(() => {
    checkPermission();
  }, [checkPermission]);

  const refresh = useCallback(() => {
    permissionService.refreshCache();
    checkPermission();
  }, [permissionService, checkPermission]);

  return {
    hasPermission,
    loading,
    source,
    refresh,
  };
}

/**
 * Hook for template-specific operations
 */
export function useTemplateManagement(userId?: string) {
  const [activeTemplates, setActiveTemplates] = useState<Array<{
    templateId: string;
    templateName: string;
    contributedPermissions: string[];
  }>>([]);
  const [conflicts, setConflicts] = useState<Array<{
    permission: string;
    conflict: string;
    resolution: string;
  }>>([]);
  const [loading, setLoading] = useState(false);

  const permissionService = PermissionService.getInstance();

  const loadTemplateInfo = useCallback(async () => {
    if (!userId) return;

    setLoading(true);

    try {
      const templateResult = await permissionService.evaluateUserTemplates(userId);
      setActiveTemplates(templateResult.templateSources);
      setConflicts(templateResult.conflicts);
    } catch (error) {
      console.error('Error loading template info:', error);
      setActiveTemplates([]);
      setConflicts([]);
    } finally {
      setLoading(false);
    }
  }, [userId, permissionService]);

  useEffect(() => {
    loadTemplateInfo();
  }, [loadTemplateInfo]);

  const hasConflicts = conflicts.length > 0;
  const hasActiveTemplates = activeTemplates.length > 0;

  const refresh = useCallback(() => {
    permissionService.refreshCache();
    loadTemplateInfo();
  }, [permissionService, loadTemplateInfo]);

  return {
    activeTemplates,
    conflicts,
    hasConflicts,
    hasActiveTemplates,
    loading,
    refresh,
  };
}