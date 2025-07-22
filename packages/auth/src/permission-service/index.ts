// AWS IAM-inspired Permission Service
// Main exports for the permission service package

// Core service
export { PermissionService } from './PermissionService';

// Template integration
export { 
  TemplateIntegratedPermissionService,
  createTemplateIntegratedService 
} from './template-integration';
export type { 
  TemplateContext, 
  TemplateEvaluationResult 
} from './template-integration';

// Types
export type {
  Permission,
  Policy,
  PolicyStatement,
  Role,
  Group,
  UserPermissions,
  PermissionContext,
  PermissionEvaluationResult,
  ResourceArn,
  PermissionCondition,
  EvaluationOptions,
} from './types';

export {
  ResourceType,
  ActionType,
  SystemPolicy,
} from './types';

// Policy templates
export { PolicyTemplates } from './PolicyTemplates';

// React hooks
export {
  usePermission,
  usePermissions,
  useStockAnalyticsPermissions,
  useAdminPermissions,
  useBillingPermissions,
  useResourcePermissions,
  useTierAccess,
} from './hooks/usePermissions';

// Template-aware hooks
export {
  useTemplatePermissions,
  useEffectivePermissions,
  useResourcePermission,
  useTemplateManagement,
} from './hooks/useTemplatePermissions';

// React components
export {
  PermissionGate,
  MultiPermissionGate,
  TierGate,
  AdminGate,
  ConditionalPermission,
  withPermission,
  withTierAccess,
} from './components/PermissionGates';

// Template-aware components
export {
  TemplatePermissionGate,
  TemplateConflictWarning,
  ActiveTemplatesDisplay,
  EnhancedPermissionGate,
  withTemplatePermissions,
  PermissionDebugInfo,
} from './components/TemplatePermissionGates';

// Import types for utilities
import type { Policy, PolicyStatement } from './types';
import { PermissionService } from './PermissionService';

// Utilities
export const PermissionUtils = {
  buildResourceArn: (service: string, resourceType: string, resourceId: string, region?: string, accountId?: string) => {
    return `epsx:${service}:${region || ''}:${accountId || ''}:${resourceType}:${resourceId}`;
  },
  
  parseResourceArn: (arn: string) => {
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
  },
  
  validatePolicy: (policy: Policy) => {
    // Basic policy validation
    if (!policy.name || !policy.statement || !Array.isArray(policy.statement)) {
      return false;
    }
    
    return policy.statement.every((stmt: PolicyStatement) => 
      stmt.effect && 
      stmt.actions && 
      stmt.resources && 
      Array.isArray(stmt.actions) && 
      Array.isArray(stmt.resources)
    );
  },
};

// Default export
export default PermissionService;
