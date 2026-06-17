'use client';

import type { PermissionDefinition } from '@/shared/api/permission-definitions';
import { loadPermissionDefinitions } from '@/shared/api/permission-definitions';
import type { UserPermissionInfo, UserPermissionStatus } from '@/shared/api/permissions';
import type { UnifiedApiClient as ApiClient } from '@/shared/utils/api-client';
import { useCallback, useEffect, useState } from 'react';
import type {
  AnalyticsData,
  PermissionHistoryItem,
  TimestampedPermission,
} from './permissions-sections';

const getPermissionAnalytics = async () => null;

const getPermissionHistory = async (_limit: number) => [];

const exportPermissionsData = async (format: string) => ({
  data: format === 'json' ? '[]' : '',
  filename: `permissions.${format}`,
});

function toTimestampedPermission(info: UserPermissionInfo): TimestampedPermission {
  const expiresAtUnix = info.expires_at
    ? Math.floor(new Date(info.expires_at).getTime() / 1000)
    : undefined;
  const timeRemaining = info.time_until_expiry !== null
    ? info.time_until_expiry * 1000
    : undefined;

  return {
    permission: info.permission,
    basePermission: info.permission,
    platform: info.permission.split(':')[0] ?? 'unknown',
    expiresAt: expiresAtUnix,
    isExpired: !info.is_active,
    timeRemaining,
  };
}

interface UsePermissionsPageContext {
  base: ApiClient;
}

export function usePermissionsPage(ctx: UsePermissionsPageContext) {
  const [activeTab, setActiveTab] = useState<
    'all' | 'active' | 'expiring' | 'expired' | 'analytics' | 'history'
  >('active');
  const [timestampedPermissions, setTimestampedPermissions] = useState<
    TimestampedPermission[]
  >([]);
  const [permissionStatus, setPermissionStatus] = useState<UserPermissionStatus | null>(null);
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null);
  const [history, setHistory] = useState<PermissionHistoryItem[]>([]);
  const [isExporting, setIsExporting] = useState(false);
  const [permissionDefinitions, setPermissionDefinitions] = useState<
    Map<string, PermissionDefinition>
  >(new Map());

  useEffect(() => {
    loadPermissionDefinitions(ctx.base).then(setPermissionDefinitions);
  }, [ctx.base]);

  const fetchPermissions = useCallback(async () => {
    try {
      const res = await ctx.base.get<UserPermissionStatus>(
        '/api/users/permissions/status',
        { include_expired: true }
      );
      if (res.success && res.data !== null) {
        setPermissionStatus(res.data);
        setTimestampedPermissions(res.data.permissions.map(toTimestampedPermission));
      }
    } catch {
      // fallback: keep empty
    }
  }, [ctx.base]);

  useEffect(() => {
    void fetchPermissions();
  }, [fetchPermissions]);

  useEffect(() => {
    if (activeTab === 'analytics') {
      getPermissionAnalytics()
        .then(setAnalytics)
        .catch(() => {});
    }
  }, [activeTab]);

  useEffect(() => {
    if (activeTab === 'history') {
      getPermissionHistory(20)
        .then(setHistory)
        .catch(() => {});
    }
  }, [activeTab]);

  const handleExport = async (format: 'json' | 'csv') => {
    setIsExporting(true);
    try {
      const { data, filename } = await exportPermissionsData(format);
      const blob = new Blob([data], {
        type: format === 'json' ? 'application/json' : 'text/csv',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch {
      setIsExporting(false);
    } finally {
      setIsExporting(false);
    }
  };

  const filteredPermissions = timestampedPermissions.filter((perm) => {
    switch (activeTab) {
      case 'active':
        return !perm.isExpired;
      case 'expiring':
        return (
          !perm.isExpired &&
          perm.timeRemaining &&
          perm.timeRemaining < 24 * 60 * 60 * 1000
        );
      case 'expired':
        return perm.isExpired;
      default:
        return true;
    }
  });

  return {
    activeTab,
    setActiveTab,
    timestampedPermissions,
    filteredPermissions,
    analytics,
    history,
    isExporting,
    permissionDefinitions,
    permissionStatus,
    handleExport,
  };
}
