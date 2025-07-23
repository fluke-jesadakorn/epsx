import { db } from '@/lib/firebase-iam';
import { 
  collection, 
  query, 
  where, 
  getDocs, 
  doc, 
  getDoc,
  orderBy 
} from 'firebase/firestore';
import {
  DynamicTemplate,
  TemplateAssignment,
  TemplatePermission,
  PackageTier,
  PermissionScope,
  ConflictResolutionStrategy,
  TemplateStatus,
} from '@epsx/types';

export interface UserTemplateContext {
  userId: string;
  packageTier: PackageTier;
  staticPermissions: string[];
  roles: string[];
}

export interface EffectivePermissions {
  permissions: string[];
  templateSources: Array<{
    templateId: string;
    templateName: string;
    contributedPermissions: string[];
  }>;
  conflicts: Array<{
    permission: string;
    conflict: string;
    resolution: string;
  }>;
}

export class TemplateEvaluationService {
  private readonly collections = {
    templates: 'dynamic_templates',
    assignments: 'template_assignments',
    users: 'users',
  };

  /**
   * Evaluate all templates for a user and return effective permissions
   */
  async evaluateUserPermissions(context: UserTemplateContext): Promise<EffectivePermissions> {
    try {
      // Get user's active template assignments
      const assignments = await this.getUserActiveAssignments(context.userId);
      
      if (assignments.length === 0) {
        return {
          permissions: context.staticPermissions,
          templateSources: [],
          conflicts: [],
        };
      }

      // Fetch templates for assignments
      const templates = await this.getTemplatesForAssignments(assignments);
      
      // Filter templates by package compatibility
      const compatibleTemplates = templates.filter(template => 
        this.isTemplateCompatible(template, context.packageTier)
      );

      // Build inheritance chains and resolve all permissions
      const resolvedPermissions = await this.resolveTemplatePermissions(
        compatibleTemplates, 
        assignments,
        context
      );

      // Merge with static permissions
      const finalPermissions = this.mergePermissions(
        context.staticPermissions,
        resolvedPermissions.permissions,
        resolvedPermissions.conflicts
      );

      return {
        permissions: finalPermissions,
        templateSources: resolvedPermissions.sources,
        conflicts: resolvedPermissions.conflicts,
      };
    } catch (error) {
      console.error('Error evaluating user permissions:', error);
      // Fallback to static permissions on error
      return {
        permissions: context.staticPermissions,
        templateSources: [],
        conflicts: [{ 
          permission: '*', 
          conflict: 'Template evaluation failed', 
          resolution: 'Using static permissions only' 
        }],
      };
    }
  }

  /**
   * Check if user has specific permission (considering templates)
   */
  async hasPermission(
    context: UserTemplateContext, 
    requiredPermission: string
  ): Promise<boolean> {
    const effectivePermissions = await this.evaluateUserPermissions(context);
    
    // Check exact match
    if (effectivePermissions.permissions.includes(requiredPermission)) {
      return true;
    }

    // Check wildcard permissions (e.g., "admin.*" covers "admin.users.create")
    return effectivePermissions.permissions.some(permission => {
      if (permission.endsWith('.*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + '.');
      }
      return false;
    });
  }

  /**
   * Get template-aware route permissions for middleware
   */
  async getRoutePermissions(
    context: UserTemplateContext,
    route: string
  ): Promise<{ allowed: boolean; requiredPermission?: string; reason: string }> {
    // Define route-to-permission mapping
    const routePermissions: Record<string, string> = {
      '/dashboard': 'app.dashboard.view',
      '/analytics': 'analytics.view',
      '/my-data': 'data.own.view',
      '/admin': 'admin.access',
      '/users': 'admin.users.view',
      '/settings': 'settings.view',
      '/reports': 'reports.view',
    };

    const requiredPermission = this.getRequiredPermissionForRoute(route, routePermissions);
    
    if (!requiredPermission) {
      // Route doesn't require specific permission
      return { allowed: true, reason: 'Public route' };
    }

    const hasAccess = await this.hasPermission(context, requiredPermission);
    
    return {
      allowed: hasAccess,
      requiredPermission,
      reason: hasAccess 
        ? 'Permission granted via template or static permissions' 
        : `Missing required permission: ${requiredPermission}`,
    };
  }

  /**
   * Private helper methods
   */
  private async getUserActiveAssignments(userId: string): Promise<TemplateAssignment[]> {
    if (!db) return [];

    try {
      const assignmentQuery = query(
        collection(db, this.collections.assignments),
        where('userId', '==', userId),
        where('status', '==', 'active'),
        orderBy('assignedAt', 'desc')
      );

      const snapshot = await getDocs(assignmentQuery);
      const assignments: TemplateAssignment[] = [];

      snapshot.forEach((doc) => {
        const data = doc.data();
        
        // Check if assignment is not expired
        if (data.expiresAt && data.expiresAt.toDate() < new Date()) {
          return; // Skip expired assignments
        }

        assignments.push({
          id: doc.id,
          templateId: data.templateId,
          userId: data.userId,
          assignedBy: data.assignedBy,
          assignedAt: data.assignedAt.toDate(),
          status: data.status,
          expiresAt: data.expiresAt?.toDate(),
          notes: data.notes,
          permissionOverrides: data.permissionOverrides,
        });
      });

      return assignments;
    } catch (error) {
      console.error('Error getting user assignments:', error);
      return [];
    }
  }

  private async getTemplatesForAssignments(assignments: TemplateAssignment[]): Promise<DynamicTemplate[]> {
    if (!db || assignments.length === 0) return [];

    try {
      const templates: DynamicTemplate[] = [];
      
      // Fetch each template (in production, batch this)
      for (const assignment of assignments) {
        const templateDoc = await getDoc(doc(db, this.collections.templates, assignment.templateId));
        
        if (templateDoc.exists()) {
          const data = templateDoc.data();
          
          // Only include active templates
          if (data.status === TemplateStatus.ACTIVE) {
            templates.push(this.convertFirestoreTemplate(data, templateDoc.id));
          }
        }
      }

      return templates;
    } catch (error) {
      console.error('Error getting templates:', error);
      return [];
    }
  }

  private isTemplateCompatible(template: DynamicTemplate, userPackageTier: PackageTier): boolean {
    // Check package tier compatibility
    if (template.packageTierCompatibility.length > 0) {
      if (!template.packageTierCompatibility.includes(userPackageTier)) {
        return false;
      }
    }

    // Check minimum package tier
    if (template.minimumPackageTier) {
      const tierOrder = Object.values(PackageTier);
      const userTierIndex = tierOrder.indexOf(userPackageTier);
      const minimumTierIndex = tierOrder.indexOf(template.minimumPackageTier);
      
      if (userTierIndex < minimumTierIndex) {
        return false;
      }
    }

    return true;
  }

  private async resolveTemplatePermissions(
    templates: DynamicTemplate[],
    assignments: TemplateAssignment[],
    context: UserTemplateContext
  ): Promise<{
    permissions: string[];
    sources: Array<{ templateId: string; templateName: string; contributedPermissions: string[] }>;
    conflicts: Array<{ permission: string; conflict: string; resolution: string }>;
  }> {
    const allPermissions = new Set<string>();
    const sources: Array<{ templateId: string; templateName: string; contributedPermissions: string[] }> = [];
    const conflicts: Array<{ permission: string; conflict: string; resolution: string }> = [];
    const permissionSources = new Map<string, string[]>(); // permission -> template names

    for (const template of templates) {
      const assignment = assignments.find(a => a.templateId === template.id);
      if (!assignment) continue;

      // Resolve template inheritance if present
      const effectivePermissions = await this.resolveTemplateInheritance(template);
      
      // Apply assignment overrides
      const finalPermissions = this.applyAssignmentOverrides(
        effectivePermissions, 
        assignment.permissionOverrides
      );

      // Filter permissions by scope and user context
      const scopedPermissions = this.filterPermissionsByScope(finalPermissions, context);

      // Track sources and detect conflicts
      const contributedPermissions: string[] = [];
      
      for (const permission of scopedPermissions) {
        contributedPermissions.push(permission.id);
        
        // Track which templates provide each permission
        if (!permissionSources.has(permission.id)) {
          permissionSources.set(permission.id, []);
        }
        permissionSources.get(permission.id)!.push(template.name);
        
        // Check for conflicts
        if (allPermissions.has(permission.id)) {
          const existingSources = permissionSources.get(permission.id) || [];
          if (existingSources.length > 1) {
            const resolution = this.resolvePermissionConflict(
              permission.id,
              template.conflictResolution
            );
            
            conflicts.push({
              permission: permission.id,
              conflict: `Multiple templates provide this permission: ${existingSources.join(', ')}`,
              resolution,
            });
          }
        }
        
        allPermissions.add(permission.id);
      }

      sources.push({
        templateId: template.id,
        templateName: template.name,
        contributedPermissions,
      });
    }

    return {
      permissions: Array.from(allPermissions),
      sources,
      conflicts,
    };
  }

  private async resolveTemplateInheritance(template: DynamicTemplate): Promise<TemplatePermission[]> {
    const permissions = [...template.permissions];
    
    // If template has parent, resolve inheritance
    if (template.parentTemplate) {
      try {
        const parentDoc = await getDoc(doc(db!, this.collections.templates, template.parentTemplate));
        
        if (parentDoc.exists()) {
          const parentTemplate = this.convertFirestoreTemplate(parentDoc.data(), parentDoc.id);
          const parentPermissions = await this.resolveTemplateInheritance(parentTemplate);
          
          // Merge based on inheritance mode
          if (template.inheritanceMode === 'extend') {
            // Add parent permissions that aren't overridden
            for (const parentPerm of parentPermissions) {
              if (!permissions.some(p => p.id === parentPerm.id)) {
                permissions.push(parentPerm);
              }
            }
          } else if (template.inheritanceMode === 'override') {
            // Start with parent permissions, then override with child
            const basePermissions = [...parentPermissions];
            for (const childPerm of template.permissions) {
              const index = basePermissions.findIndex(p => p.id === childPerm.id);
              if (index >= 0) {
                basePermissions[index] = childPerm;
              } else {
                basePermissions.push(childPerm);
              }
            }
            return basePermissions;
          }
        }
      } catch (error) {
        console.error('Error resolving template inheritance:', error);
      }
    }
    
    return permissions;
  }

  private applyAssignmentOverrides(
    permissions: TemplatePermission[],
    overrides?: Record<string, any>
  ): TemplatePermission[] {
    if (!overrides) return permissions;

    return permissions.map(permission => {
      const override = overrides[permission.id];
      if (override) {
        return { ...permission, ...override };
      }
      return permission;
    });
  }

  private filterPermissionsByScope(
    permissions: TemplatePermission[],
    context: UserTemplateContext
  ): TemplatePermission[] {
    return permissions.filter(permission => {
      // Allow OWN scope for all users
      if (permission.scope === PermissionScope.OWN) return true;
      
      // COMPANY scope requires user to be part of organization
      if (permission.scope === PermissionScope.COMPANY) {
        // In real implementation, check user's organization membership
        return true; // Simplified for now
      }
      
      // PARTNER scope requires partner relationship
      if (permission.scope === PermissionScope.PARTNER) {
        // In real implementation, check partner relationships
        return context.packageTier !== PackageTier.FREE;
      }
      
      // GLOBAL scope requires admin permissions
      if (permission.scope === PermissionScope.GLOBAL) {
        return context.roles.includes('admin') || context.roles.includes('super_admin');
      }
      
      return true;
    });
  }

  private resolvePermissionConflict(
    permissionId: string,
    strategy: ConflictResolutionStrategy = ConflictResolutionStrategy.MERGE_PERMISSIVE
  ): string {
    switch (strategy) {
      case ConflictResolutionStrategy.MERGE_PERMISSIVE:
        return 'Granted - using most permissive version';
      case ConflictResolutionStrategy.MERGE_RESTRICTIVE:
        return 'Granted - using most restrictive version';
      case ConflictResolutionStrategy.FAIL:
        return 'Denied - conflict resolution set to fail';
      case ConflictResolutionStrategy.PRIORITIZE_EXPLICIT:
        return 'Granted - explicit permissions prioritized';
      case ConflictResolutionStrategy.CUSTOM:
        return 'Granted - custom resolution applied';
      default:
        return 'Granted - default resolution';
    }
  }

  private mergePermissions(
    staticPermissions: string[],
    templatePermissions: string[],
    conflicts: Array<{ permission: string; conflict: string; resolution: string }>
  ): string[] {
    const merged = new Set([...staticPermissions]);
    
    // Add template permissions, excluding those with failing conflicts
    for (const permission of templatePermissions) {
      const conflict = conflicts.find(c => c.permission === permission);
      if (!conflict || !conflict.resolution.includes('Denied')) {
        merged.add(permission);
      }
    }
    
    return Array.from(merged);
  }

  private getRequiredPermissionForRoute(
    route: string, 
    routePermissions: Record<string, string>
  ): string | null {
    // Check exact match first
    if (routePermissions[route]) {
      return routePermissions[route];
    }
    
    // Check for parent route patterns
    for (const [routePattern, permission] of Object.entries(routePermissions)) {
      if (route.startsWith(routePattern + '/')) {
        return permission;
      }
    }
    
    return null;
  }

  private convertFirestoreTemplate(data: any, id: string): DynamicTemplate {
    return {
      id,
      name: data.name,
      description: data.description,
      version: data.version,
      permissions: data.permissions || [],
      parentTemplate: data.parentTemplate,
      inheritanceMode: data.inheritanceMode || 'extend',
      scope: data.scope,
      status: data.status,
      packageTierCompatibility: data.packageTierCompatibility || [],
      minimumPackageTier: data.minimumPackageTier,
      validationRules: data.validationRules || [],
      conflictResolution: data.conflictResolution || ConflictResolutionStrategy.MERGE_PERMISSIVE,
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
}

export const templateEvaluationService = new TemplateEvaluationService();