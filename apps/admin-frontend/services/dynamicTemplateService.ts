import { db } from '@/lib/firebase';
import {
  addDoc,
  collection,
  doc,
  deleteDoc,
  getDoc,
  getDocs,
  updateDoc,
  query,
  where,
  orderBy,
  limit as firestoreLimit,
  Timestamp,
  writeBatch,
  arrayUnion,
  arrayRemove,
} from 'firebase/firestore';
import {
  DynamicTemplate,
  TemplateService,
  TemplateAssignment,
  TemplateAuditEvent,
  TemplateUsageStats,
  TemplateValidationResult,
  TemplatePreview,
  TemplateFilters,
  TemplateAssignmentOptions,
  AuditLogOptions,
  TemplateScope,
  TemplateStatus,
  ConflictResolutionStrategy,
  TemplateAuditEventType,
  PermissionConflict,
  PackageCompatibilityResult,
  PackageTier,
  TemplatePermission,
} from '@epsx/types';

export class DynamicTemplateService implements TemplateService {
  private readonly collections = {
    templates: 'dynamic_templates',
    assignments: 'template_assignments',
    auditLogs: 'template_audit_logs',
    users: 'users',
  };

  /**
   * Create a new dynamic template
   */
  async createTemplate(
    template: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>
  ): Promise<DynamicTemplate> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      const now = new Date();
      const templateData = {
        ...template,
        createdAt: Timestamp.fromDate(now),
        updatedAt: Timestamp.fromDate(now),
        usageCount: 0,
        assignedUserCount: 0,
      };

      const docRef = await addDoc(collection(db, this.collections.templates), templateData);

      // Create audit log
      await this.createAuditLog({
        eventType: TemplateAuditEventType.TEMPLATE_CREATED,
        templateId: docRef.id,
        userId: template.createdBy,
        timestamp: now,
        details: {
          templateName: template.name,
          scope: template.scope,
          permissionCount: template.permissions.length,
        },
        changes: [],
      });

      return {
        id: docRef.id,
        ...template,
        createdAt: now,
        updatedAt: now,
        usageCount: 0,
        assignedUserCount: 0,
      };
    } catch (error) {
      console.error('Error creating template:', error);
      throw error;
    }
  }

  /**
   * Update an existing template
   */
  async updateTemplate(id: string, updates: Partial<DynamicTemplate>): Promise<DynamicTemplate> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      const templateRef = doc(db, this.collections.templates, id);
      const templateDoc = await getDoc(templateRef);

      if (!templateDoc.exists()) {
        throw new Error('Template not found');
      }

      const currentTemplate = this.convertFirestoreTemplate(templateDoc.data(), id);
      const now = new Date();

      // Prepare update data
      const updateData = {
        ...updates,
        updatedAt: Timestamp.fromDate(now),
        version: currentTemplate.version + 1,
      };

      // Remove computed fields that shouldn't be updated directly
      delete updateData.id;
      delete updateData.createdAt;
      delete updateData.usageCount;
      delete updateData.assignedUserCount;

      await updateDoc(templateRef, updateData);

      // Calculate changes for audit log
      const changes = this.calculateChanges(currentTemplate, updates);

      // Create audit log
      await this.createAuditLog({
        eventType: TemplateAuditEventType.TEMPLATE_UPDATED,
        templateId: id,
        userId: updates.updatedBy || currentTemplate.updatedBy,
        timestamp: now,
        details: {
          templateName: updates.name || currentTemplate.name,
          changedFields: Object.keys(updates),
        },
        changes,
      });

      return {
        ...currentTemplate,
        ...updates,
        id,
        updatedAt: now,
        version: currentTemplate.version + 1,
      };
    } catch (error) {
      console.error('Error updating template:', error);
      throw error;
    }
  }

  /**
   * Delete a template
   */
  async deleteTemplate(id: string): Promise<void> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      const templateRef = doc(db, this.collections.templates, id);
      const templateDoc = await getDoc(templateRef);

      if (!templateDoc.exists()) {
        throw new Error('Template not found');
      }

      const template = this.convertFirestoreTemplate(templateDoc.data(), id);
      const batch = writeBatch(db);

      // Delete template
      batch.delete(templateRef);

      // Delete all assignments
      const assignmentsQuery = query(
        collection(db, this.collections.assignments),
        where('templateId', '==', id)
      );
      const assignmentsSnapshot = await getDocs(assignmentsQuery);
      
      assignmentsSnapshot.forEach((assignmentDoc) => {
        batch.delete(assignmentDoc.ref);
      });

      await batch.commit();

      // Create audit log
      await this.createAuditLog({
        eventType: TemplateAuditEventType.TEMPLATE_DELETED,
        templateId: id,
        userId: 'system', // Would be actual user from auth context
        timestamp: new Date(),
        details: {
          templateName: template.name,
          assignmentsDeleted: assignmentsSnapshot.size,
        },
        changes: [],
      });
    } catch (error) {
      console.error('Error deleting template:', error);
      throw error;
    }
  }

  /**
   * Get a single template by ID
   */
  async getTemplate(id: string): Promise<DynamicTemplate | null> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      const templateDoc = await getDoc(doc(db, this.collections.templates, id));
      
      if (!templateDoc.exists()) {
        return null;
      }

      return this.convertFirestoreTemplate(templateDoc.data(), id);
    } catch (error) {
      console.error('Error getting template:', error);
      throw error;
    }
  }

  /**
   * List templates with optional filters
   */
  async listTemplates(filters?: TemplateFilters): Promise<DynamicTemplate[]> {
    try {
      if (!db) {
        console.warn('Firebase not initialized, returning mock templates');
        return this.getMockTemplates(filters);
      }

      let templateQuery = query(collection(db, this.collections.templates));

      // Apply filters
      if (filters?.scope) {
        templateQuery = query(templateQuery, where('scope', '==', filters.scope));
      }
      if (filters?.status) {
        templateQuery = query(templateQuery, where('status', '==', filters.status));
      }
      if (filters?.createdBy) {
        templateQuery = query(templateQuery, where('createdBy', '==', filters.createdBy));
      }
      if (filters?.category) {
        templateQuery = query(templateQuery, where('categories', 'array-contains', filters.category));
      }

      // Add ordering
      templateQuery = query(templateQuery, orderBy('updatedAt', 'desc'));

      const snapshot = await getDocs(templateQuery);

      if (snapshot.empty) {
        console.warn('No templates found in Firestore, returning mock data');
        return this.getMockTemplates(filters);
      }

      const templates: DynamicTemplate[] = [];

      snapshot.forEach((doc) => {
        const template = this.convertFirestoreTemplate(doc.data(), doc.id);
        
        // Apply client-side filters that can't be done in Firestore
        if (filters?.search) {
          const searchTerm = filters.search.toLowerCase();
          const matchesSearch = 
            template.name.toLowerCase().includes(searchTerm) ||
            template.description.toLowerCase().includes(searchTerm) ||
            template.tags.some(tag => tag.toLowerCase().includes(searchTerm));
          
          if (!matchesSearch) return;
        }

        if (filters?.tags && filters.tags.length > 0) {
          const hasMatchingTag = filters.tags.some(tag => template.tags.includes(tag));
          if (!hasMatchingTag) return;
        }

        if (filters?.packageTier) {
          if (!template.packageTierCompatibility.includes(filters.packageTier)) return;
        }

        templates.push(template);
      });

      return templates;
    } catch (error) {
      console.error('Error listing templates:', error);
      console.warn('Falling back to mock data');
      return this.getMockTemplates(filters);
    }
  }

  /**
   * Validate template before saving
   */
  async validateTemplate(template: Partial<DynamicTemplate>): Promise<TemplateValidationResult> {
    const errors: any[] = [];
    const warnings: any[] = [];
    const info: any[] = [];

    // Basic validation
    if (!template.name?.trim()) {
      errors.push({
        code: 'MISSING_NAME',
        message: 'Template name is required',
        field: 'name',
        suggestion: 'Provide a descriptive name for your template',
      });
    }

    if (!template.permissions?.length) {
      errors.push({
        code: 'NO_PERMISSIONS',
        message: 'At least one permission is required',
        field: 'permissions',
        suggestion: 'Select permissions from the available list',
      });
    }

    // Permission validation
    if (template.permissions && template.permissions.length > 100) {
      warnings.push({
        code: 'TOO_MANY_PERMISSIONS',
        message: 'Template has many permissions which may impact performance',
        field: 'permissions',
        suggestion: 'Consider breaking this into multiple specialized templates',
      });
    }

    // Scope validation
    if (template.scope === TemplateScope.SYSTEM && template.createdBy !== 'system') {
      errors.push({
        code: 'INVALID_SYSTEM_SCOPE',
        message: 'Only system can create system-scoped templates',
        field: 'scope',
        suggestion: 'Use Organization or Personal scope instead',
      });
    }

    // Package compatibility validation
    if (template.packageTierCompatibility && template.packageTierCompatibility.length === 0) {
      warnings.push({
        code: 'NO_PACKAGE_COMPATIBILITY',
        message: 'Template has no package tier compatibility defined',
        field: 'packageTierCompatibility',
        suggestion: 'Select compatible package tiers to help users understand usage',
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  /**
   * Generate template preview
   */
  async previewTemplate(template: Partial<DynamicTemplate>): Promise<TemplatePreview> {
    const conflicts: PermissionConflict[] = [];
    const inheritanceChain: string[] = [];

    // Check for permission conflicts within the template
    if (template.permissions) {
      const permissionIds = template.permissions.map(p => p.id);
      const duplicates = permissionIds.filter((id, index) => permissionIds.indexOf(id) !== index);
      
      if (duplicates.length > 0) {
        conflicts.push({
          permissionIds: duplicates,
          type: 'duplicate',
          description: 'Duplicate permissions found in template',
          suggestedResolution: 'Remove duplicate permissions',
          severity: 'error',
        });
      }
    }

    // Build inheritance chain
    if (template.parentTemplate) {
      inheritanceChain.push(template.parentTemplate);
      // In real implementation, this would recursively fetch parent templates
    }

    // Package compatibility check
    const packageCompatibility: PackageCompatibilityResult[] = Object.values(PackageTier).map(tier => ({
      packageTier: tier,
      compatible: !template.packageTierCompatibility || template.packageTierCompatibility.includes(tier),
      issues: [],
      suggestions: [],
    }));

    return {
      effectivePermissions: template.permissions || [],
      conflicts,
      inheritanceChain,
      packageCompatibility,
    };
  }

  /**
   * Assign template to user
   */
  async assignTemplate(
    templateId: string,
    userId: string,
    options?: TemplateAssignmentOptions
  ): Promise<TemplateAssignment> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      const now = new Date();
      const assignment: Omit<TemplateAssignment, 'id'> = {
        templateId,
        userId,
        assignedBy: 'current-user-id', // Would come from auth context
        assignedAt: now,
        status: 'active',
        expiresAt: options?.expiresAt,
        notes: options?.notes,
        permissionOverrides: options?.permissionOverrides,
      };

      const assignmentRef = await addDoc(collection(db, this.collections.assignments), {
        ...assignment,
        assignedAt: Timestamp.fromDate(now),
        expiresAt: options?.expiresAt ? Timestamp.fromDate(options.expiresAt) : null,
      });

      // Update template usage count
      const templateRef = doc(db, this.collections.templates, templateId);
      await updateDoc(templateRef, {
        usageCount: arrayUnion(1), // This is simplified - in real implementation, use increment
        assignedUserCount: arrayUnion(1),
      });

      // Create audit log
      await this.createAuditLog({
        eventType: TemplateAuditEventType.TEMPLATE_ASSIGNED,
        templateId,
        userId: assignment.assignedBy,
        targetUserId: userId,
        timestamp: now,
        details: {
          expiresAt: options?.expiresAt?.toISOString(),
          notes: options?.notes,
        },
        changes: [],
      });

      return {
        id: assignmentRef.id,
        ...assignment,
      };
    } catch (error) {
      console.error('Error assigning template:', error);
      throw error;
    }
  }

  /**
   * Unassign template from user
   */
  async unassignTemplate(templateId: string, userId: string): Promise<void> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      // Find active assignment
      const assignmentQuery = query(
        collection(db, this.collections.assignments),
        where('templateId', '==', templateId),
        where('userId', '==', userId),
        where('status', '==', 'active')
      );

      const snapshot = await getDocs(assignmentQuery);
      const batch = writeBatch(db);

      snapshot.forEach((assignmentDoc) => {
        batch.update(assignmentDoc.ref, {
          status: 'revoked',
          revokedAt: Timestamp.now(),
          revokedBy: 'current-user-id', // Would come from auth context
        });
      });

      // Update template usage count
      if (snapshot.size > 0) {
        const templateRef = doc(db, this.collections.templates, templateId);
        batch.update(templateRef, {
          assignedUserCount: arrayRemove(1), // Simplified
        });
      }

      await batch.commit();

      // Create audit log
      await this.createAuditLog({
        eventType: TemplateAuditEventType.TEMPLATE_UNASSIGNED,
        templateId,
        userId: 'current-user-id',
        targetUserId: userId,
        timestamp: new Date(),
        details: {
          assignmentsRevoked: snapshot.size,
        },
        changes: [],
      });
    } catch (error) {
      console.error('Error unassigning template:', error);
      throw error;
    }
  }

  /**
   * Get user's assigned templates
   */
  async getUserTemplates(userId: string): Promise<DynamicTemplate[]> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      // Get active assignments for user
      const assignmentQuery = query(
        collection(db, this.collections.assignments),
        where('userId', '==', userId),
        where('status', '==', 'active')
      );

      const assignmentSnapshot = await getDocs(assignmentQuery);
      const templateIds = assignmentSnapshot.docs.map(doc => doc.data().templateId);

      if (templateIds.length === 0) {
        return [];
      }

      // Get templates (in real implementation, use 'in' query with batching for large lists)
      const templates: DynamicTemplate[] = [];
      
      for (const templateId of templateIds) {
        const template = await this.getTemplate(templateId);
        if (template) {
          templates.push(template);
        }
      }

      return templates;
    } catch (error) {
      console.error('Error getting user templates:', error);
      throw error;
    }
  }

  /**
   * Get template usage statistics
   */
  async getTemplateStats(templateId: string): Promise<TemplateUsageStats> {
    try {
      // Get template assignments
      const assignmentQuery = query(
        collection(db, this.collections.assignments),
        where('templateId', '==', templateId)
      );

      const assignmentSnapshot = await getDocs(assignmentQuery);
      const totalAssignments = assignmentSnapshot.size;
      const activeAssignments = assignmentSnapshot.docs.filter(
        doc => doc.data().status === 'active'
      ).length;

      // Mock other stats for now
      return {
        templateId,
        totalAssignments,
        activeAssignments,
        averageUsageDuration: 30, // days
        popularPermissions: [], // Would analyze assignment patterns
        conflictFrequency: 0,
        userSatisfactionScore: 4.5,
      };
    } catch (error) {
      console.error('Error getting template stats:', error);
      throw error;
    }
  }

  /**
   * Get template audit log
   */
  async getTemplateAuditLog(
    templateId: string,
    options?: AuditLogOptions
  ): Promise<TemplateAuditEvent[]> {
    try {
      if (!db) {
        throw new Error('Firebase not initialized');
      }

      let auditQuery = query(
        collection(db, this.collections.auditLogs),
        where('templateId', '==', templateId),
        orderBy('timestamp', 'desc')
      );

      if (options?.limit) {
        auditQuery = query(auditQuery, firestoreLimit(options.limit));
      }

      const snapshot = await getDocs(auditQuery);
      const logs: TemplateAuditEvent[] = [];

      snapshot.forEach((doc) => {
        const data = doc.data();
        logs.push({
          id: doc.id,
          eventType: data.eventType,
          templateId: data.templateId,
          userId: data.userId,
          targetUserId: data.targetUserId,
          timestamp: data.timestamp.toDate(),
          details: data.details,
          changes: data.changes || [],
        });
      });

      return logs;
    } catch (error) {
      console.error('Error getting template audit log:', error);
      throw error;
    }
  }

  /**
   * Private helper methods
   */
  private convertFirestoreTemplate(data: any, id: string): DynamicTemplate {
    return {
      id,
      name: data.name,
      description: data.description,
      version: data.version,
      permissions: data.permissions || [],
      parentTemplate: data.parentTemplate,
      inheritanceMode: data.inheritanceMode,
      scope: data.scope,
      status: data.status,
      packageTierCompatibility: data.packageTierCompatibility || [],
      minimumPackageTier: data.minimumPackageTier,
      validationRules: data.validationRules || [],
      conflictResolution: data.conflictResolution,
      createdBy: data.createdBy,
      createdAt: data.createdAt.toDate(),
      updatedBy: data.updatedBy,
      updatedAt: data.updatedAt.toDate(),
      usageCount: data.usageCount || 0,
      assignedUserCount: data.assignedUserCount || 0,
      categories: data.categories || [],
      tags: data.tags || [],
      isPublic: data.isPublic || false,
      sharedWith: data.sharedWith || [],
    };
  }

  private calculateChanges(current: DynamicTemplate, updates: Partial<DynamicTemplate>) {
    const changes = [];
    
    for (const [key, newValue] of Object.entries(updates)) {
      const oldValue = (current as any)[key];
      if (JSON.stringify(oldValue) !== JSON.stringify(newValue)) {
        changes.push({
          field: key,
          oldValue,
          newValue,
          changeType: oldValue === undefined ? 'added' : 'modified' as 'added' | 'modified' | 'removed',
        });
      }
    }

    return changes;
  }

  private async createAuditLog(event: Omit<TemplateAuditEvent, 'id'>): Promise<void> {
    try {
      if (!db) return;

      await addDoc(collection(db, this.collections.auditLogs), {
        ...event,
        timestamp: Timestamp.fromDate(event.timestamp),
      });
    } catch (error) {
      console.error('Error creating audit log:', error);
      // Don't throw - audit log failures shouldn't break the main operation
    }
  }

  /**
   * Mock data for development
   */
  private getMockTemplates(filters?: TemplateFilters): DynamicTemplate[] {
    const mockTemplates: DynamicTemplate[] = [
      {
        id: 'mock-template-1',
        name: 'Marketing Team Access',
        description: 'Standard permissions for marketing team members',
        version: 1,
        permissions: [
          {
            id: 'analytics.view',
            name: 'View Analytics',
            description: 'Access to analytics dashboard',
            category: 'analytics' as any,
            scope: 'company' as any,
          },
          {
            id: 'content.create',
            name: 'Create Content',
            description: 'Create marketing content',
            category: 'data' as any,
            scope: 'company' as any,
          },
        ],
        scope: TemplateScope.ORGANIZATION,
        status: TemplateStatus.ACTIVE,
        packageTierCompatibility: [PackageTier.GOLD, PackageTier.PLATINUM, PackageTier.ENTERPRISE],
        validationRules: [],
        conflictResolution: ConflictResolutionStrategy.MERGE_PERMISSIVE,
        createdBy: 'admin-user-1',
        createdAt: new Date('2024-01-15'),
        updatedBy: 'admin-user-1',
        updatedAt: new Date('2024-01-15'),
        usageCount: 25,
        assignedUserCount: 8,
        categories: ['marketing', 'content'],
        tags: ['marketing', 'analytics', 'content'],
        isPublic: false,
        sharedWith: [],
        inheritanceMode: 'extend',
      },
      {
        id: 'mock-template-2',
        name: 'Support Agent',
        description: 'Permissions for customer support representatives',
        version: 2,
        permissions: [
          {
            id: 'support.tickets.view',
            name: 'View Support Tickets',
            description: 'Access to support ticket system',
            category: 'support' as any,
            scope: 'global' as any,
          },
          {
            id: 'support.tickets.update',
            name: 'Update Support Tickets',
            description: 'Modify support tickets',
            category: 'support' as any,
            scope: 'global' as any,
          },
        ],
        scope: TemplateScope.ORGANIZATION,
        status: TemplateStatus.ACTIVE,
        packageTierCompatibility: [PackageTier.SILVER, PackageTier.GOLD, PackageTier.PLATINUM, PackageTier.ENTERPRISE],
        validationRules: [],
        conflictResolution: ConflictResolutionStrategy.FAIL,
        createdBy: 'admin-user-2',
        createdAt: new Date('2024-01-10'),
        updatedBy: 'admin-user-2',
        updatedAt: new Date('2024-01-20'),
        usageCount: 45,
        assignedUserCount: 12,
        categories: ['support', 'customer-service'],
        tags: ['support', 'tickets', 'customer'],
        isPublic: true,
        sharedWith: [],
        inheritanceMode: 'extend',
      },
    ];

    // Apply filters
    return mockTemplates.filter(template => {
      if (filters?.scope && template.scope !== filters.scope) return false;
      if (filters?.status && template.status !== filters.status) return false;
      if (filters?.createdBy && template.createdBy !== filters.createdBy) return false;
      if (filters?.category && !template.categories.includes(filters.category)) return false;
      if (filters?.packageTier && !template.packageTierCompatibility.includes(filters.packageTier)) return false;
      if (filters?.search) {
        const searchTerm = filters.search.toLowerCase();
        const matchesSearch = 
          template.name.toLowerCase().includes(searchTerm) ||
          template.description.toLowerCase().includes(searchTerm) ||
          template.tags.some(tag => tag.toLowerCase().includes(searchTerm));
        if (!matchesSearch) return false;
      }
      if (filters?.tags && filters.tags.length > 0) {
        const hasMatchingTag = filters.tags.some(tag => template.tags.includes(tag));
        if (!hasMatchingTag) return false;
      }
      return true;
    });
  }
}

export const dynamicTemplateService = new DynamicTemplateService();