import type { 
  Policy, 
  PolicyStatement, 
  UserPermissions,
  PermissionContext,
  PermissionEvaluationResult,
  ResourceArn,
  PermissionCondition,
  EvaluationOptions
} from './types';
import { 
  TemplateIntegratedPermissionService, 
  TemplateContext,
  TemplateEvaluationResult 
} from './template-integration';

export class PermissionService {
  private static instance: PermissionService;
  private permissionCache: Map<string, PermissionEvaluationResult> = new Map();
  private cacheTimeout = 5 * 60 * 1000; // 5 minutes
  private templateService: TemplateIntegratedPermissionService;

  private constructor() {
    this.templateService = new TemplateIntegratedPermissionService();
  }

  public static getInstance(): PermissionService {
    if (!PermissionService.instance) {
      PermissionService.instance = new PermissionService();
    }
    return PermissionService.instance;
  }

  /**
   * Main permission evaluation method
   */
  public async evaluatePermission(
    context: PermissionContext,
    options: EvaluationOptions = {}
  ): Promise<PermissionEvaluationResult> {
    const startTime = Date.now();
    
    try {
      // Check cache first
      const cacheKey = this.generateCacheKey(context);
      const cached = this.permissionCache.get(cacheKey);
      if (cached && (Date.now() - cached.context.timestamp.getTime()) < this.cacheTimeout) {
        return cached;
      }

      // Get user permissions
      const userPermissions = await this.getUserPermissions(context.userId);
      
      // Evaluate permissions
      const result = await this.evaluateUserPermissions(
        userPermissions,
        context,
        options
      );

      // Cache the result
      this.permissionCache.set(cacheKey, result);

      return result;
    } catch (error) {
      console.error('Permission evaluation error:', error);
      return {
        allowed: false,
        reason: 'Permission evaluation failed',
        matchedPolicies: [],
        evaluationTime: Date.now() - startTime,
        context,
      };
    }
  }

  /**
   * Evaluate user permissions against request
   */
  private async evaluateUserPermissions(
    userPermissions: UserPermissions,
    context: PermissionContext,
    _options: EvaluationOptions
  ): Promise<PermissionEvaluationResult> {
    const startTime = Date.now();
    const matchedPolicies: Policy[] = [];
    let finalDecision = false;
    let reason = 'No matching policies found';

    // Collect all policies (direct, role-based, group-based)
    const allPolicies = [
      ...userPermissions.directPolicies,
      ...userPermissions.roles.flatMap(role => role.policies),
      ...userPermissions.groups.flatMap(group => group.policies)
    ];

    // AWS IAM evaluation logic: Explicit Deny > Explicit Allow > Implicit Deny
    let hasExplicitDeny = false;
    let hasExplicitAllow = false;

    for (const policy of allPolicies) {
      for (const statement of policy.statement) {
        const matches = await this.evaluateStatement(statement, context);
        
        if (matches) {
          matchedPolicies.push(policy);
          
          if (statement.effect === 'Deny') {
            hasExplicitDeny = true;
            reason = `Explicit deny from policy: ${policy.name}`;
            break;
          } else if (statement.effect === 'Allow') {
            hasExplicitAllow = true;
            reason = `Explicit allow from policy: ${policy.name}`;
          }
        }
      }
      
      if (hasExplicitDeny) break;
    }

    // Apply AWS IAM decision logic
    if (hasExplicitDeny) {
      finalDecision = false;
    } else if (hasExplicitAllow) {
      finalDecision = true;
    } else {
      finalDecision = false;
      reason = 'Implicit deny - no matching allow statements';
    }

    return {
      allowed: finalDecision,
      reason,
      matchedPolicies,
      evaluationTime: Date.now() - startTime,
      context,
    };
  }

  /**
   * Evaluate a policy statement against context
   */
  private async evaluateStatement(
    statement: PolicyStatement,
    context: PermissionContext
  ): Promise<boolean> {
    // Check if action matches
    const actionMatches = this.matchesPattern(context.action, statement.actions);
    if (!actionMatches) return false;

    // Check if resource matches
    const resourceMatches = this.matchesPattern(context.resource, statement.resources);
    if (!resourceMatches) return false;

    // Evaluate conditions
    if (statement.conditions) {
      const conditionResult = await this.evaluateConditions(
        statement.conditions,
        context
      );
      if (!conditionResult) return false;
    }

    return true;
  }

  /**
   * Check if a value matches any pattern in the array
   */
  private matchesPattern(value: string, patterns: string[]): boolean {
    return patterns.some(pattern => {
      // Handle wildcards
      if (pattern === '*') return true;
      if (pattern.includes('*')) {
        const regex = new RegExp(pattern.replace(/\*/g, '.*'));
        return regex.test(value);
      }
      return value === pattern;
    });
  }

  /**
   * Evaluate conditions in a policy statement
   */
  private async evaluateConditions(
    conditions: Record<string, PermissionCondition>,
    context: PermissionContext
  ): Promise<boolean> {
    for (const [key, condition] of Object.entries(conditions)) {
      const contextValue = this.getContextValue(key, context);
      
      if (!this.evaluateCondition(condition, contextValue)) {
        return false;
      }
    }
    return true;
  }

  /**
   * Get context value for condition evaluation
   */
  private getContextValue(key: string, context: PermissionContext): any {
    switch (key) {
      case 'aws:RequestedRegion':
        return context.additionalContext?.region;
      case 'aws:CurrentTime':
        return context.timestamp;
      case 'aws:SourceIp':
        return context.ipAddress;
      case 'aws:UserAgent':
        return context.userAgent;
      case 'aws:userid':
        return context.userId;
      default:
        return context.additionalContext?.[key];
    }
  }

  /**
   * Evaluate a single condition
   */
  private evaluateCondition(condition: PermissionCondition, contextValue: any): boolean {
    switch (condition.operator) {
      case 'StringEquals':
        return contextValue === condition.value;
      case 'StringLike':
        const regex = new RegExp(condition.value.toString().replace(/\*/g, '.*'));
        return regex.test(contextValue);
      case 'NumericEquals':
        return Number(contextValue) === Number(condition.value);
      case 'NumericLessThan':
        return Number(contextValue) < Number(condition.value);
      case 'NumericGreaterThan':
        return Number(contextValue) > Number(condition.value);
      case 'DateEquals':
        return new Date(contextValue).getTime() === new Date(condition.value).getTime();
      case 'DateLessThan':
        return new Date(contextValue).getTime() < new Date(condition.value).getTime();
      case 'DateGreaterThan':
        return new Date(contextValue).getTime() > new Date(condition.value).getTime();
      case 'IpAddress':
        return this.isIpInRange(contextValue, condition.value.toString());
      case 'NotIpAddress':
        return !this.isIpInRange(contextValue, condition.value.toString());
      default:
        return false;
    }
  }

  /**
   * Check if IP address is in range
   */
  private isIpInRange(ip: string, range: string): boolean {
    // Simplified IP range check - in production, use a proper IP library
    if (range.includes('/')) {
      // CIDR notation
      return true; // Implement proper CIDR matching
    }
    return ip === range;
  }

  /**
   * Get user permissions from database (enhanced with templates)
   */
  private async getUserPermissions(userId: string): Promise<UserPermissions> {
    // Get base user permissions from database
    const basePermissions = await this.getBaseUserPermissions(userId);
    
    // Enhance with template-derived permissions
    const templateContext = await this.buildTemplateContext(userId);
    const enhancedPermissions = await this.templateService.enhanceUserPermissionsWithTemplates(
      basePermissions, 
      templateContext
    );
    
    return enhancedPermissions;
  }

  /**
   * Get base user permissions without templates
   */
  private async getBaseUserPermissions(userId: string): Promise<UserPermissions> {
    // This would connect to your database
    // For now, return mock data
    return {
      userId,
      directPolicies: [],
      roles: [],
      groups: [],
      effectivePermissions: [],
      lastEvaluated: new Date(),
    };
  }

  /**
   * Build template context for a user
   */
  private async buildTemplateContext(userId: string): Promise<TemplateContext> {
    // This would fetch user data from database
    // For now, return default context
    return {
      userId,
      packageTier: 'FREE' as any, // Would come from user data
      roles: [],
      staticPermissions: [],
      organizationId: undefined,
      partnerIds: [],
    };
  }

  /**
   * Generate cache key for permission evaluation
   */
  private generateCacheKey(context: PermissionContext): string {
    return `${context.userId}:${context.resource}:${context.action}:${context.timestamp.getTime()}`;
  }

  /**
   * Clear permission cache
   */
  public clearCache(): void {
    this.permissionCache.clear();
  }

  /**
   * Parse resource ARN
   */
  public parseResourceArn(arn: string): ResourceArn | null {
    const parts = arn.split(':');
    if (parts.length < 5) return null;

    return {
      partition: parts[0],
      service: parts[1],
      region: parts[2] || undefined,
      accountId: parts[3] || undefined,
      resourceType: parts[4],
      resourceId: parts[5] || '',
    };
  }

  /**
   * Build resource ARN
   */
  public buildResourceArn(
    service: string,
    resourceType: string,
    resourceId: string,
    region?: string,
    accountId?: string
  ): string {
    return `epsx:${service}:${region || ''}:${accountId || ''}:${resourceType}:${resourceId}`;
  }

  /**
   * Set template service for dynamic template integration
   */
  public setTemplateService(templateService: any): void {
    this.templateService = new TemplateIntegratedPermissionService(templateService);
  }

  /**
   * Evaluate templates for a user and get effective permissions
   */
  public async evaluateUserTemplates(userId: string): Promise<TemplateEvaluationResult> {
    const templateContext = await this.buildTemplateContext(userId);
    return await this.templateService.evaluateTemplatePermissions(templateContext);
  }

  /**
   * Check if user has permission (template-aware)
   */
  public async hasPermissionWithTemplates(
    userId: string,
    resource: string,
    action: string,
    additionalContext?: Record<string, any>
  ): Promise<boolean> {
    const context: PermissionContext = {
      userId,
      requestId: `req-${Date.now()}`,
      timestamp: new Date(),
      resource,
      action,
      additionalContext,
    };

    const result = await this.evaluatePermission(context);
    return result.allowed;
  }

  /**
   * Get effective permissions for a user including templates
   */
  public async getEffectivePermissions(userId: string): Promise<{
    staticPermissions: UserPermissions;
    templatePermissions: TemplateEvaluationResult;
    combinedPolicies: Policy[];
  }> {
    const staticPermissions = await this.getBaseUserPermissions(userId);
    const templatePermissions = await this.evaluateUserTemplates(userId);
    const templateContext = await this.buildTemplateContext(userId);
    
    // Get template-enhanced permissions
    const enhancedPermissions = await this.templateService.enhanceUserPermissionsWithTemplates(
      staticPermissions,
      templateContext
    );

    return {
      staticPermissions,
      templatePermissions,
      combinedPolicies: enhancedPermissions.directPolicies,
    };
  }

  /**
   * Refresh permission cache (including template cache)
   */
  public refreshCache(): void {
    this.clearCache();
    // Template service would also clear its cache if it had one
  }
}

export default PermissionService;
