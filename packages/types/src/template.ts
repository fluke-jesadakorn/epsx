import { UserRole } from './auth/roles';

/**
 * Dynamic Template System Types
 * Extends the existing IAM system with custom template capabilities
 */

export interface TemplatePermission {
  /** Permission identifier using action:resource pattern */
  id: string;
  /** Human-readable name */
  name: string;
  /** Detailed description */
  description: string;
  /** Permission category for organization */
  category: PermissionCategory;
  /** Resource scope */
  scope: PermissionScope;
  /** Conditional restrictions */
  conditions?: PermissionCondition[];
}

export interface PermissionCondition {
  /** Condition type */
  type: 'usage_limit' | 'time_range' | 'ip_restriction' | 'company_scope' | 'custom';
  /** Condition parameters */
  params: Record<string, any>;
  /** Human-readable description */
  description: string;
}

export enum PermissionCategory {
  DASHBOARD = 'dashboard',
  API = 'api', 
  DATA = 'data',
  ADMIN = 'admin',
  ANALYTICS = 'analytics',
  INTEGRATION = 'integration',
  BILLING = 'billing',
  SUPPORT = 'support'
}

export enum PermissionScope {
  OWN = 'own',
  COMPANY = 'company',
  PARTNER = 'partner',
  GLOBAL = 'global'
}

export enum TemplateScope {
  SYSTEM = 'system',           // Built-in system templates
  ORGANIZATION = 'organization', // Org-wide custom templates
  PERSONAL = 'personal'        // Individual admin templates
}

export enum TemplateStatus {
  DRAFT = 'draft',
  ACTIVE = 'active',
  ARCHIVED = 'archived',
  DEPRECATED = 'deprecated'
}

export interface DynamicTemplate {
  /** Unique template identifier */
  id: string;
  
  /** Template metadata */
  name: string;
  description: string;
  version: number;
  
  /** Template permissions */
  permissions: TemplatePermission[];
  
  /** Template hierarchy */
  parentTemplate?: string;
  inheritanceMode: 'extend' | 'override' | 'merge';
  
  /** Template scope and access */
  scope: TemplateScope;
  status: TemplateStatus;
  
  /** Package tier compatibility */
  packageTierCompatibility: PackageTier[];
  minimumPackageTier?: PackageTier;
  
  /** Template constraints */
  validationRules: TemplateValidationRule[];
  conflictResolution: ConflictResolutionStrategy;
  
  /** Audit information */
  createdBy: string;
  createdAt: Date;
  updatedBy: string;
  updatedAt: Date;
  
  /** Usage tracking */
  usageCount: number;
  assignedUserCount: number;
  
  /** Template categories and tags */
  categories: string[];
  tags: string[];
  
  /** Template sharing */
  isPublic: boolean;
  sharedWith: string[]; // User IDs who can use this template
}

export interface TemplateValidationRule {
  /** Rule type */
  type: 'permission_conflict' | 'scope_escalation' | 'package_compatibility' | 'custom';
  /** Rule configuration */
  config: Record<string, any>;
  /** Error message for violations */
  errorMessage: string;
  /** Whether to block or warn */
  severity: 'error' | 'warning' | 'info';
}

export enum ConflictResolutionStrategy {
  FAIL = 'fail',              // Fail on conflicts
  MERGE_PERMISSIVE = 'merge_permissive', // Take more permissive
  MERGE_RESTRICTIVE = 'merge_restrictive', // Take more restrictive
  PRIORITIZE_EXPLICIT = 'prioritize_explicit', // Explicit permissions win
  CUSTOM = 'custom'           // Use custom resolution logic
}

export enum PackageTier {
  FREE = 'FREE',
  BRONZE = 'BRONZE', 
  SILVER = 'SILVER',
  GOLD = 'GOLD',
  PLATINUM = 'PLATINUM',
  ENTERPRISE = 'ENTERPRISE'
}

export interface TemplateBuilder {
  /** Current template being built */
  template: Partial<DynamicTemplate>;
  
  /** Available permissions to choose from */
  availablePermissions: TemplatePermission[];
  
  /** Template preview */
  preview: TemplatePreview;
  
  /** Validation results */
  validation: TemplateValidationResult;
}

export interface TemplatePreview {
  /** Effective permissions after inheritance and conflicts */
  effectivePermissions: TemplatePermission[];
  
  /** Permission conflicts detected */
  conflicts: PermissionConflict[];
  
  /** Inheritance chain */
  inheritanceChain: string[];
  
  /** Package compatibility check */
  packageCompatibility: PackageCompatibilityResult[];
}

export interface PermissionConflict {
  /** Conflicting permission IDs */
  permissionIds: string[];
  
  /** Conflict type */
  type: 'duplicate' | 'scope_mismatch' | 'condition_conflict' | 'inheritance_conflict';
  
  /** Conflict description */
  description: string;
  
  /** Suggested resolution */
  suggestedResolution: string;
  
  /** Severity */
  severity: 'error' | 'warning' | 'info';
}

export interface PackageCompatibilityResult {
  packageTier: PackageTier;
  compatible: boolean;
  issues: string[];
  suggestions: string[];
}

export interface TemplateValidationResult {
  /** Overall validation status */
  isValid: boolean;
  
  /** Validation errors that must be fixed */
  errors: TemplateValidationError[];
  
  /** Warnings that should be addressed */
  warnings: TemplateValidationError[];
  
  /** Informational messages */
  info: TemplateValidationError[];
}

export interface TemplateValidationError {
  /** Error code */
  code: string;
  
  /** Human-readable message */
  message: string;
  
  /** Field or section where error occurs */
  field?: string;
  
  /** Suggested fix */
  suggestion?: string;
}

export interface TemplateAssignment {
  /** Assignment ID */
  id: string;
  
  /** Template and user information */
  templateId: string;
  userId: string;
  
  /** Assignment metadata */
  assignedBy: string;
  assignedAt: Date;
  expiresAt?: Date;
  
  /** Assignment status */
  status: 'active' | 'expired' | 'revoked' | 'suspended';
  
  /** Assignment notes */
  notes?: string;
  
  /** Override permissions for this specific assignment */
  permissionOverrides?: TemplatePermission[];
}

export interface TemplateUsageStats {
  templateId: string;
  totalAssignments: number;
  activeAssignments: number;
  averageUsageDuration: number;
  popularPermissions: string[];
  conflictFrequency: number;
  userSatisfactionScore?: number;
}

/**
 * Template audit events
 */
export interface TemplateAuditEvent {
  id: string;
  eventType: TemplateAuditEventType;
  templateId: string;
  userId: string; // Who performed the action
  targetUserId?: string; // Who was affected (for assignments)
  timestamp: Date;
  details: Record<string, any>;
  changes?: TemplateChange[];
}

export enum TemplateAuditEventType {
  TEMPLATE_CREATED = 'template_created',
  TEMPLATE_UPDATED = 'template_updated',
  TEMPLATE_DELETED = 'template_deleted',
  TEMPLATE_PUBLISHED = 'template_published',
  TEMPLATE_ARCHIVED = 'template_archived',
  TEMPLATE_ASSIGNED = 'template_assigned',
  TEMPLATE_UNASSIGNED = 'template_unassigned',
  PERMISSION_OVERRIDE = 'permission_override'
}

export interface TemplateChange {
  field: string;
  oldValue: any;
  newValue: any;
  changeType: 'added' | 'removed' | 'modified';
}

/**
 * Service interfaces for template operations
 */
export interface TemplateService {
  // Template CRUD
  createTemplate(template: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>): Promise<DynamicTemplate>;
  updateTemplate(id: string, updates: Partial<DynamicTemplate>): Promise<DynamicTemplate>;
  deleteTemplate(id: string): Promise<void>;
  getTemplate(id: string): Promise<DynamicTemplate | null>;
  listTemplates(filters?: TemplateFilters): Promise<DynamicTemplate[]>;
  
  // Template validation
  validateTemplate(template: Partial<DynamicTemplate>): Promise<TemplateValidationResult>;
  previewTemplate(template: Partial<DynamicTemplate>): Promise<TemplatePreview>;
  
  // Template assignment
  assignTemplate(templateId: string, userId: string, options?: TemplateAssignmentOptions): Promise<TemplateAssignment>;
  unassignTemplate(templateId: string, userId: string): Promise<void>;
  getUserTemplates(userId: string): Promise<DynamicTemplate[]>;
  
  // Template analytics
  getTemplateStats(templateId: string): Promise<TemplateUsageStats>;
  getTemplateAuditLog(templateId: string, options?: AuditLogOptions): Promise<TemplateAuditEvent[]>;
}

export interface TemplateFilters {
  scope?: TemplateScope;
  status?: TemplateStatus;
  category?: string;
  tags?: string[];
  createdBy?: string;
  packageTier?: PackageTier;
  search?: string;
}

export interface TemplateAssignmentOptions {
  expiresAt?: Date;
  notes?: string;
  permissionOverrides?: TemplatePermission[];
  notifyUser?: boolean;
}

export interface AuditLogOptions {
  startDate?: Date;
  endDate?: Date;
  eventTypes?: TemplateAuditEventType[];
  limit?: number;
  offset?: number;
}

/**
 * Integration with existing auth system
 */
export interface ExtendedUserProfile {
  /** Existing user data */
  id: string;
  roles: UserRole[];
  
  /** Dynamic template assignments */
  templateAssignments: TemplateAssignment[];
  
  /** Computed effective permissions from all sources */
  effectivePermissions: TemplatePermission[];
  
  /** Last permissions computation timestamp */
  permissionsComputedAt: Date;
}