import {
  DynamicTemplate,
  TemplatePermission,
  TemplateValidationResult,
  TemplateValidationError,
  PermissionConflict,
  TemplateScope,
  TemplateStatus,
  PackageTier,
  ConflictResolutionStrategy,
  PermissionCategory,
  PermissionScope,
} from '@epsx/types';

export class TemplateValidationService {
  /**
   * Comprehensive template validation
   */
  async validateTemplate(
    template: Partial<DynamicTemplate>,
    existingTemplates?: DynamicTemplate[]
  ): Promise<TemplateValidationResult> {
    const errors: TemplateValidationError[] = [];
    const warnings: TemplateValidationError[] = [];
    const info: TemplateValidationError[] = [];

    // Basic field validation
    this.validateBasicFields(template, errors);

    // Permission validation
    if (template.permissions) {
      this.validatePermissions(template.permissions, errors, warnings);
    }

    // Scope and security validation
    this.validateScopeAndSecurity(template, errors, warnings);

    // Package compatibility validation
    this.validatePackageCompatibility(template, warnings, info);

    // Template naming and uniqueness
    if (existingTemplates) {
      this.validateUniqueness(template, existingTemplates, errors, warnings);
    }

    // Inheritance validation
    if (template.parentTemplate) {
      this.validateInheritance(template, existingTemplates || [], errors, warnings);
    }

    // Business rule validation
    this.validateBusinessRules(template, warnings, info);

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  /**
   * Validate permissions for conflicts and issues
   */
  async validatePermissionConflicts(
    permissions: TemplatePermission[],
    parentPermissions?: TemplatePermission[]
  ): Promise<PermissionConflict[]> {
    const conflicts: PermissionConflict[] = [];

    // Check for duplicate permissions
    const duplicates = this.findDuplicatePermissions(permissions);
    if (duplicates.length > 0) {
      conflicts.push({
        permissionIds: duplicates,
        type: 'duplicate',
        description: 'Duplicate permissions found in template',
        suggestedResolution: 'Remove duplicate permissions or merge their conditions',
        severity: 'error',
      });
    }

    // Check for scope conflicts
    const scopeConflicts = this.findScopeConflicts(permissions);
    scopeConflicts.forEach(conflict => conflicts.push(conflict));

    // Check for inheritance conflicts
    if (parentPermissions) {
      const inheritanceConflicts = this.findInheritanceConflicts(permissions, parentPermissions);
      inheritanceConflicts.forEach(conflict => conflicts.push(conflict));
    }

    // Check for condition conflicts
    const conditionConflicts = this.findConditionConflicts(permissions);
    conditionConflicts.forEach(conflict => conflicts.push(conflict));

    return conflicts;
  }

  /**
   * Validate template against package tier requirements
   */
  validatePackageTierCompatibility(
    template: Partial<DynamicTemplate>,
    packageTier: PackageTier
  ): { compatible: boolean; issues: string[]; suggestions: string[] } {
    const issues: string[] = [];
    const suggestions: string[] = [];

    // Check if template explicitly supports this package tier
    if (
      template.packageTierCompatibility &&
      !template.packageTierCompatibility.includes(packageTier)
    ) {
      issues.push(`Template is not marked as compatible with ${packageTier} tier`);
      suggestions.push('Add this package tier to the compatibility list or upgrade your plan');
    }

    // Check minimum package tier requirement
    if (template.minimumPackageTier) {
      const tierOrder = Object.values(PackageTier);
      const currentTierIndex = tierOrder.indexOf(packageTier);
      const minimumTierIndex = tierOrder.indexOf(template.minimumPackageTier);

      if (currentTierIndex < minimumTierIndex) {
        issues.push(`Template requires minimum ${template.minimumPackageTier} tier, but user has ${packageTier}`);
        suggestions.push(`Upgrade to ${template.minimumPackageTier} or higher to use this template`);
      }
    }

    // Check permission restrictions based on package tier
    if (template.permissions) {
      const restrictedPermissions = this.getRestrictedPermissionsForTier(template.permissions, packageTier);
      if (restrictedPermissions.length > 0) {
        issues.push(`Template contains ${restrictedPermissions.length} permissions not available in ${packageTier} tier`);
        suggestions.push('These permissions will be ignored or require package upgrade');
      }
    }

    return {
      compatible: issues.length === 0,
      issues,
      suggestions,
    };
  }

  /**
   * Private validation methods
   */
  private validateBasicFields(template: Partial<DynamicTemplate>, errors: TemplateValidationError[]): void {
    // Name validation
    if (!template.name?.trim()) {
      errors.push({
        code: 'MISSING_NAME',
        message: 'Template name is required',
        field: 'name',
        suggestion: 'Provide a descriptive name for your template',
      });
    } else {
      if (template.name.length < 3) {
        errors.push({
          code: 'NAME_TOO_SHORT',
          message: 'Template name must be at least 3 characters long',
          field: 'name',
          suggestion: 'Use a more descriptive name',
        });
      }
      if (template.name.length > 100) {
        errors.push({
          code: 'NAME_TOO_LONG',
          message: 'Template name must be less than 100 characters',
          field: 'name',
          suggestion: 'Shorten the template name',
        });
      }
    }

    // Description validation
    if (!template.description?.trim()) {
      errors.push({
        code: 'MISSING_DESCRIPTION',
        message: 'Template description is required',
        field: 'description',
        suggestion: 'Provide a clear description of what this template is for',
      });
    } else if (template.description.length < 10) {
      errors.push({
        code: 'DESCRIPTION_TOO_SHORT',
        message: 'Template description should be at least 10 characters',
        field: 'description',
        suggestion: 'Provide a more detailed description',
      });
    }

    // Scope validation
    if (!template.scope) {
      errors.push({
        code: 'MISSING_SCOPE',
        message: 'Template scope is required',
        field: 'scope',
        suggestion: 'Select Personal, Organization, or System scope',
      });
    }

    // Status validation
    if (!template.status) {
      errors.push({
        code: 'MISSING_STATUS',
        message: 'Template status is required',
        field: 'status',
        suggestion: 'Set template status to Draft, Active, or Archived',
      });
    }
  }

  private validatePermissions(
    permissions: TemplatePermission[],
    errors: TemplateValidationError[],
    warnings: TemplateValidationError[]
  ): void {
    if (permissions.length === 0) {
      errors.push({
        code: 'NO_PERMISSIONS',
        message: 'At least one permission is required',
        field: 'permissions',
        suggestion: 'Select permissions from the available list',
      });
      return;
    }

    if (permissions.length > 100) {
      warnings.push({
        code: 'TOO_MANY_PERMISSIONS',
        message: 'Template has many permissions which may impact performance',
        field: 'permissions',
        suggestion: 'Consider breaking this into multiple specialized templates',
      });
    }

    // Validate individual permissions
    permissions.forEach((permission, index) => {
      if (!permission.id?.trim()) {
        errors.push({
          code: 'INVALID_PERMISSION_ID',
          message: `Permission at index ${index} has invalid ID`,
          field: `permissions[${index}].id`,
          suggestion: 'Ensure all permissions have valid IDs',
        });
      }

      if (!permission.name?.trim()) {
        errors.push({
          code: 'INVALID_PERMISSION_NAME',
          message: `Permission at index ${index} has invalid name`,
          field: `permissions[${index}].name`,
          suggestion: 'Ensure all permissions have descriptive names',
        });
      }

      // Validate permission conditions
      if (permission.conditions) {
        permission.conditions.forEach((condition, condIndex) => {
          if (!condition.type || !condition.params) {
            errors.push({
              code: 'INVALID_PERMISSION_CONDITION',
              message: `Permission "${permission.name}" has invalid condition at index ${condIndex}`,
              field: `permissions[${index}].conditions[${condIndex}]`,
              suggestion: 'Ensure all conditions have valid type and parameters',
            });
          }
        });
      }
    });

    // Check for permission category balance
    const categories = permissions.map(p => p.category);
    const categoryCount = categories.reduce((acc, category) => {
      acc[category] = (acc[category] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    if (Object.keys(categoryCount).length === 1 && permissions.length > 5) {
      warnings.push({
        code: 'SINGLE_CATEGORY_TEMPLATE',
        message: 'Template contains only permissions from a single category',
        field: 'permissions',
        suggestion: 'Consider if this template could be more specific or if other categories are needed',
      });
    }
  }

  private validateScopeAndSecurity(
    template: Partial<DynamicTemplate>,
    errors: TemplateValidationError[],
    warnings: TemplateValidationError[]
  ): void {
    // System scope validation
    if (template.scope === TemplateScope.SYSTEM) {
      if (template.createdBy !== 'system' && !template.createdBy?.includes('admin')) {
        errors.push({
          code: 'INVALID_SYSTEM_SCOPE',
          message: 'Only system administrators can create system-scoped templates',
          field: 'scope',
          suggestion: 'Use Organization or Personal scope instead',
        });
      }

      if (template.isPublic) {
        warnings.push({
          code: 'PUBLIC_SYSTEM_TEMPLATE',
          message: 'System templates are typically not public',
          field: 'isPublic',
          suggestion: 'Consider if this template should really be public',
        });
      }
    }

    // Public template validation
    if (template.isPublic && template.scope === TemplateScope.PERSONAL) {
      warnings.push({
        code: 'PUBLIC_PERSONAL_TEMPLATE',
        message: 'Personal templates are typically not public',
        field: 'isPublic',
        suggestion: 'Consider changing scope to Organization or making template private',
      });
    }

    // Sensitive permissions check
    if (template.permissions) {
      const sensitivePermissions = template.permissions.filter(p => 
        p.id.includes('admin') || 
        p.id.includes('delete') || 
        p.id.includes('manage') ||
        p.scope === PermissionScope.GLOBAL
      );

      if (sensitivePermissions.length > 0 && template.isPublic) {
        warnings.push({
          code: 'SENSITIVE_PERMISSIONS_PUBLIC',
          message: 'Template contains sensitive permissions but is marked as public',
          field: 'isPublic',
          suggestion: 'Consider making this template private or removing sensitive permissions',
        });
      }
    }
  }

  private validatePackageCompatibility(
    template: Partial<DynamicTemplate>,
    warnings: TemplateValidationError[],
    info: TemplateValidationError[]
  ): void {
    if (!template.packageTierCompatibility || template.packageTierCompatibility.length === 0) {
      warnings.push({
        code: 'NO_PACKAGE_COMPATIBILITY',
        message: 'Template has no package tier compatibility defined',
        field: 'packageTierCompatibility',
        suggestion: 'Select compatible package tiers to help users understand usage',
      });
    } else {
      // Check if template supports all tiers above minimum
      if (template.minimumPackageTier) {
        const tierOrder = Object.values(PackageTier);
        const minimumIndex = tierOrder.indexOf(template.minimumPackageTier);
        const supportedTiers = template.packageTierCompatibility;
        const expectedTiers = tierOrder.slice(minimumIndex);
        
        const missingTiers = expectedTiers.filter(tier => !supportedTiers.includes(tier));
        if (missingTiers.length > 0) {
          info.push({
            code: 'INCOMPLETE_TIER_SUPPORT',
            message: `Template could support additional tiers: ${missingTiers.join(', ')}`,
            field: 'packageTierCompatibility',
            suggestion: 'Consider adding support for higher tier packages',
          });
        }
      }
    }
  }

  private validateUniqueness(
    template: Partial<DynamicTemplate>,
    existingTemplates: DynamicTemplate[],
    errors: TemplateValidationError[],
    warnings: TemplateValidationError[]
  ): void {
    if (!template.name) return;

    // Check for exact name duplicates
    const duplicateName = existingTemplates.find(
      existing => existing.name.toLowerCase() === template.name!.toLowerCase() && existing.id !== template.id
    );

    if (duplicateName) {
      errors.push({
        code: 'DUPLICATE_NAME',
        message: 'A template with this name already exists',
        field: 'name',
        suggestion: 'Choose a unique name for your template',
      });
    }

    // Check for very similar names
    const similarNames = existingTemplates.filter(existing => {
      if (existing.id === template.id) return false;
      const similarity = this.calculateNameSimilarity(template.name!, existing.name);
      return similarity > 0.8;
    });

    if (similarNames.length > 0) {
      warnings.push({
        code: 'SIMILAR_NAME',
        message: 'Template name is very similar to existing templates',
        field: 'name',
        suggestion: `Consider a more distinctive name. Similar: ${similarNames.map(t => t.name).join(', ')}`,
      });
    }
  }

  private validateInheritance(
    template: Partial<DynamicTemplate>,
    existingTemplates: DynamicTemplate[],
    errors: TemplateValidationError[],
    warnings: TemplateValidationError[]
  ): void {
    if (!template.parentTemplate) return;

    const parentTemplate = existingTemplates.find(t => t.id === template.parentTemplate);
    
    if (!parentTemplate) {
      errors.push({
        code: 'PARENT_NOT_FOUND',
        message: 'Parent template not found',
        field: 'parentTemplate',
        suggestion: 'Select a valid parent template or remove inheritance',
      });
      return;
    }

    // Check for circular inheritance
    if (this.hasCircularInheritance(template, existingTemplates)) {
      errors.push({
        code: 'CIRCULAR_INHERITANCE',
        message: 'Circular inheritance detected',
        field: 'parentTemplate',
        suggestion: 'Remove circular inheritance by selecting a different parent',
      });
    }

    // Check inheritance depth
    const inheritanceDepth = this.calculateInheritanceDepth(template, existingTemplates);
    if (inheritanceDepth > 5) {
      warnings.push({
        code: 'DEEP_INHERITANCE',
        message: 'Template inheritance is very deep which may impact performance',
        field: 'parentTemplate',
        suggestion: 'Consider flattening the inheritance hierarchy',
      });
    }

    // Scope compatibility check
    if (parentTemplate.scope === TemplateScope.PERSONAL && template.scope === TemplateScope.ORGANIZATION) {
      warnings.push({
        code: 'SCOPE_INHERITANCE_MISMATCH',
        message: 'Inheriting from personal template in organization template',
        field: 'scope',
        suggestion: 'Consider if this inheritance makes sense for your use case',
      });
    }
  }

  private validateBusinessRules(
    template: Partial<DynamicTemplate>,
    warnings: TemplateValidationError[],
    info: TemplateValidationError[]
  ): void {
    // Check if template has too few permissions for its scope
    if (template.scope === TemplateScope.ORGANIZATION && template.permissions && template.permissions.length < 3) {
      info.push({
        code: 'FEW_PERMISSIONS_ORG_SCOPE',
        message: 'Organization templates typically have more permissions',
        field: 'permissions',
        suggestion: 'Consider if additional permissions would be useful for this template',
      });
    }

    // Check conflict resolution strategy appropriateness
    if (template.conflictResolution === ConflictResolutionStrategy.FAIL && template.parentTemplate) {
      warnings.push({
        code: 'STRICT_CONFLICT_WITH_INHERITANCE',
        message: 'Strict conflict resolution with inheritance may cause assignment failures',
        field: 'conflictResolution',
        suggestion: 'Consider using merge strategy when using inheritance',
      });
    }

    // Check tags and categories
    if (!template.tags || template.tags.length === 0) {
      info.push({
        code: 'NO_TAGS',
        message: 'Template has no tags for discoverability',
        field: 'tags',
        suggestion: 'Add relevant tags to help users find this template',
      });
    }

    if (!template.categories || template.categories.length === 0) {
      info.push({
        code: 'NO_CATEGORIES',
        message: 'Template has no categories for organization',
        field: 'categories',
        suggestion: 'Add categories to help organize templates',
      });
    }
  }

  // Helper methods
  private findDuplicatePermissions(permissions: TemplatePermission[]): string[] {
    const seen = new Set<string>();
    const duplicates = new Set<string>();

    permissions.forEach(permission => {
      if (seen.has(permission.id)) {
        duplicates.add(permission.id);
      } else {
        seen.add(permission.id);
      }
    });

    return Array.from(duplicates);
  }

  private findScopeConflicts(permissions: TemplatePermission[]): PermissionConflict[] {
    const conflicts: PermissionConflict[] = [];
    const scopeGroups = permissions.reduce((acc, permission) => {
      const base = permission.id?.split('.')[0]; // e.g., 'user' from 'user.create'
      if (base && !acc[base]) acc[base] = [];
      if (base) acc[base]!.push(permission);
      return acc;
    }, {} as Record<string, TemplatePermission[]>);

    Object.entries(scopeGroups).forEach(([base, groupPermissions]) => {
      const scopes = groupPermissions.map(p => p.scope);
      const uniqueScopes = new Set(scopes);
      
      if (uniqueScopes.size > 1 && scopes.includes(PermissionScope.GLOBAL)) {
        conflicts.push({
          permissionIds: groupPermissions.map(p => p.id),
          type: 'scope_mismatch',
          description: `Conflicting scopes in ${base} permissions - global scope conflicts with restricted scopes`,
          suggestedResolution: 'Use consistent scope or remove global scope permission',
          severity: 'warning',
        });
      }
    });

    return conflicts;
  }

  private findInheritanceConflicts(
    permissions: TemplatePermission[],
    parentPermissions: TemplatePermission[]
  ): PermissionConflict[] {
    const conflicts: PermissionConflict[] = [];
    
    permissions.forEach(permission => {
      const parentPermission = parentPermissions.find(p => p.id === permission.id);
      if (parentPermission && parentPermission.scope !== permission.scope) {
        conflicts.push({
          permissionIds: [permission.id],
          type: 'inheritance_conflict',
          description: `Permission ${permission.id} has different scope than parent template`,
          suggestedResolution: 'Align scopes or use permission overrides',
          severity: 'warning',
        });
      }
    });

    return conflicts;
  }

  private findConditionConflicts(permissions: TemplatePermission[]): PermissionConflict[] {
    const conflicts: PermissionConflict[] = [];
    
    permissions.forEach(permission => {
      if (permission.conditions && permission.conditions.length > 1) {
        // Check for conflicting conditions (simplified logic)
        const usageLimits = permission.conditions.filter(c => c.type === 'usage_limit');
        if (usageLimits.length > 1) {
          conflicts.push({
            permissionIds: [permission.id],
            type: 'condition_conflict',
            description: `Permission ${permission.id} has multiple usage limit conditions`,
            suggestedResolution: 'Combine or clarify usage limit conditions',
            severity: 'warning',
          });
        }
      }
    });

    return conflicts;
  }

  private getRestrictedPermissionsForTier(
    permissions: TemplatePermission[],
    packageTier: PackageTier
  ): TemplatePermission[] {
    // Simplified logic - in real implementation, this would check against package definitions
    const restrictedCategories = {
      [PackageTier.FREE]: [PermissionCategory.ADMIN, PermissionCategory.ANALYTICS],
      [PackageTier.BRONZE]: [PermissionCategory.ADMIN],
      [PackageTier.SILVER]: [],
      [PackageTier.GOLD]: [],
      [PackageTier.PLATINUM]: [],
      [PackageTier.ENTERPRISE]: [],
    };

    const restricted = restrictedCategories[packageTier] || [];
    return permissions.filter(p => {
      if (!p.category) return false;
      return restricted.some(category => category === p.category);
    });
  }

  private calculateNameSimilarity(name1: string, name2: string): number {
    // Simple Levenshtein distance calculation
    const matrix: number[][] = Array(name2.length + 1).fill(null).map(() => Array(name1.length + 1).fill(0));
    
    for (let i = 0; i <= name1.length; i++) matrix[0]![i] = i;
    for (let j = 0; j <= name2.length; j++) matrix[j]![0] = j;
    
    for (let j = 1; j <= name2.length; j++) {
      for (let i = 1; i <= name1.length; i++) {
        const indicator = name1[i - 1] === name2[j - 1] ? 0 : 1;
        matrix[j]![i] = Math.min(
          matrix[j]![i - 1]! + 1,
          matrix[j - 1]![i]! + 1,
          matrix[j - 1]![i - 1]! + indicator
        );
      }
    }
    
    const maxLength = Math.max(name1.length, name2.length);
    return 1 - (matrix[name2.length]![name1.length]! / maxLength);
  }

  private hasCircularInheritance(
    template: Partial<DynamicTemplate>,
    existingTemplates: DynamicTemplate[]
  ): boolean {
    const visited = new Set<string>();
    let current = template.parentTemplate;
    
    while (current && !visited.has(current)) {
      if (current === template.id) return true;
      visited.add(current);
      
      const parentTemplate = existingTemplates.find(t => t.id === current);
      current = parentTemplate?.parentTemplate;
    }
    
    return visited.has(current || '');
  }

  private calculateInheritanceDepth(
    template: Partial<DynamicTemplate>,
    existingTemplates: DynamicTemplate[]
  ): number {
    let depth = 0;
    let current = template.parentTemplate;
    
    while (current && depth < 10) { // Safety limit
      depth++;
      const parentTemplate = existingTemplates.find(t => t.id === current);
      current = parentTemplate?.parentTemplate;
    }
    
    return depth;
  }
}

export const templateValidationService = new TemplateValidationService();