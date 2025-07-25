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
        const response = await fetch('/api/admin/auth/profile');
        if (response.ok) {
          const profile = await response.json();
          setAdminUser({
            id: profile.user_id,
            role: profile.role || AdminRole.ADMIN,
            email: profile.email,
            permissions: profile.permissions || [],
            assignedBy: 'system',
            assignedAt: new Date(),
            isActive: true
          });
        }
      } catch (error) {
        adminLogger.error('Failed to load admin profile', { error: error.message }, 'useAdminAuth');
        setAdminUser(null);
      }
      setLoading(false);
    };

    loadAdminProfile();
  }, []);

  const hasPermission = (resource: string, action: string): boolean => {
    return AdminService.hasPermission(adminUser.role, resource, action);
  };

  const canManageUsers = (): boolean => {
    return AdminService.canManageUsers(adminUser.role);
  };

  const canViewPayments = (): boolean => {
    return AdminService.canViewPayments(adminUser.role);
  };

  const canManageSystem = (): boolean => {
    return AdminService.canManageSystem(adminUser.role);
  };

  const canViewAnalytics = (): boolean => {
    return AdminService.canViewAnalytics(adminUser.role);
  };

  const getAvailableActions = (resource: string): string[] => {
    return AdminService.getAvailableActions(adminUser.role, resource);
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
