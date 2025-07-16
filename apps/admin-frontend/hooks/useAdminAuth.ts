"use client";

import { AdminRole } from '@/types/admin/roles';
import { AdminService } from '@/services/adminService';

export function useAdminAuth() {
  // This would connect to your admin auth system
  // For now, using mock data - replace with actual auth implementation
  const adminUser = {
    id: 'admin-1',
    role: AdminRole.ADMIN, // This should come from your auth system
    email: 'admin@example.com',
    permissions: [],
    assignedBy: 'system',
    assignedAt: new Date(),
    isActive: true
  };

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
    hasPermission,
    canManageUsers,
    canViewPayments,
    canManageSystem,
    canViewAnalytics,
    getAvailableActions
  };
}
