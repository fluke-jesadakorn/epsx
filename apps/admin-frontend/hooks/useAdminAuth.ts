"use client";

import { useState, useEffect } from 'react';
import { AdminRole } from '@/types/admin/roles';
import { AdminService } from '@/services/adminService';
import { adminLogger } from '@/lib/logger';

export function useAdminAuth() {
  const [adminUser, setAdminUser] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadAdminProfile = async () => {
      try {
        const profile = await AdminService.getCurrentUser();
        if (profile) {
          setAdminUser({
            id: profile.user_id || profile.id,
            role: profile.role || AdminRole.ADMIN,
            email: profile.email,
            permissions: profile.permissions || [],
            assignedBy: 'system',
            assignedAt: new Date(),
            isActive: true
          });
        }
      } catch (error) {
        adminLogger.error('Failed to load admin profile', { error: error instanceof Error ? error.message : String(error) }, 'useAdminAuth');
        setAdminUser(null);
      }
      setLoading(false);
    };

    loadAdminProfile();
  }, []);

  const hasPermission = async (resource: string, action: string): Promise<boolean> => {
    if (!adminUser?.id) return false;
    try {
      const result = await AdminService.evaluatePermission({
        userId: adminUser.id,
        action: `${action}:${resource}`,
        resource: `admin:${resource}`
      });
      return result?.allowed || false;
    } catch (error) {
      adminLogger.error('Permission check failed', { resource, action, error });
      return false;
    }
  };

  const canManageUsers = (): boolean => {
    return adminUser?.role === AdminRole.SUPER_ADMIN || adminUser?.role === AdminRole.ADMIN;
  };

  const canViewPayments = (): boolean => {
    return adminUser?.role === AdminRole.SUPER_ADMIN || adminUser?.role === AdminRole.ADMIN;
  };

  const canManageSystem = (): boolean => {
    return adminUser?.role === AdminRole.SUPER_ADMIN;
  };

  const canViewAnalytics = (): boolean => {
    return adminUser?.role === AdminRole.SUPER_ADMIN || adminUser?.role === AdminRole.ADMIN;
  };

  const getAvailableActions = (resource: string): string[] => {
    if (!adminUser) return [];
    
    const baseActions = ['view'];
    if (adminUser.role === AdminRole.SUPER_ADMIN) {
      return [...baseActions, 'create', 'edit', 'delete', 'manage'];
    } else if (adminUser.role === AdminRole.ADMIN) {
      return [...baseActions, 'create', 'edit'];
    }
    
    return baseActions;
  };

  return {
    adminUser,
    loading,
    hasPermission,
    canManageUsers,
    canViewPayments,
    canManageSystem,
    canViewAnalytics,
    getAvailableActions
  };
}
