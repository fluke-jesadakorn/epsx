import { 
  collection, 
  doc, 
  getDocs, 
  getDoc, 
  addDoc, 
  query, 
  where, 
  orderBy, 
  limit as firestoreLimit,
  Timestamp,
  writeBatch
} from 'firebase/firestore';
import { db } from '@/lib/firebase';
import type { 
  UserWithPermissions, 
  CustomPermission, 
  PackagePermission, 
  EffectivePermission,
  PermissionAuditLog,
  Permission
} from '../types/admin/iam-enhanced';
import { PackageTier, SubscriptionStatus, PermissionSource } from '../types/admin/iam-enhanced';
import { buildPackagePermissions } from '../config/packagePermissions';

export class FirebaseIAMService {
  private readonly collections = {
    users: 'users',
    customPermissions: 'custom_permissions',
    packagePermissions: 'package_permissions',
    auditLogs: 'permission_audit_logs',
    effectivePermissions: 'effective_permissions'
  };

  /**
   * Get all users with IAM data
   */
  async getUsers(filters?: {
    packageTier?: PackageTier;
    subscriptionStatus?: string;
    hasCustomPermissions?: boolean;
  }): Promise<UserWithPermissions[]> {
    try {
      // Check if Firebase is properly initialized
      if (!db) {
        console.warn('Firebase not initialized, returning mock data');
        return this.getMockUsers(filters);
      }

      let userQuery = query(collection(db, this.collections.users));

      // Apply filters
      if (filters?.packageTier) {
        userQuery = query(userQuery, where('packageTier', '==', filters.packageTier));
      }
      if (filters?.subscriptionStatus) {
        userQuery = query(userQuery, where('subscriptionStatus', '==', filters.subscriptionStatus));
      }

      const usersSnapshot = await getDocs(userQuery);
      
      // If no users found in Firestore, return mock data for development
      if (usersSnapshot.empty) {
        console.warn('No users found in Firestore, returning mock data for development');
        return this.getMockUsers(filters);
      }

      const users: UserWithPermissions[] = [];

      for (const userDoc of usersSnapshot.docs) {
        try {
          const userData = userDoc.data();
          const user = await this.buildUserWithPermissions(userDoc.id, userData);
          
          // Apply hasCustomPermissions filter
          if (filters?.hasCustomPermissions && user.customPermissions.length === 0) {
            continue;
          }
          
          users.push(user);
        } catch (userError) {
          console.warn(`Failed to build user ${userDoc.id}:`, userError);
          // Continue with other users
        }
      }

      return users;
    } catch (error) {
      console.error('Error fetching users from Firebase:', error);
      console.warn('Falling back to mock data');
      return this.getMockUsers(filters);
    }
  }

  /**
   * Get user with all permission details
   */
  async getUserWithPermissions(userId: string): Promise<UserWithPermissions> {
    try {
      if (!db) {
        console.warn('Firebase not initialized, returning mock user');
        return this.getMockUsers().find(u => u.id === userId) || this.getMockUsers()[0];
      }

      const userDoc = await getDoc(doc(db, this.collections.users, userId));
      
      if (!userDoc.exists()) {
        console.warn(`User ${userId} not found in Firestore, returning mock user`);
        return this.getMockUsers().find(u => u.id === userId) || this.getMockUsers()[0];
      }

      return await this.buildUserWithPermissions(userId, userDoc.data());
    } catch (error) {
      console.error('Error fetching user with permissions:', error);
      console.warn('Falling back to mock user data');
      return this.getMockUsers().find(u => u.id === userId) || this.getMockUsers()[0];
    }
  }

  /**
   * Update user package tier
   */
  async updateUserPackageTier(userId: string, newTier: PackageTier, updatedBy: string): Promise<void> {
    try {
      const batch = writeBatch(db);
      
      // Update user document
      const userRef = doc(db, this.collections.users, userId);
      batch.update(userRef, {
        packageTier: newTier,
        updatedAt: Timestamp.now(),
        updatedBy
      });

      // Apply new package permissions
      await this.applyPackagePermissions(userId, newTier, batch);

      // Create audit log
      await this.createAuditLog({
        userId,
        action: `Package tier updated to ${newTier}`,
        resource: 'user:package_tier',
        performedBy: updatedBy,
        timestamp: new Date(),
        metadata: { newTier, oldTier: 'unknown' }
      });

      await batch.commit();
    } catch (error) {
      console.error('Error updating user package tier:', error);
      throw error;
    }
  }

  /**
   * Apply package permissions to user
   */
  async applyPackagePermissions(userId: string, packageTier: PackageTier, batch?: any): Promise<void> {
    try {
      const shouldCommit = !batch;
      if (!batch) {
        batch = writeBatch(db);
      }

      // Get package permissions for this tier
      const packagePermissions = buildPackagePermissions()[packageTier];

      // Remove existing package permissions
      const existingPermissionsQuery = query(
        collection(db, this.collections.effectivePermissions),
        where('userId', '==', userId),
        where('source', '==', PermissionSource.PACKAGE)
      );
      const existingSnapshot = await getDocs(existingPermissionsQuery);
      
      existingSnapshot.forEach(doc => {
        batch.delete(doc.ref);
      });

      // Add new package permissions
      for (const permission of packagePermissions) {
        const effectivePermission: Omit<EffectivePermission, 'id'> = {
          featureId: permission.featureId,
          permission: permission.permission,
          source: PermissionSource.PACKAGE,
          grantedAt: new Date(),
          grantedBy: 'SYSTEM'
        };

        const permissionRef = doc(collection(db, this.collections.effectivePermissions));
        batch.set(permissionRef, {
          ...effectivePermission,
          userId,
          grantedAt: Timestamp.now()
        });
      }

      if (shouldCommit) {
        await batch.commit();
      }
    } catch (error) {
      console.error('Error applying package permissions:', error);
      throw error;
    }
  }

  /**
   * Grant custom permission to user
   */
  async grantCustomPermission(
    userId: string,
    featureId: string,
    permission: Permission,
    grantedBy: string,
    options?: { expiresAt?: Date; reason?: string }
  ): Promise<CustomPermission> {
    try {
      const customPermission: Omit<CustomPermission, 'id'> = {
        userId,
        featureId,
        permission,
        grantedBy,
        grantedAt: new Date(),
        expiresAt: options?.expiresAt,
        reason: options?.reason,
        isActive: true
      };

      // Add to custom permissions collection
      const customPermRef = await addDoc(collection(db, this.collections.customPermissions), {
        ...customPermission,
        grantedAt: Timestamp.now(),
        expiresAt: options?.expiresAt ? Timestamp.fromDate(options.expiresAt) : null
      });

      // Add to effective permissions
      const effectivePermission: Omit<EffectivePermission, 'id'> = {
        featureId,
        permission,
        source: PermissionSource.CUSTOM,
        grantedAt: new Date(),
        expiresAt: options?.expiresAt,
        grantedBy
      };

      await addDoc(collection(db, this.collections.effectivePermissions), {
        ...effectivePermission,
        userId,
        grantedAt: Timestamp.now(),
        expiresAt: options?.expiresAt ? Timestamp.fromDate(options.expiresAt) : null,
        customPermissionId: customPermRef.id
      });

      // Create audit log
      await this.createAuditLog({
        userId,
        action: `Granted custom permission: ${featureId}`,
        resource: `permission:${featureId}`,
        performedBy: grantedBy,
        reason: options?.reason,
        timestamp: new Date(),
        metadata: { featureId, permission }
      });

      return {
        id: customPermRef.id,
        ...customPermission
      };
    } catch (error) {
      console.error('Error granting custom permission:', error);
      throw error;
    }
  }

  /**
   * Revoke custom permission
   */
  async revokeCustomPermission(permissionId: string, revokedBy: string, reason?: string): Promise<void> {
    try {
      const batch = writeBatch(db);

      // Get custom permission details
      const customPermDoc = await getDoc(doc(db, this.collections.customPermissions, permissionId));
      if (!customPermDoc.exists()) {
        throw new Error('Custom permission not found');
      }

      const customPermData = customPermDoc.data() as CustomPermission;

      // Deactivate custom permission
      batch.update(customPermDoc.ref, {
        isActive: false,
        revokedAt: Timestamp.now(),
        revokedBy,
        revokeReason: reason
      });

      // Remove from effective permissions
      const effectivePermQuery = query(
        collection(db, this.collections.effectivePermissions),
        where('userId', '==', customPermData.userId),
        where('customPermissionId', '==', permissionId)
      );
      const effectiveSnapshot = await getDocs(effectivePermQuery);
      
      effectiveSnapshot.forEach(doc => {
        batch.delete(doc.ref);
      });

      // Create audit log
      await this.createAuditLog({
        userId: customPermData.userId,
        action: `Revoked custom permission: ${customPermData.featureId}`,
        resource: `permission:${customPermData.featureId}`,
        performedBy: revokedBy,
        reason,
        timestamp: new Date(),
        metadata: { permissionId, featureId: customPermData.featureId }
      });

      await batch.commit();
    } catch (error) {
      console.error('Error revoking custom permission:', error);
      throw error;
    }
  }

  /**
   * Check if user has feature access
   */
  async hasFeatureAccess(userId: string, featureId: string): Promise<boolean> {
    try {
      if (!db) {
        console.warn('Firebase not initialized, checking mock permissions');
        const mockUser = this.getMockUsers().find(u => u.id === userId);
        if (!mockUser) return false;
        
        // Check if the feature is in package permissions
        return mockUser.packagePermissions.some(perm => perm.featureId === featureId);
      }

      const effectivePermQuery = query(
        collection(db, this.collections.effectivePermissions),
        where('userId', '==', userId),
        where('featureId', '==', featureId)
      );

      const snapshot = await getDocs(effectivePermQuery);
      
      // Check if any non-expired permission exists
      const now = new Date();
      for (const doc of snapshot.docs) {
        const permission = doc.data();
        
        // Check if permission is expired
        if (permission.expiresAt && permission.expiresAt.toDate() < now) {
          continue;
        }
        
        return true;
      }

      return false;
    } catch (error) {
      console.error('Error checking feature access:', error);
      // Default to false for security
      return false;
    }
  }

  /**
   * Get user's effective permissions
   */
  async getUserEffectivePermissions(userId: string): Promise<EffectivePermission[]> {
    try {
      const effectivePermQuery = query(
        collection(db, this.collections.effectivePermissions),
        where('userId', '==', userId),
        orderBy('grantedAt', 'desc')
      );

      const snapshot = await getDocs(effectivePermQuery);
      const permissions: EffectivePermission[] = [];
      const now = new Date();

      snapshot.forEach(doc => {
        const data = doc.data();
        
        // Skip expired permissions
        if (data.expiresAt && data.expiresAt.toDate() < now) {
          return;
        }

        permissions.push({
          featureId: data.featureId,
          permission: data.permission,
          source: data.source,
          grantedAt: data.grantedAt.toDate(),
          expiresAt: data.expiresAt?.toDate(),
          grantedBy: data.grantedBy
        });
      });

      return permissions;
    } catch (error) {
      console.error('Error fetching effective permissions:', error);
      throw error;
    }
  }

  /**
   * Bulk apply template permissions
   */
  async bulkApplyTemplate(userIds: string[], templateId: string, appliedBy: string): Promise<void> {
    try {
      // This would require template definitions in Firestore
      // For now, implement as a placeholder
      const batch = writeBatch(db);

      for (const userId of userIds) {
        await this.createAuditLog({
          userId,
          action: `Applied permission template: ${templateId}`,
          resource: `template:${templateId}`,
          performedBy: appliedBy,
          timestamp: new Date(),
          metadata: { templateId }
        });
      }

      await batch.commit();
    } catch (error) {
      console.error('Error bulk applying template:', error);
      throw error;
    }
  }

  /**
   * Create audit log entry
   */
  async createAuditLog(logEntry: Omit<PermissionAuditLog, 'id'>): Promise<void> {
    try {
      await addDoc(collection(db, this.collections.auditLogs), {
        ...logEntry,
        timestamp: Timestamp.now()
      });
    } catch (error) {
      console.error('Error creating audit log:', error);
      // Don't throw - audit log failures shouldn't break the main operation
    }
  }

  /**
   * Get user audit logs
   */
  async getUserAuditLogs(userId: string, limitCount: number = 50): Promise<PermissionAuditLog[]> {
    try {
      const auditQuery = query(
        collection(db, this.collections.auditLogs),
        where('userId', '==', userId),
        orderBy('timestamp', 'desc'),
        firestoreLimit(limitCount)
      );

      const snapshot = await getDocs(auditQuery);
      const logs: PermissionAuditLog[] = [];

      snapshot.forEach(doc => {
        const data = doc.data();
        logs.push({
          id: doc.id,
          userId: data.userId,
          action: data.action,
          resource: data.resource,
          performedBy: data.performedBy,
          reason: data.reason,
          timestamp: data.timestamp.toDate(),
          metadata: data.metadata
        });
      });

      return logs;
    } catch (error) {
      console.error('Error fetching audit logs:', error);
      throw error;
    }
  }

  /**
   * Preview package upgrade effects
   */
  async previewPackageUpgrade(userId: string, newTier: PackageTier): Promise<{
    currentPermissions: EffectivePermission[];
    newPermissions: PackagePermission[];
    addedPermissions: PackagePermission[];
    removedPermissions: PackagePermission[];
  }> {
    try {
      const currentPermissions = await this.getUserEffectivePermissions(userId);
      const newPermissions = buildPackagePermissions()[newTier];
      
      // Calculate added permissions (simplified - in real implementation, compare with current package permissions)
      const addedPermissions = newPermissions;
      const removedPermissions: PackagePermission[] = [];

      return {
        currentPermissions,
        newPermissions,
        addedPermissions,
        removedPermissions
      };
    } catch (error) {
      console.error('Error previewing package upgrade:', error);
      throw error;
    }
  }

  /**
   * Clean up expired permissions
   */
  async cleanupExpiredPermissions(): Promise<void> {
    try {
      const now = Timestamp.now();
      const expiredQuery = query(
        collection(db, this.collections.effectivePermissions),
        where('expiresAt', '<', now)
      );

      const snapshot = await getDocs(expiredQuery);
      const batch = writeBatch(db);

      snapshot.forEach(doc => {
        batch.delete(doc.ref);
      });

      await batch.commit();
      console.log(`Cleaned up ${snapshot.size} expired permissions`);
    } catch (error) {
      console.error('Error cleaning up expired permissions:', error);
    }
  }

  /**
   * Private helper to build user with all permissions
   */
  private async buildUserWithPermissions(userId: string, userData: any): Promise<UserWithPermissions> {
    try {
      // Get custom permissions
      const customPermQuery = query(
        collection(db, this.collections.customPermissions),
        where('userId', '==', userId),
        where('isActive', '==', true)
      );
      const customPermSnapshot = await getDocs(customPermQuery);
      const customPermissions: CustomPermission[] = [];

      customPermSnapshot.forEach(doc => {
        const data = doc.data();
        customPermissions.push({
          id: doc.id,
          userId: data.userId,
          featureId: data.featureId,
          permission: data.permission,
          grantedBy: data.grantedBy,
          grantedAt: data.grantedAt.toDate(),
          expiresAt: data.expiresAt?.toDate(),
          reason: data.reason,
          isActive: data.isActive
        });
      });

      // Get effective permissions
      const effectivePermissions = await this.getUserEffectivePermissions(userId);

      // Get package permissions for current tier
      const packageTier = userData.packageTier as PackageTier || PackageTier.FREE;
      const packagePermissions = buildPackagePermissions()[packageTier];

      return {
        id: userId,
        email: userData.email || '',
        name: userData.name || userData.displayName || '',
        displayName: userData.displayName,
        emailVerified: userData.emailVerified || false,
        disabled: userData.disabled || false,
        roles: userData.roles || [],
        groups: userData.groups || [],
        attachedPolicies: userData.attachedPolicies || [],
        status: userData.status || 'active',
        lastActivity: userData.lastActivity,
        createdAt: userData.createdAt?.toDate?.()?.toISOString() || userData.createdAt || new Date().toISOString(),
        updatedAt: userData.updatedAt?.toDate?.()?.toISOString() || userData.updatedAt || new Date().toISOString(),
        packageTier,
        subscriptionStatus: userData.subscriptionStatus || SubscriptionStatus.PENDING,
        lastPaymentDate: userData.lastPaymentDate?.toDate?.(),
        customPermissions,
        effectivePermissions,
        packagePermissions
      };
    } catch (error) {
      console.error('Error building user with permissions:', error);
      throw error;
    }
  }

  /**
   * Mock data for development when Firebase is not available
   */
  private getMockUsers(filters?: {
    packageTier?: PackageTier;
    subscriptionStatus?: string;
    hasCustomPermissions?: boolean;
  }): UserWithPermissions[] {
    const mockUsers: UserWithPermissions[] = [
      {
        id: 'mock-user-1',
        email: 'admin@epsx.com',
        name: 'Admin User',
        displayName: 'Admin User',
        emailVerified: true,
        disabled: false,
        roles: ['admin'],
        groups: [],
        attachedPolicies: [],
        status: 'active',
        lastActivity: new Date().toISOString(),
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        packageTier: PackageTier.ENTERPRISE,
        subscriptionStatus: SubscriptionStatus.ACTIVE,
        lastPaymentDate: new Date(),
        customPermissions: [],
        effectivePermissions: [],
        packagePermissions: buildPackagePermissions()[PackageTier.ENTERPRISE]
      },
      {
        id: 'mock-user-2',
        email: 'gold@epsx.com',
        name: 'Gold User',
        displayName: 'Gold User',
        emailVerified: true,
        disabled: false,
        roles: [],
        groups: [],
        attachedPolicies: [],
        status: 'active',
        lastActivity: new Date().toISOString(),
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        packageTier: PackageTier.GOLD,
        subscriptionStatus: SubscriptionStatus.ACTIVE,
        lastPaymentDate: new Date(),
        customPermissions: [],
        effectivePermissions: [],
        packagePermissions: buildPackagePermissions()[PackageTier.GOLD]
      },
      {
        id: 'mock-user-3',
        email: 'free@epsx.com',
        name: 'Free User',
        displayName: 'Free User',
        emailVerified: true,
        disabled: false,
        roles: [],
        groups: [],
        attachedPolicies: [],
        status: 'active',
        lastActivity: new Date().toISOString(),
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        packageTier: PackageTier.FREE,
        subscriptionStatus: SubscriptionStatus.PENDING,
        lastPaymentDate: undefined,
        customPermissions: [],
        effectivePermissions: [],
        packagePermissions: buildPackagePermissions()[PackageTier.FREE]
      }
    ];

    // Apply filters to mock data
    return mockUsers.filter(user => {
      if (filters?.packageTier && user.packageTier !== filters.packageTier) {
        return false;
      }
      if (filters?.subscriptionStatus && user.subscriptionStatus !== filters.subscriptionStatus) {
        return false;
      }
      if (filters?.hasCustomPermissions && user.customPermissions.length === 0) {
        return false;
      }
      return true;
    });
  }
}

export const firebaseIAMService = new FirebaseIAMService();
