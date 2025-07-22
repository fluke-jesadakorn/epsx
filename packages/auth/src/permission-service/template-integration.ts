import {
  DynamicTemplate,
  TemplateAssignment,
  TemplatePermission,
  PackageTier,
  PermissionScope,
  ConflictResolutionStrategy,
  TemplateStatus,
} from '@epsx/types';
import type { 
  Policy, 
  PolicyStatement, 
  UserPermissions,
  PermissionContext,
  PermissionEvaluationResult,
} from './types';

export interface TemplateContext {
  userId: string;
  packageTier: PackageTier;
  roles: string[];
  staticPermissions: string[];
  organizationId?: string;
  partnerIds?: string[];
}

export interface TemplateEvaluationResult {
  templatePermissions: string[];
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
  mergedWithStatic: boolean;
}

/**
 * Enhanced permission service that integrates dynamic templates
 * with the existing AWS IAM-inspired permission system
 */
export class TemplateIntegratedPermissionService {
  private templateService?: any; // Will be injected

  constructor(templateService?: any) {
    this.templateService = templateService;
  }

  /**
   * Convert dynamic templates to AWS IAM-style policies
   */
  async convertTemplatesToPolicies(
    templates: DynamicTemplate[],
    assignments: TemplateAssignment[],
    context: TemplateContext
  ): Promise<Policy[]> {
    const policies: Policy[] = [];

    for (const template of templates) {
      const assignment = assignments.find(a => a.templateId === template.id);
      if (!assignment) continue;

      // Check if template is compatible with user's package tier
      if (!this.isTemplateCompatible(template, context.packageTier)) {
        continue;
      }

      // Resolve template inheritance
      const effectivePermissions = await this.resolveTemplateInheritance(template);

      // Apply assignment overrides
      const finalPermissions = this.applyAssignmentOverrides(
        effectivePermissions,
        assignment.permissionOverrides
      );

      // Filter by scope
      const scopedPermissions = this.filterPermissionsByScope(finalPermissions, context);

      // Convert to AWS IAM-style policy
      const policy = this.convertTemplateToPolicy(template, scopedPermissions, assignment);
      policies.push(policy);
    }

    return policies;
  }

  /**
   * Enhance UserPermissions with template-derived policies
   */
  async enhanceUserPermissionsWithTemplates(
    basePermissions: UserPermissions,
    context: TemplateContext
  ): Promise<UserPermissions> {
    if (!this.templateService) {
      return basePermissions;
    }

    try {
      // Get user's active template assignments
      const assignments = await this.getUserActiveTemplateAssignments(context.userId);
      
      if (assignments.length === 0) {
        return basePermissions;
      }

      // Get templates for assignments
      const templates = await this.getTemplatesForAssignments(assignments);

      // Convert templates to policies
      const templatePolicies = await this.convertTemplatesToPolicies(
        templates,
        assignments,
        context
      );

      // Merge with existing permissions
      const enhancedPermissions: UserPermissions = {
        ...basePermissions,
        directPolicies: [
          ...basePermissions.directPolicies,
          ...templatePolicies,
        ],
        lastEvaluated: new Date(),
      };

      return enhancedPermissions;
    } catch (error) {
      console.error('Error enhancing permissions with templates:', error);
      return basePermissions;
    }
  }

  /**
   * Evaluate templates and get effective permissions
   */
  async evaluateTemplatePermissions(context: TemplateContext): Promise<TemplateEvaluationResult> {
    if (!this.templateService) {
      return {
        templatePermissions: [],
        templateSources: [],
        conflicts: [],
        mergedWithStatic: false,
      };
    }

    try {
      // Get user's template assignments
      const assignments = await this.getUserActiveTemplateAssignments(context.userId);
      const templates = await this.getTemplatesForAssignments(assignments);

      // Filter compatible templates
      const compatibleTemplates = templates.filter(template => 
        this.isTemplateCompatible(template, context.packageTier)
      );

      // Resolve all permissions from templates
      const result = await this.resolveAllTemplatePermissions(
        compatibleTemplates,
        assignments,
        context
      );

      return result;
    } catch (error) {
      console.error('Error evaluating template permissions:', error);
      return {
        templatePermissions: [],
        templateSources: [],
        conflicts: [{ 
          permission: '*', 
          conflict: 'Template evaluation failed', 
          resolution: 'Using fallback permissions' 
        }],
        mergedWithStatic: false,
      };
    }
  }

  /**
   * Private helper methods
   */
  private async getUserActiveTemplateAssignments(userId: string): Promise<TemplateAssignment[]> {
    // This would interface with the template service
    if (!this.templateService?.getUserActiveAssignments) {
      return [];
    }
    return await this.templateService.getUserActiveAssignments(userId);
  }

  private async getTemplatesForAssignments(assignments: TemplateAssignment[]): Promise<DynamicTemplate[]> {
    if (!this.templateService?.getTemplatesForAssignments) {
      return [];
    }
    return await this.templateService.getTemplatesForAssignments(assignments);
  }

  private isTemplateCompatible(template: DynamicTemplate, packageTier: PackageTier): boolean {
    // Check package compatibility
    if (template.packageTierCompatibility.length > 0) {
      if (!template.packageTierCompatibility.includes(packageTier)) {
        return false;
      }
    }

    // Check minimum package tier
    if (template.minimumPackageTier) {
      const tierOrder = Object.values(PackageTier);
      const userTierIndex = tierOrder.indexOf(packageTier);
      const minimumTierIndex = tierOrder.indexOf(template.minimumPackageTier);
      
      if (userTierIndex < minimumTierIndex) {
        return false;
      }
    }

    // Check if template is active
    return template.status === TemplateStatus.ACTIVE;
  }

  private async resolveTemplateInheritance(template: DynamicTemplate): Promise<TemplatePermission[]> {
    const permissions = [...template.permissions];
    
    // If template has parent, resolve inheritance recursively
    if (template.parentTemplate && this.templateService?.getTemplate) {
      try {
        const parentTemplate = await this.templateService.getTemplate(template.parentTemplate);
        
        if (parentTemplate) {
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
    context: TemplateContext
  ): TemplatePermission[] {
    return permissions.filter(permission => {
      switch (permission.scope) {
        case PermissionScope.OWN:
          return true; // Always allowed
        
        case PermissionScope.COMPANY:
          return !!context.organizationId; // User must be part of organization
        
        case PermissionScope.PARTNER:
          return context.partnerIds && context.partnerIds.length > 0;
        
        case PermissionScope.GLOBAL:
          return context.roles.includes('admin') || context.roles.includes('super_admin');
        
        default:
          return true;
      }
    });
  }

  private convertTemplateToPolicy(
    template: DynamicTemplate,
    permissions: TemplatePermission[],
    assignment: TemplateAssignment
  ): Policy {
    const statements: PolicyStatement[] = permissions.map(permission => ({
      sid: `Template-${template.id}-${permission.id}`,
      effect: 'Allow',
      actions: [permission.id],
      resources: this.buildResourcesForPermission(permission),
      conditions: permission.conditions ? this.convertConditions(permission.conditions) : undefined,
    }));

    return {
      id: `template-policy-${template.id}`,
      name: `Template: ${template.name}`,
      description: `Auto-generated policy from template: ${template.description}`,
      version: template.version.toString(),
      statement: statements,
      createdAt: assignment.assignedAt,
      updatedAt: template.updatedAt,
    };
  }

  private buildResourcesForPermission(permission: TemplatePermission): string[] {
    // Convert template permission to AWS IAM-style resource ARNs
    const baseResource = permission.id.replace(/\./g, ':');
    
    switch (permission.scope) {
      case PermissionScope.OWN:
        return [`epsx:${baseResource}:own:*`];
      case PermissionScope.COMPANY:
        return [`epsx:${baseResource}:company:*`];
      case PermissionScope.PARTNER:
        return [`epsx:${baseResource}:partner:*`];
      case PermissionScope.GLOBAL:
        return [`epsx:${baseResource}:*:*`];
      default:
        return [`epsx:${baseResource}:*:*`];
    }
  }

  private convertConditions(templateConditions: any[]): Record<string, any> {
    const conditions: Record<string, any> = {};
    
    templateConditions.forEach((condition, index) => {
      conditions[`TemplateCondition${index}`] = {
        key: condition.type,
        operator: 'StringEquals', // Simplified mapping
        value: condition.params,
      };
    });
    
    return conditions;
  }

  private async resolveAllTemplatePermissions(
    templates: DynamicTemplate[],
    assignments: TemplateAssignment[],
    context: TemplateContext
  ): Promise<TemplateEvaluationResult> {
    const allPermissions = new Set<string>();
    const sources: Array<{ templateId: string; templateName: string; contributedPermissions: string[] }> = [];
    const conflicts: Array<{ permission: string; conflict: string; resolution: string }> = [];
    const permissionSources = new Map<string, string[]>();

    for (const template of templates) {
      const assignment = assignments.find(a => a.templateId === template.id);
      if (!assignment) continue;

      // Resolve inheritance and get effective permissions
      const effectivePermissions = await this.resolveTemplateInheritance(template);
      const finalPermissions = this.applyAssignmentOverrides(
        effectivePermissions,
        assignment.permissionOverrides
      );
      const scopedPermissions = this.filterPermissionsByScope(finalPermissions, context);

      const contributedPermissions: string[] = [];

      for (const permission of scopedPermissions) {
        contributedPermissions.push(permission.id);

        // Track permission sources
        if (!permissionSources.has(permission.id)) {
          permissionSources.set(permission.id, []);
        }
        permissionSources.get(permission.id)!.push(template.name);

        // Detect conflicts
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
      templatePermissions: Array.from(allPermissions),
      templateSources: sources,
      conflicts,
      mergedWithStatic: true,
    };
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
      case ConflictResolutionStrategy.FIRST_WINS:
        return 'Granted - first template assignment wins';
      case ConflictResolutionStrategy.LAST_WINS:
        return 'Granted - last template assignment wins';
      default:
        return 'Granted - default resolution';
    }
  }
}

/**
 * Factory function to create template-integrated permission service
 */
export function createTemplateIntegratedService(templateService?: any): TemplateIntegratedPermissionService {
  return new TemplateIntegratedPermissionService(templateService);
}

// Export types for use in other modules
export type {
  TemplateContext,
  TemplateEvaluationResult,
};