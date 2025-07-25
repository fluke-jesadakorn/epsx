// Service for permission management API calls
import { logger } from '@/lib/logger';
import { apiClient, type Permission, type Role, type UserPermissionStatus, type PermissionProfile, type ApiResponse, isApiError } from '@epsx/api-client';

// Re-export types from api-client for backward compatibility
export type { Permission, Role, UserPermissionStatus, PermissionProfile } from '@epsx/api-client';

// Local types that extend api-client types
export interface User {
  id: string;
  name: string;
  email: string;
  roles: string[];
  directPermissions: string[];
}

// Extended PermissionProfile for local use with additional fields
export interface LocalPermissionProfile extends Omit<PermissionProfile, 'category' | 'target_tier' | 'is_active' | 'created_at'> {
  category: 'User' | 'Premium' | 'Admin' | 'System';
  permissions: string[];
  targetTier: string;
  isActive: boolean;
  userCount?: number;
  createdAt: string;
  features: string[];
}

class PermissionService {
  async getPermissions(): Promise<Permission[]> {
    const response = await apiClient.getPermissions();
    if (isApiError(response)) {
      logger.error('Failed to fetch permissions from API', { error: response.error, details: response.details });
      // Fallback to mock data during development
      return this.getMockPermissions();
    }
    return response.data;
  }

  async getPermissionProfiles(): Promise<LocalPermissionProfile[]> {
    const response = await apiClient.listPermissionProfiles();
    if (isApiError(response)) {
      logger.error('Failed to fetch permission profiles from API', { error: response.error, details: response.details });
      return this.getMockPermissionProfiles();
    }
    
    // Transform API response to local format
    return response.data.permission_profiles.map(profile => ({
      id: profile.id,
      name: profile.name,
      description: profile.description,
      category: profile.category as 'User' | 'Premium' | 'Admin' | 'System',
      permissions: [], // This would need to be populated from another endpoint
      targetTier: profile.target_tier,
      isActive: profile.is_active,
      userCount: undefined, // Not available in API response
      createdAt: profile.created_at,
      features: [] // This would need to be derived from permissions or metadata
    }));
  }

  async checkUserPermissions(permission: string): Promise<boolean> {
    const response = await apiClient.checkUserPermission({ permission });
    if (isApiError(response)) {
      logger.error('Failed to check user permission', { permission, error: response.error, details: response.details });
      return false;
    }
    return response.data.allowed || false;
  }

  async getUserPermissionStatus(): Promise<UserPermissionStatus | null> {
    const response = await apiClient.getUserPermissionStatus();
    if (isApiError(response)) {
      logger.error('Failed to fetch user permission status', { error: response.error, details: response.details });
      return null;
    }
    return response.data;
  }

  async assignPermissionProfile(userId: string, profileId: string, expiresAt?: string): Promise<void> {
    const response = await apiClient.assignUserPermissionProfile(userId, profileId, expiresAt);
    if (isApiError(response)) {
      logger.error('Failed to assign permission profile', { userId, profileId, error: response.error, details: response.details });
      throw new Error(response.error);
    }
    logger.info('Permission profile assigned successfully', { userId, profileId });
  }

  async revokePermissionProfile(userId: string, profileId: string): Promise<void> {
    const response = await apiClient.revokeUserPermissionProfile(userId, profileId);
    if (isApiError(response)) {
      logger.error('Failed to revoke permission profile', { userId, profileId, error: response.error, details: response.details });
      throw new Error(response.error);
    }
    logger.info('Permission profile revoked successfully', { userId, profileId });
  }

  async getRoles(): Promise<Role[]> {
    const response = await apiClient.getRoles();
    if (isApiError(response)) {
      logger.error('Failed to fetch roles from API', { error: response.error, details: response.details });
      // Fallback to mock data during development
      return this.getMockRoles();
    }
    return response.data;
  }

  async getUsers(): Promise<User[]> {
    const response = await apiClient.listUsers();
    if (isApiError(response)) {
      logger.error('Failed to fetch users from API', { error: response.error, details: response.details });
      // Fallback to mock data during development
      return this.getMockUsers();
    }
    
    // Transform API response to local User format
    return response.data.users.map(user => ({
      id: user.uid,
      name: user.displayName || user.email,
      email: user.email,
      roles: user.role ? [user.role] : [],
      directPermissions: user.permissions || []
    }));
  }

  async updateRolePermissions(roleId: string, permissionIds: string[]): Promise<void> {
    const response = await apiClient.updateRolePermissions(roleId, permissionIds);
    if (isApiError(response)) {
      logger.error('Failed to update role permissions', { roleId, error: response.error, details: response.details });
      throw new Error(response.error);
    }
    logger.info('Role permissions updated successfully', { roleId, permissionCount: permissionIds.length });
  }

  async updateUserPermissions(userId: string, roleIds: string[], directPermissions: string[]): Promise<void> {
    const response = await apiClient.updateUserPermissions(userId, roleIds, directPermissions);
    if (isApiError(response)) {
      logger.error('Failed to update user permissions', { userId, error: response.error, details: response.details });
      throw new Error(response.error);
    }
    logger.info('User permissions updated successfully', { userId, roleCount: roleIds.length, directPermissionCount: directPermissions.length });
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

  private getMockPermissionProfiles(): LocalPermissionProfile[] {
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