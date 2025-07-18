// AWS IAM-inspired permission service types
export interface Permission {
  id: string;
  resource: string;
  action: string;
  effect: 'Allow' | 'Deny';
  conditions?: PermissionCondition[];
  createdAt: Date;
  updatedAt: Date;
}

export interface PermissionCondition {
  key: string;
  operator: 'StringEquals' | 'StringLike' | 'NumericEquals' | 'NumericLessThan' | 'NumericGreaterThan' | 'DateEquals' | 'DateLessThan' | 'DateGreaterThan' | 'IpAddress' | 'NotIpAddress';
  value: string | number | Date;
}

export interface Policy {
  id: string;
  name: string;
  description: string;
  version: string;
  statement: PolicyStatement[];
  createdAt: Date;
  updatedAt: Date;
}

export interface PolicyStatement {
  sid?: string; // Statement ID
  effect: 'Allow' | 'Deny';
  actions: string[];
  resources: string[];
  conditions?: Record<string, PermissionCondition>;
  principals?: string[]; // User IDs or role ARNs
}

export interface Role {
  id: string;
  name: string;
  description: string;
  policies: Policy[];
  inlinePolicies?: Policy[];
  assumeRolePolicyDocument?: Policy;
  maxSessionDuration?: number;
  path?: string;
  tags?: Record<string, string>;
  createdAt: Date;
  updatedAt: Date;
}

export interface Group {
  id: string;
  name: string;
  description: string;
  policies: Policy[];
  members: string[]; // User IDs
  path?: string;
  tags?: Record<string, string>;
  createdAt: Date;
  updatedAt: Date;
}

export interface UserPermissions {
  userId: string;
  directPolicies: Policy[];
  roles: Role[];
  groups: Group[];
  effectivePermissions?: Permission[];
  lastEvaluated?: Date;
}

// Resource types in your application
export enum ResourceType {
  // Stock Analytics
  STOCK_RANKINGS = 'stock:rankings',
  STOCK_ANALYTICS = 'stock:analytics',
  STOCK_RESEARCH = 'stock:research',
  STOCK_SCREENER = 'stock:screener',
  
  // Market Data
  MARKET_DATA = 'market:data',
  MARKET_SYNC = 'market:sync',
  REALTIME_DATA = 'market:realtime',
  
  // User Management
  USER_PROFILE = 'user:profile',
  USER_SETTINGS = 'user:settings',
  USER_SUBSCRIPTION = 'user:subscription',
  
  // Admin Functions
  ADMIN_USERS = 'admin:users',
  ADMIN_ANALYTICS = 'admin:analytics',
  ADMIN_SYSTEM = 'admin:system',
  
  // API Access
  API_LIMITS = 'api:limits',
  API_WEBHOOKS = 'api:webhooks',
  
  // Payment & Billing
  PAYMENT_HISTORY = 'payment:history',
  PAYMENT_METHODS = 'payment:methods',
  BILLING_MANAGEMENT = 'billing:management',
}

// Action types
export enum ActionType {
  // CRUD Operations
  CREATE = 'create',
  READ = 'read',
  UPDATE = 'update',
  DELETE = 'delete',
  
  // Specific Actions
  LIST = 'list',
  VIEW = 'view',
  EXPORT = 'export',
  IMPORT = 'import',
  
  // Analytics Actions
  ANALYZE = 'analyze',
  RANK = 'rank',
  SCREEN = 'screen',
  
  // Admin Actions
  MANAGE = 'manage',
  CONFIGURE = 'configure',
  MONITOR = 'monitor',
  
  // Special Actions
  ASSUME_ROLE = 'assume-role',
  DELEGATE = 'delegate',
}

// Context for permission evaluation
export interface PermissionContext {
  userId: string;
  requestId: string;
  timestamp: Date;
  ipAddress?: string;
  userAgent?: string;
  resource: string;
  action: string;
  additionalContext?: Record<string, any>;
}

// Permission evaluation result
export interface PermissionEvaluationResult {
  allowed: boolean;
  reason: string;
  matchedPolicies: Policy[];
  evaluationTime: number;
  context: PermissionContext;
  appliedConditions?: PermissionCondition[];
}

// Resource ARN (Amazon Resource Name) format
export interface ResourceArn {
  partition: string; // epsx
  service: string; // stock, market, user, admin
  region?: string;
  accountId?: string;
  resourceType: string;
  resourceId: string;
}

// Built-in system policies
export enum SystemPolicy {
  FULL_ACCESS = 'FullAccess',
  READ_ONLY = 'ReadOnly',
  POWER_USER = 'PowerUser',
  ADMIN_ACCESS = 'AdminAccess',
  BILLING_ACCESS = 'BillingAccess',
  ANALYTICS_ACCESS = 'AnalyticsAccess',
  STOCK_TRADER_ACCESS = 'StockTraderAccess',
  MARKET_DATA_ACCESS = 'MarketDataAccess',
}

// Permission evaluation options
export interface EvaluationOptions {
  includeResourcePolicy?: boolean;
  includeSessionPolicy?: boolean;
  simulateOnly?: boolean;
  logEvaluation?: boolean;
}
