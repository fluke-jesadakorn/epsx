// Service for permission management API calls - refactored to use server actions
import { logger } from '@/lib/logger';
import { 
  getUserPermissions,
  checkPermission,
  checkFeatureAccess,
  checkRankingAccess,
  getPermissionProfiles as getServerPermissionProfiles,
  assignPermissionProfile as assignServerPermissionProfile,
  revokePermissionProfile as revokeServerPermissionProfile,
  getPermissionMatrix,
  getPaginatedFeatureAccess
} from '@epsx/server-actions';

// Types for backward compatibility
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

export interface UserPermissionStatus {
  userId: string;
  permissions: string[];
  roles: string[];
  packageTier: string;
  isActive: boolean;
}

export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  category: string;
  target_tier: string;
  is_active: boolean;
  created_at: string;
}

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
    try {
      const permissions = await getUserPermissions();
      // Transform server response to match expected format
      return permissions.map(p => ({
        id: p.id,
        name: p.name,
        description: p.description || '',
        resource: p.resource,
        action: p.action,
        risk: 'low' as const, // Default risk level
      }));
    } catch (error) {
      logger.error('Failed to fetch permissions from server actions', { error: error instanceof Error ? error.message : String(error) });
      // Fallback to mock data during development
      return this.getMockPermissions();
    }
  }

  async getPermissionProfiles(): Promise<LocalPermissionProfile[]> {
    try {
      const profiles = await getServerPermissionProfiles();
      
      // Transform server response to local format
      return profiles.map(profile => ({
        id: profile.id,
        name: profile.name,
        description: profile.description || '',
        category: (profile.category || 'User') as 'User' | 'Premium' | 'Admin' | 'System',
        permissions: [], // This would need to be populated from permissions field
        targetTier: profile.target_tier || 'Bronze',
        isActive: profile.isActive,
        userCount: undefined, // Not available in server response
        createdAt: profile.createdAt,
        features: [] // This would need to be derived from permissions or metadata
      }));
    } catch (error) {
      logger.error('Failed to fetch permission profiles from server actions', { error: error instanceof Error ? error.message : String(error) });
      return this.getMockPermissionProfiles();
    }
  }

  async checkUserPermissions(permission: string): Promise<boolean> {
    try {
      return await checkPermission(permission);
    } catch (error) {
      logger.error('Failed to check user permission', { permission, error: error instanceof Error ? error.message : String(error) });
      return false;
    }
  }

  async getUserPermissionStatus(): Promise<UserPermissionStatus | null> {
    try {
      const permissions = await getUserPermissions();
      // Transform to expected format - this would need to be adjusted based on actual server response
      return {
        userId: 'current', // This would come from the server response
        permissions: permissions.map(p => p.name),
        roles: [], // This would come from the server response
        packageTier: 'BRONZE', // This would come from the server response
        isActive: true
      };
    } catch (error) {
      logger.error('Failed to fetch user permission status', { error: error instanceof Error ? error.message : String(error) });
      return null;
    }
  }

  async assignPermissionProfile(userId: string, profileId: string, expiresAt?: string): Promise<void> {
    try {
      await assignServerPermissionProfile({
        profileId,
        userId,
        expiresAt
      });
      logger.info('Permission profile assigned successfully', { userId, profileId });
    } catch (error) {
      logger.error('Failed to assign permission profile', { userId, profileId, error: error instanceof Error ? error.message : String(error) });
      throw error;
    }
  }

  async revokePermissionProfile(userId: string, profileId: string): Promise<void> {
    try {
      await revokeServerPermissionProfile({
        profileId,
        userId
      });
      logger.info('Permission profile revoked successfully', { userId, profileId });
    } catch (error) {
      logger.error('Failed to revoke permission profile', { userId, profileId, error: error instanceof Error ? error.message : String(error) });
      throw error;
    }
  }

  async getRoles(): Promise<Role[]> {
    try {
      // This would need to be implemented as a new server action
      logger.error('getRoles not yet implemented with server actions');
      return this.getMockRoles();
    } catch (error) {
      logger.error('Failed to fetch roles from server actions', { error: error instanceof Error ? error.message : String(error) });
      return this.getMockRoles();
    }
  }

  async getUsers(): Promise<User[]> {
    try {
      // This would need to be implemented as a new server action
      logger.error('getUsers not yet implemented with server actions');
      return this.getMockUsers();
    } catch (error) {
      logger.error('Failed to fetch users from server actions', { error: error instanceof Error ? error.message : String(error) });
      return this.getMockUsers();
    }
  }

  async updateRolePermissions(roleId: string, permissionIds: string[]): Promise<void> {
    try {
      // This would need to be implemented as a new server action
      throw new Error('updateRolePermissions not yet implemented with server actions');
    } catch (error) {
      logger.error('Failed to update role permissions', { roleId, error: error instanceof Error ? error.message : String(error) });
      throw error;
    }
  }

  async updateUserPermissions(userId: string, roleIds: string[], directPermissions: string[]): Promise<void> {
    try {
      // This would need to be implemented as a new server action
      throw new Error('updateUserPermissions not yet implemented with server actions');
    } catch (error) {
      logger.error('Failed to update user permissions', { userId, error: error instanceof Error ? error.message : String(error) });
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