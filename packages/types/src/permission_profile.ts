import type { UserRole } from './auth/roles';
import { StockRankingType } from './domains/analytics';

/**
 * Dynamic Permission Profile System Types
 * Extends the existing IAM system with custom permission profile capabilities
 */

export interface PermissionProfilePermission {
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
  SUPPORT = 'support',
  STOCK_RANKING = 'stock_ranking'
}

export enum PermissionScope {
  OWN = 'own',
  COMPANY = 'company',
  PARTNER = 'partner',
  GLOBAL = 'global'
}

export enum PermissionProfileScope {
  SYSTEM = 'system',           // Built-in system permission profiles
  ORGANIZATION = 'organization', // Org-wide custom permission profiles
  PERSONAL = 'personal'        // Individual admin permission profiles
}

export enum PermissionProfileStatus {
  DRAFT = 'draft',
  ACTIVE = 'active',
  ARCHIVED = 'archived',
  DEPRECATED = 'deprecated'
}

export interface DynamicPermissionProfile {
  /** Unique permission profile identifier */
  id: string;
  
  /** Permission profile metadata */
  name: string;
  description: string;
  version: number;
  
  /** Permission profile permissions */
  permissions: PermissionProfilePermission[];
  
  /** Permission profile hierarchy */
  parentPermissionProfile?: string;
  inheritanceMode: 'extend' | 'override' | 'merge';
  
  /** Permission profile scope and access */
  scope: PermissionProfileScope;
  status: PermissionProfileStatus;
  
  /** Package tier compatibility */
  packageTierCompatibility: PackageTier[];
  minimumPackageTier?: PackageTier;
  
  /** Stock ranking configuration */
  stockRankingConfig?: StockRankingConfig;
  
  /** Permission profile constraints */
  validationRules: PermissionProfileValidationRule[];
  conflictResolution: ConflictResolutionStrategy;
  
  /** Audit information */
  createdBy: string;
  createdAt: Date;
  updatedBy: string;
  updatedAt: Date;
  
  /** Usage tracking */
  usageCount: number;
  assignedUserCount: number;
  
  /** Permission profile categories and tags */
  categories: string[];
  tags: string[];
  
  /** Permission profile sharing */
  isPublic: boolean;
  sharedWith: string[]; // User IDs who can use this permission profile
}

export interface PermissionProfileValidationRule {
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

export interface StockRankingConfig {
  /** Maximum number of rankings visible per request */
  maxRankings: number;
  /** Allowed ranking types */
  allowedRankingTypes: StockRankingType[];
  /** API rate limits per minute */
  rateLimitPerMinute: number;
  /** Whether real-time updates are enabled */
  realTimeUpdates: boolean;
  /** Available markets */
  allowedMarkets: string[];
  /** Advanced features access */
  advancedFeatures: StockRankingAdvancedFeatures;
  /** Export capabilities */
  exportOptions: StockRankingExportOptions;
}


export interface StockRankingAdvancedFeatures {
  /** Custom ranking algorithms */
  customAlgorithms: boolean;
  /** AI-powered insights */
  aiInsights: boolean;
  /** Pattern recognition */
  patternRecognition: boolean;
  /** Historical data access */
  historicalData: boolean;
  /** Alert system */
  alertSystem: boolean;
  /** Portfolio tracking */
  portfolioTracking: boolean;
}

export interface StockRankingExportOptions {
  /** CSV export */
  csv: boolean;
  /** Excel export */
  excel: boolean;
  /** PDF reports */
  pdf: boolean;
  /** API data export */
  apiExport: boolean;
  /** Maximum exports per day */
  maxExportsPerDay: number;
}

export interface PermissionProfileBuilder {
  /** Current permission profile being built */
  permissionProfile: Partial<DynamicPermissionProfile>;
  
  /** Available permissions to choose from */
  availablePermissions: PermissionProfilePermission[];
  
  /** Permission profile preview */
  preview: PermissionProfilePreview;
  
  /** Validation results */
  validation: PermissionProfileValidationResult;
}

export interface PermissionProfilePreview {
  /** Effective permissions after inheritance and conflicts */
  effectivePermissions: PermissionProfilePermission[];
  
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

export interface PermissionProfileValidationResult {
  /** Overall validation status */
  isValid: boolean;
  
  /** Validation errors that must be fixed */
  errors: PermissionProfileValidationError[];
  
  /** Warnings that should be addressed */
  warnings: PermissionProfileValidationError[];
  
  /** Informational messages */
  info: PermissionProfileValidationError[];
}

export interface PermissionProfileValidationError {
  /** Error code */
  code: string;
  
  /** Human-readable message */
  message: string;
  
  /** Field or section where error occurs */
  field?: string;
  
  /** Suggested fix */
  suggestion?: string;
}

export interface PermissionProfileAssignment {
  /** Assignment ID */
  id: string;
  
  /** Permission profile and user information */
  permissionProfileId: string;
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
  permissionOverrides?: PermissionProfilePermission[];
}

export interface PermissionProfileUsageStats {
  permissionProfileId: string;
  totalAssignments: number;
  activeAssignments: number;
  averageUsageDuration: number;
  popularPermissions: string[];
  conflictFrequency: number;
  userSatisfactionScore?: number;
}

/**
 * Permission profile audit events
 */
export interface PermissionProfileAuditEvent {
  id: string;
  eventType: PermissionProfileAuditEventType;
  permissionProfileId: string;
  userId: string; // Who performed the action
  targetUserId?: string; // Who was affected (for assignments)
  timestamp: Date;
  details: Record<string, any>;
  changes?: PermissionProfileChange[];
}

export enum PermissionProfileAuditEventType {
  PERMISSION_PROFILE_CREATED = 'permission_profile_created',
  PERMISSION_PROFILE_UPDATED = 'permission_profile_updated',
  PERMISSION_PROFILE_DELETED = 'permission_profile_deleted',
  PERMISSION_PROFILE_PUBLISHED = 'permission_profile_published',
  PERMISSION_PROFILE_ARCHIVED = 'permission_profile_archived',
  PERMISSION_PROFILE_ASSIGNED = 'permission_profile_assigned',
  PERMISSION_PROFILE_UNASSIGNED = 'permission_profile_unassigned',
  PERMISSION_OVERRIDE = 'permission_override'
}

export interface PermissionProfileChange {
  field: string;
  oldValue: any;
  newValue: any;
  changeType: 'added' | 'removed' | 'modified';
}

/**
 * Service interfaces for permission profile operations
 */
export interface PermissionProfileService {
  // Permission profile CRUD
  createPermissionProfile(permissionProfile: Omit<DynamicPermissionProfile, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>): Promise<DynamicPermissionProfile>;
  updatePermissionProfile(id: string, updates: Partial<DynamicPermissionProfile>): Promise<DynamicPermissionProfile>;
  deletePermissionProfile(id: string): Promise<void>;
  getPermissionProfile(id: string): Promise<DynamicPermissionProfile | null>;
  listPermissionProfiles(filters?: PermissionProfileFilters): Promise<DynamicPermissionProfile[]>;
  
  // Permission profile validation
  validatePermissionProfile(permissionProfile: Partial<DynamicPermissionProfile>): Promise<PermissionProfileValidationResult>;
  previewPermissionProfile(permissionProfile: Partial<DynamicPermissionProfile>): Promise<PermissionProfilePreview>;
  
  // Permission profile assignment
  assignPermissionProfile(permissionProfileId: string, userId: string, options?: PermissionProfileAssignmentOptions): Promise<PermissionProfileAssignment>;
  unassignPermissionProfile(permissionProfileId: string, userId: string): Promise<void>;
  getUserPermissionProfiles(userId: string): Promise<DynamicPermissionProfile[]>;
  
  // Permission profile analytics
  getPermissionProfileStats(permissionProfileId: string): Promise<PermissionProfileUsageStats>;
  getPermissionProfileAuditLog(permissionProfileId: string, options?: AuditLogOptions): Promise<PermissionProfileAuditEvent[]>;
}

export interface PermissionProfileFilters {
  scope?: PermissionProfileScope;
  status?: PermissionProfileStatus;
  category?: string;
  tags?: string[];
  createdBy?: string;
  packageTier?: PackageTier;
  search?: string;
}

export interface PermissionProfileAssignmentOptions {
  expiresAt?: Date;
  notes?: string;
  permissionOverrides?: PermissionProfilePermission[];
  notifyUser?: boolean;
}

export interface AuditLogOptions {
  startDate?: Date;
  endDate?: Date;
  eventTypes?: PermissionProfileAuditEventType[];
  limit?: number;
  offset?: number;
}

/**
 * Integration with existing auth system
 */
/**
 * Stock Ranking Package Assignment Types
 */
export interface StockRankingPackageAssignment {
  /** Assignment ID */
  id: string;
  
  /** Package assignment information */
  userId: string;
  packageTier: PackageTier;
  permissionProfileId: string;
  
  /** Stock ranking configuration */
  stockRankingConfig: StockRankingConfig;
  
  /** Assignment metadata */
  assignedBy: string;
  assignedAt: Date;
  expiresAt?: Date;
  
  /** Assignment status */
  status: 'active' | 'expired' | 'revoked' | 'suspended';
  
  /** Assignment reason and notes */
  reason: string;
  notes?: string;
  
  /** Usage tracking */
  usageStats: StockRankingUsageStats;
}

export interface StockRankingUsageStats {
  /** API calls made */
  apiCallsUsed: number;
  /** Last API call timestamp */
  lastApiCall?: Date;
  /** Rankings viewed */
  rankingsViewed: number;
  /** Exports used */
  exportsUsed: number;
  /** Features accessed */
  featuresAccessed: string[];
}

export interface BulkStockRankingAssignment {
  /** Target user IDs */
  userIds: string[];
  /** Package tier to assign */
  packageTier: PackageTier;
  /** Permission profile to use */
  permissionProfileId: string;
  /** Assignment reason */
  reason: string;
  /** Optional expiration */
  expiresAt?: Date;
  /** Admin performing assignment */
  assignedBy: string;
  /** Notification settings */
  notifyUsers: boolean;
}

export interface BulkStockRankingAssignmentResult {
  /** Successfully assigned users */
  successful: string[];
  /** Failed assignments with reasons */
  failed: Array<{
    userId: string;
    reason: string;
  }>;
  /** Assignment summary */
  summary: {
    totalRequested: number;
    successful: number;
    failed: number;
    packageTier: PackageTier;
    assignmentTime: Date;
  };
}

export interface ExtendedUserProfile {
  /** Existing user data */
  id: string;
  roles: UserRole[];
  
  /** Dynamic permission profile assignments */
  permissionProfileAssignments: PermissionProfileAssignment[];
  
  /** Stock ranking package assignments */
  stockRankingAssignments: StockRankingPackageAssignment[];
  
  /** Computed effective permissions from all sources */
  effectivePermissions: PermissionProfilePermission[];
  
  /** Last permissions computation timestamp */
  permissionsComputedAt: Date;
}

/**
 * Predefined Stock Ranking Configurations
 */
export class StockRankingPackageConfigs {
  static readonly BRONZE: StockRankingConfig = {
    maxRankings: 5,
    allowedRankingTypes: [
      StockRankingType.EPS_GROWTH,
      StockRankingType.MARKET_CAP,
      StockRankingType.VOLUME
    ],
    rateLimitPerMinute: 10,
    realTimeUpdates: false,
    allowedMarkets: ['NYSE', 'NASDAQ'],
    advancedFeatures: {
      customAlgorithms: false,
      aiInsights: false,
      patternRecognition: false,
      historicalData: false,
      alertSystem: false,
      portfolioTracking: false
    },
    exportOptions: {
      csv: true,
      excel: false,
      pdf: false,
      apiExport: false,
      maxExportsPerDay: 3
    }
  };

  static readonly SILVER: StockRankingConfig = {
    maxRankings: 25,
    allowedRankingTypes: [
      StockRankingType.EPS_GROWTH,
      StockRankingType.MARKET_CAP,
      StockRankingType.VOLUME,
      StockRankingType.PRICE_CHANGE,
      StockRankingType.TECHNICAL_INDICATORS
    ],
    rateLimitPerMinute: 50,
    realTimeUpdates: true,
    allowedMarkets: ['NYSE', 'NASDAQ', 'AMEX'],
    advancedFeatures: {
      customAlgorithms: false,
      aiInsights: true,
      patternRecognition: false,
      historicalData: true,
      alertSystem: true,
      portfolioTracking: false
    },
    exportOptions: {
      csv: true,
      excel: true,
      pdf: false,
      apiExport: false,
      maxExportsPerDay: 10
    }
  };

  static readonly GOLD: StockRankingConfig = {
    maxRankings: 50,
    allowedRankingTypes: [
      StockRankingType.EPS_GROWTH,
      StockRankingType.MARKET_CAP,
      StockRankingType.VOLUME,
      StockRankingType.PRICE_CHANGE,
      StockRankingType.TECHNICAL_INDICATORS,
      StockRankingType.AI_INSIGHTS,
      StockRankingType.PATTERN_RECOGNITION
    ],
    rateLimitPerMinute: 200,
    realTimeUpdates: true,
    allowedMarkets: ['NYSE', 'NASDAQ', 'AMEX', 'LSE', 'TSX'],
    advancedFeatures: {
      customAlgorithms: true,
      aiInsights: true,
      patternRecognition: true,
      historicalData: true,
      alertSystem: true,
      portfolioTracking: true
    },
    exportOptions: {
      csv: true,
      excel: true,
      pdf: true,
      apiExport: true,
      maxExportsPerDay: 50
    }
  };

  static readonly PLATINUM: StockRankingConfig = {
    maxRankings: 100,
    allowedRankingTypes: Object.values(StockRankingType),
    rateLimitPerMinute: 500,
    realTimeUpdates: true,
    allowedMarkets: ['NYSE', 'NASDAQ', 'AMEX', 'LSE', 'TSX', 'ASX', 'HKEX', 'SSE'],
    advancedFeatures: {
      customAlgorithms: true,
      aiInsights: true,
      patternRecognition: true,
      historicalData: true,
      alertSystem: true,
      portfolioTracking: true
    },
    exportOptions: {
      csv: true,
      excel: true,
      pdf: true,
      apiExport: true,
      maxExportsPerDay: 200
    }
  };

  static readonly ENTERPRISE: StockRankingConfig = {
    maxRankings: -1, // Unlimited
    allowedRankingTypes: Object.values(StockRankingType),
    rateLimitPerMinute: 1000,
    realTimeUpdates: true,
    allowedMarkets: ['*'], // All markets
    advancedFeatures: {
      customAlgorithms: true,
      aiInsights: true,
      patternRecognition: true,
      historicalData: true,
      alertSystem: true,
      portfolioTracking: true
    },
    exportOptions: {
      csv: true,
      excel: true,
      pdf: true,
      apiExport: true,
      maxExportsPerDay: -1 // Unlimited
    }
  };

  static getConfigForTier(tier: PackageTier): StockRankingConfig {
    switch (tier) {
      case PackageTier.BRONZE:
        return this.BRONZE;
      case PackageTier.SILVER:
        return this.SILVER;
      case PackageTier.GOLD:
        return this.GOLD;
      case PackageTier.PLATINUM:
        return this.PLATINUM;
      case PackageTier.ENTERPRISE:
        return this.ENTERPRISE;
      case PackageTier.FREE:
      default:
        return {
          maxRankings: 3,
          allowedRankingTypes: [StockRankingType.EPS_GROWTH],
          rateLimitPerMinute: 5,
          realTimeUpdates: false,
          allowedMarkets: ['NYSE'],
          advancedFeatures: {
            customAlgorithms: false,
            aiInsights: false,
            patternRecognition: false,
            historicalData: false,
            alertSystem: false,
            portfolioTracking: false
          },
          exportOptions: {
            csv: false,
            excel: false,
            pdf: false,
            apiExport: false,
            maxExportsPerDay: 1
          }
        };
    }
  }

  static getAllConfigs(): Record<PackageTier, StockRankingConfig> {
    return {
      [PackageTier.FREE]: this.getConfigForTier(PackageTier.FREE),
      [PackageTier.BRONZE]: this.BRONZE,
      [PackageTier.SILVER]: this.SILVER,
      [PackageTier.GOLD]: this.GOLD,
      [PackageTier.PLATINUM]: this.PLATINUM,
      [PackageTier.ENTERPRISE]: this.ENTERPRISE
    };
  }
}