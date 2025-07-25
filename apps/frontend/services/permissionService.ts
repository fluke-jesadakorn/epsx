// Service for permission management API calls
import { logger } from '@/lib/logger';

export interface Permission {
  id: string;
  name: string;
  description: string;
  resource: string;
  action: string;
  risk: 'low' | 'medium' | 'high';
}

export interface Role {
  id: string;
  name: string;
  permissions: string[];
  userCount: number;
  isSystem: boolean;
}

export interface User {
  id: string;
  name: string;
  email: string;
  roles: string[];
  directPermissions: string[];
}

export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  category: 'User' | 'Premium' | 'Admin' | 'System';
  permissions: string[];
  targetTier: string;
  isActive: boolean;
  userCount?: number;
  createdAt: string;
  features: string[];
}

export interface UserPermissionStatus {
  userId: string;
  permissions: string[];
  profiles: string[];
  role: string;
  effectivePermissions: string[];
  hasWildcardAccess: boolean;
}

class PermissionService {
  private baseUrl = '/api/v1';

  async getPermissions(): Promise<Permission[]> {
    try {
      const response = await fetch(`${this.baseUrl}/permissions`);
      if (!response.ok) {
        throw new Error('Failed to fetch permissions');
      }
      return await response.json();
    } catch (error) {
      logger.error('Failed to fetch permissions from API', { error: error.message });
      // Fallback to mock data during development
      return this.getMockPermissions();
    }
  }

  async getPermissionProfiles(): Promise<PermissionProfile[]> {
    try {
      const response = await fetch(`${this.baseUrl}/permission-profiles`);
      if (!response.ok) {
        throw new Error('Failed to fetch permission profiles');
      }
      return await response.json();
    } catch (error) {
      logger.error('Failed to fetch permission profiles from API', { error: error.message });
      return this.getMockPermissionProfiles();
    }
  }

  async checkUserPermissions(permission: string): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/authentication/check-permission`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ permission }),
      });
      
      if (!response.ok) {
        return false;
      }
      
      const result = await response.json();
      return result.allowed || false;
    } catch (error) {
      logger.error('Failed to check user permission', { permission, error: error.message });
      return false;
    }
  }

  async getUserPermissionStatus(): Promise<UserPermissionStatus | null> {
    try {
      const response = await fetch(`${this.baseUrl}/authentication/profile`);
      if (!response.ok) {
        throw new Error('Failed to fetch user permission status');
      }
      
      const userData = await response.json();
      return {
        userId: userData.user_id,
        permissions: userData.permissions || [],
        profiles: userData.permission_profiles || [],
        role: userData.role || 'user',
        effectivePermissions: userData.effective_permissions || userData.permissions || [],
        hasWildcardAccess: userData.permissions?.includes('*') || false
      };
    } catch (error) {
      logger.error('Failed to fetch user permission status', { error: error.message });
      return null;
    }
  }

  async assignPermissionProfile(userId: string, profileId: string, expiresAt?: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/admin/users/${userId}/permission-profiles`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          permission_profile_id: profileId,
          expires_at: expiresAt,
          assigned_by: 'admin',
          reason: 'Manual assignment via admin interface'
        }),
      });
      
      if (!response.ok) {
        throw new Error('Failed to assign permission profile');
      }
      
      logger.info('Permission profile assigned successfully', { userId, profileId });
    } catch (error) {
      logger.error('Failed to assign permission profile', { userId, profileId, error: error.message });
      throw error;
    }
  }

  async revokePermissionProfile(userId: string, profileId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/admin/users/${userId}/permission-profiles/${profileId}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        throw new Error('Failed to revoke permission profile');
      }
      
      logger.info('Permission profile revoked successfully', { userId, profileId });
    } catch (error) {
      logger.error('Failed to revoke permission profile', { userId, profileId, error: error.message });
      throw error;
    }
  }

  async getRoles(): Promise<Role[]> {
    try {
      const response = await fetch(`${this.baseUrl}/roles`);
      if (!response.ok) {
        throw new Error('Failed to fetch roles');
      }
      return await response.json();
    } catch (error) {
      logger.error('Failed to fetch roles from API', { error: error.message });
      // Fallback to mock data during development
      return this.getMockRoles();
    }
  }

  async getUsers(): Promise<User[]> {
    try {
      const response = await fetch(`${this.baseUrl}/users`);
      if (!response.ok) {
        throw new Error('Failed to fetch users');
      }
      return await response.json();
    } catch (error) {
      logger.error('Failed to fetch users from API', { error: error.message });
      // Fallback to mock data during development
      return this.getMockUsers();
    }
  }

  async updateRolePermissions(roleId: string, permissionIds: string[]): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/roles/${roleId}/permissions`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ permissions: permissionIds }),
      });
      
      if (!response.ok) {
        throw new Error('Failed to update role permissions');
      }
      logger.info('Role permissions updated successfully', { roleId, permissionCount: permissionIds.length });
    } catch (error) {
      logger.error('Failed to update role permissions', { roleId, error: error.message });
      throw error;
    }
  }

  async updateUserPermissions(userId: string, roleIds: string[], directPermissions: string[]): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/users/${userId}/permissions`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          roles: roleIds,
          directPermissions 
        }),
      });
      
      if (!response.ok) {
        throw new Error('Failed to update user permissions');
      }
      logger.info('User permissions updated successfully', { userId, roleCount: roleIds.length, directPermissionCount: directPermissions.length });
    } catch (error) {
      logger.error('Failed to update user permissions', { userId, error: error.message });
      throw error;
    }
  }

  // Temporary mock data fallbacks
  private getMockPermissions(): Permission[] {
    return [
      { id: '1', name: 'View Analytics', description: 'Access analytics dashboard', resource: 'analytics', action: 'read', risk: 'low' },
      { id: '2', name: 'Export Analytics', description: 'Export analytics data', resource: 'analytics', action: 'export', risk: 'medium' },
      { id: '3', name: 'Delete Analytics', description: 'Delete analytics data', resource: 'analytics', action: 'delete', risk: 'high' },
      { id: '4', name: 'View Users', description: 'View user profiles', resource: 'users', action: 'read', risk: 'low' },
      { id: '5', name: 'Create Users', description: 'Create new users', resource: 'users', action: 'create', risk: 'medium' },
      { id: '6', name: 'Delete Users', description: 'Delete user accounts', resource: 'users', action: 'delete', risk: 'high' },
      { id: '7', name: 'Manage Billing', description: 'Access billing settings', resource: 'billing', action: 'manage', risk: 'high' },
      { id: '8', name: 'System Config', description: 'Configure system settings', resource: 'system', action: 'configure', risk: 'high' },
      { id: '9', name: 'View Audit Logs', description: 'Access audit logs', resource: 'audit', action: 'read', risk: 'medium' },
      { id: '10', name: 'Pattern Analysis', description: 'Run pattern analysis', resource: 'patterns', action: 'analyze', risk: 'low' }
    ];
  }

  private getMockRoles(): Role[] {
    return [
      { id: '1', name: 'Admin', permissions: ['1', '2', '3', '4', '5', '6', '7', '8', '9', '10'], userCount: 3, isSystem: true },
      { id: '2', name: 'Analyst', permissions: ['1', '2', '9', '10'], userCount: 12, isSystem: false },
      { id: '3', name: 'User Manager', permissions: ['4', '5'], userCount: 5, isSystem: false },
      { id: '4', name: 'Viewer', permissions: ['1', '4'], userCount: 25, isSystem: false }
    ];
  }

  private getMockUsers(): User[] {
    return [
      { id: '1', name: 'Admin User', email: 'admin@system.local', roles: ['1'], directPermissions: [] },
      { id: '2', name: 'Analytics User', email: 'analyst@company.com', roles: ['2'], directPermissions: ['3'] },
      { id: '3', name: 'Viewer User', email: 'viewer@company.com', roles: ['4'], directPermissions: [] }
    ];
  }

  private getMockPermissionProfiles(): PermissionProfile[] {
    return [
      {
        id: '1',
        name: 'Bronze User',
        description: 'Basic stock ranking access with 5 rankings maximum',
        category: 'User',
        permissions: [
          'api:rankings:read',
          'api:stock-rankings:basic',
          'route:/dashboard',
          'route:/rankings'
        ],
        targetTier: 'Bronze',
        isActive: true,
        userCount: 50,
        createdAt: '2025-01-01T00:00:00Z',
        features: ['basic_rankings', 'eps_analysis', 'basic_market_data']
      },
      {
        id: '2',
        name: 'Silver User',
        description: 'Enhanced analytics with 25 rankings maximum',
        category: 'Premium',
        permissions: [
          'api:rankings:*',
          'api:stock-rankings:silver',
          'api:analytics:read',
          'route:/analytics/*',
          'route:/premium/*',
          'route:/trading/*'
        ],
        targetTier: 'Silver',
        isActive: true,
        userCount: 25,
        createdAt: '2025-01-01T00:00:00Z',
        features: ['advanced_rankings', 'technical_indicators', 'price_alerts', 'market_screener']
      },
      {
        id: '3',
        name: 'Gold User',
        description: 'Premium features with 50 rankings maximum',
        category: 'Premium',
        permissions: [
          'api:premium:*',
          'api:stock-rankings:gold',
          'api:analytics:*',
          'route:/premium/*',
          'route:/reports/*'
        ],
        targetTier: 'Gold',
        isActive: true,
        userCount: 15,
        createdAt: '2025-01-01T00:00:00Z',
        features: ['ai_insights', 'pattern_recognition', 'custom_metrics', 'advanced_analytics']
      },
      {
        id: '4',
        name: 'Admin Assistant',
        description: 'Basic administrative functions',
        category: 'Admin',
        permissions: [
          'admin.users.view',
          'admin.users.manage',
          'admin.analytics.view',
          'admin.audit.view',
          'route:/admin/*'
        ],
        targetTier: 'Admin',
        isActive: true,
        userCount: 5,
        createdAt: '2025-01-01T00:00:00Z',
        features: ['user_management', 'basic_admin_analytics', 'audit_logs']
      },
      {
        id: '5',
        name: 'System Administrator',
        description: 'Full system administration access',
        category: 'System',
        permissions: [
          'admin:*',
          'system:*',
          'api:admin:*',
          'admin.permission_profiles.manage',
          'admin.system.configure'
        ],
        targetTier: 'Admin',
        isActive: true,
        userCount: 2,
        createdAt: '2025-01-01T00:00:00Z',
        features: ['full_admin_access', 'system_configuration', 'permission_management']
      }
    ];
  }
}

export const permissionService = new PermissionService();