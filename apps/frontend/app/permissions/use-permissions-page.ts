'use client';

import type { PermissionDefinition } from '@/shared/api/permission-definitions';
import { loadPermissionDefinitions } from '@/shared/api/permission-definitions';
import type { ApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';
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

function parsePermissionWithTimestamp(permission: string): {
  basePermission: string;
  timestamp?: number;
} {
  const parts = permission.split(':');
  if (parts.length >= 4) {
    const lastPart = parts[parts.length - 1];
    const timestamp = parseInt(lastPart, 10);
    if (!isNaN(timestamp)) {
      const basePermission = parts.slice(0, -1).join(':');
      return { basePermission, timestamp };
    }
  }
  return { basePermission: permission };
}

interface UsePermissionsPageContext {
  userPermissions?: string[] | Record<string, unknown> | null;
  base: ApiClient;
}

export function usePermissionsPage(ctx: UsePermissionsPageContext) {
  const [activeTab, setActiveTab] = useState<
    'all' | 'active' | 'expiring' | 'expired' | 'analytics' | 'history'
  >('active');
  const [timestampedPermissions, setTimestampedPermissions] = useState<
    TimestampedPermission[]
  >([]);
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null);
  const [history, setHistory] = useState<PermissionHistoryItem[]>([]);
  const [isExporting, setIsExporting] = useState(false);
  const [permissionDefinitions, setPermissionDefinitions] = useState<
    Map<string, PermissionDefinition>
  >(new Map());

  useEffect(() => {
    loadPermissionDefinitions(ctx.base).then(setPermissionDefinitions);
  }, [ctx.base]);

  useEffect(() => {
    if (!ctx.userPermissions) {
      setTimestampedPermissions([]);
      return;
    }

    const permissionStrings =
      typeof ctx.userPermissions === 'object' &&
      ctx.userPermissions !== null &&
      !Array.isArray(ctx.userPermissions)
        ? Object.keys(ctx.userPermissions)
        : Array.isArray(ctx.userPermissions)
          ? ctx.userPermissions
          : [];

    const parsed = permissionStrings.map((perm) => {
      const { basePermission, timestamp } =
        parsePermissionWithTimestamp(perm);
      const expiresAt = timestamp;
      const isExpired = expiresAt ? Date.now() / 1000 > expiresAt : false;
      const timeRemaining = expiresAt ? expiresAt * 1000 - Date.now() : undefined;

      return {
        permission: perm,
        basePermission,
        expiresAt,
        isExpired,
        timeRemaining,
      };
    });

    setTimestampedPermissions(parsed);
  }, [ctx.userPermissions]);

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
    handleExport,
  };
}
