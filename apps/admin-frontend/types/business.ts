/**
 * Business Domain Types
 * Consolidates domain-specific business logic types
 * Replaces: payment.d.ts, userLevel.ts, embedded-permissions.ts, granular-permissions.ts
 */

// ============================================================================
// Payment & Subscription Types
// ============================================================================

export enum PaymentTier {
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  DIAMOND = 'diamond'
}

export interface PaymentTierConfig {
  id: string;
  name: string;
  description: string;
  monthlyPrice: number;
  yearlyPrice: number;
  features: string[];
  maxUsers?: number;
  maxApiCalls?: number;
  priority: 'low' | 'medium' | 'high';
  isActive: boolean;
  stripePriceId?: string;
  trialDays?: number;
}

export interface UserSubscription {
  id: string;
  userId: string;
  tierName: string;
  paymentTier: PaymentTierConfig;
  startDate: string;
  endDate: string;
  status: 'active' | 'cancelled' | 'expired' | 'pending' | 'trial';
  autoRenew: boolean;
  paymentMethod: 'credit_card' | 'paypal' | 'bank_transfer' | 'usdt' | 'crypto';
  lastPaymentDate?: string;
  nextPaymentDate?: string;
  cancelledAt?: string;
  cancelReason?: string;
  metadata?: Record<string, any>;
}

export interface PaymentStatus {
  id: string;
  userId: string;
  lastPaymentDate: string;
  expirationDate: string;
  paymentMethod: 'USDT' | 'credit_card' | 'paypal';
  transactionId: string;
  amount: number;
  currency: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled' | 'refunded';
  failureReason?: string;
}

export interface USDTDetails {
  network: 'ERC20' | 'TRC20' | 'BEP20' | 'Arbitrum' | 'TON';
  walletAddress: string;
  qrCodePath?: string;
  tag?: string;
  paymentStatus: PaymentStatus;
  minimumAmount: number;
  maxConfirmationTime: number; // in minutes
}

export interface PaymentTransaction {
  id: string;
  userId: string;
  amount: number;
  currency: string;
  status: PaymentStatus['status'];
  paymentMethod: PaymentStatus['paymentMethod'];
  transactionHash?: string;
  blockchainNetwork?: string;
  confirmations?: number;
  requiredConfirmations?: number;
  createdAt: string;
  confirmedAt?: string;
  description?: string;
  metadata?: Record<string, any>;
}

// ============================================================================
// User Level & Tier Management
// ============================================================================

export enum UserLevel {
  BRONZE = 'BRONZE',
  SILVER = 'SILVER', 
  GOLD = 'GOLD',
  PLATINUM = 'PLATINUM',
  DIAMOND = 'DIAMOND',
  VIP = 'VIP'
}

export interface UserLevelConfig {
  level: UserLevel;
  name: string;
  description: string;
  color: string;
  benefits: string[];
  tokenMultiplier: number;
  maxTokens: number;
  priority: number;
  apiRateLimit: number; // requests per minute
  supportLevel: 'basic' | 'priority' | 'premium' | 'dedicated';
}

export interface UserLevelAssignment {
  id: string;
  userId: string;
  userLevel: UserLevel;
  assignedBy: string;
  assignedAt: string;
  expiresAt?: string;
  reason?: string;
  previousLevel?: UserLevel;
  autoUpgraded?: boolean;
  metadata?: Record<string, any>;
}

// ============================================================================
// Stock Ranking & Analytics Domain
// ============================================================================

export interface StockData {
  symbol: string;
  name: string;
  exchange: string;
  country: string;
  sector: string;
  industry: string;
  marketCap: number;
  sharesOutstanding: number;
  currency: string;
  lastUpdated: string;
}

export interface EPSData {
  symbol: string;
  quarter: string;
  year: number;
  actualEPS: number;
  estimatedEPS: number;
  surprise: number;
  surprisePercent: number;
  reportDate: string;
  updatedAt: string;
}

export interface StockRanking {
  id: string;
  symbol: string;
  stockData: StockData;
  epsData: EPSData[];
  ranking: number;
  score: number;
  growthRate: number;
  volatility: number;
  momentum: number;
  calculatedAt: string;
  rankingType: string;
  period: '1M' | '3M' | '6M' | '1Y' | '2Y' | '5Y';
}

export interface RankingAlgorithm {
  id: string;
  name: string;
  description: string;
  version: string;
  parameters: Record<string, any>;
  weights: {
    growth: number;
    value: number;
    momentum: number;
    quality: number;
    risk: number;
  };
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface AnalyticsPackage {
  id: string;
  name: string;
  description: string;
  features: {
    stockCount: number;
    updateFrequency: 'realtime' | 'hourly' | 'daily';
    historicalData: '1Y' | '2Y' | '5Y' | 'unlimited';
    advancedMetrics: boolean;
    customAlerts: boolean;
    apiAccess: boolean;
    exportFormats: string[];
  };
  pricing: {
    monthlyPrice: number;
    yearlyPrice: number;
    enterprise: boolean;
  };
  restrictions: {
    maxApiCalls: number;
    maxExports: number;
    supportLevel: 'email' | 'priority' | 'dedicated';
  };
  isActive: boolean;
}

// ============================================================================
// Platform & Multi-tenancy Types
// ============================================================================

export enum Platform {
  EPSX = 'epsx',
  ADMIN = 'admin',
  EPSX_PAY = 'epsx-pay',
  EPSX_TOKEN = 'epsx-token',
  EPSX_ANALYTICS = 'epsx-analytics'
}

export interface PlatformConfig {
  id: Platform;
  name: string;
  description: string;
  baseUrl: string;
  apiUrl: string;
  features: string[];
  permissions: string[];
  theme: {
    primaryColor: string;
    secondaryColor: string;
    logo: string;
  };
  isActive: boolean;
}

export interface CrossPlatformUser {
  id: string;
  email: string;
  platforms: Platform[];
  permissions: Record<Platform, string[]>;
  subscriptions: Record<Platform, UserSubscription>;
  preferences: Record<Platform, Record<string, any>>;
  lastAccessed: Record<Platform, string>;
}

// ============================================================================
// Permission System Domain Types
// ============================================================================

export interface StructuredPermission {
  platform: Platform;
  resource: string;
  action: string;
  conditions?: PermissionCondition[];
}

export interface PermissionCondition {
  type: 'time_range' | 'ip_range' | 'rate_limit' | 'user_attribute';
  parameters: Record<string, any>;
}

export interface EmbeddedTimestampPermission {
  basePermission: string; // e.g., "epsx:analytics:view"
  timestamp: number; // Unix timestamp for expiry
  fullPermission: string; // e.g., "epsx:analytics:view:1640995200"
}

export interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  category: 'role_based' | 'feature_based' | 'time_limited' | 'custom';
  targetPlatform: Platform;
  permissions: StructuredPermission[];
  conditions: PermissionCondition[];
  isActive: boolean;
  usageCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface PermissionGroup {
  id: string;
  name: string;
  description: string;
  platform: Platform;
  permissions: string[];
  inherits?: string[]; // Other group IDs to inherit from
  priority: number;
  isSystemGroup: boolean;
}

// ============================================================================
// Security & Compliance Types
// ============================================================================

export interface SecurityEvent {
  id: string;
  userId: string;
  type: 'login_attempt' | 'permission_escalation' | 'data_access' | 'api_abuse' | 'suspicious_activity';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  ipAddress: string;
  userAgent: string;
  location?: {
    country: string;
    city: string;
    coordinates?: [number, number];
  };
  timestamp: string;
  resolved: boolean;
  resolvedBy?: string;
  resolvedAt?: string;
  metadata: Record<string, any>;
}

export interface ComplianceRule {
  id: string;
  name: string;
  description: string;
  type: 'gdpr' | 'ccpa' | 'sox' | 'custom';
  platform: Platform;
  conditions: {
    userAttributes?: Record<string, any>;
    permissions?: string[];
    dataTypes?: string[];
    actions?: string[];
  };
  requirements: {
    dataRetention: number; // days
    auditLog: boolean;
    encryption: boolean;
    anonymization: boolean;
    consentRequired: boolean;
  };
  isActive: boolean;
}

export interface RiskAssessment {
  id: string;
  userId: string;
  riskScore: number; // 0-100
  riskLevel: 'low' | 'medium' | 'high' | 'critical';
  factors: Array<{
    category: string;
    description: string;
    impact: number;
    weight: number;
  }>;
  recommendations: string[];
  assessedAt: string;
  assessedBy: 'system' | 'admin';
  validUntil: string;
}

// ============================================================================
// Analytics & Reporting Domain Types
// ============================================================================

export interface ReportTemplate {
  id: string;
  name: string;
  description: string;
  type: 'user_activity' | 'permission_audit' | 'security_summary' | 'financial_summary' | 'custom';
  parameters: {
    timeRange: {
      type: 'relative' | 'absolute';
      value: string; // e.g., '30d', '2024-01-01_2024-01-31'
    };
    filters: Record<string, any>;
    groupBy: string[];
    metrics: string[];
  };
  format: 'pdf' | 'csv' | 'json' | 'xlsx';
  schedule?: {
    frequency: 'daily' | 'weekly' | 'monthly' | 'quarterly';
    dayOfWeek?: number; // 0-6
    dayOfMonth?: number; // 1-31
    hour: number; // 0-23
  };
  recipients: string[];
  isActive: boolean;
}

export interface AnalyticsMetric {
  id: string;
  name: string;
  description: string;
  category: string;
  dataType: 'number' | 'percentage' | 'currency' | 'count' | 'duration';
  unit?: string;
  calculation: {
    type: 'sum' | 'avg' | 'count' | 'max' | 'min' | 'custom';
    expression?: string; // For custom calculations
  };
  visualization: {
    type: 'line' | 'bar' | 'pie' | 'gauge' | 'number';
    color?: string;
    thresholds?: Array<{
      value: number;
      color: string;
      label: string;
    }>;
  };
}

export interface Dashboard {
  id: string;
  name: string;
  description: string;
  layout: Array<{
    id: string;
    metric: string;
    position: { x: number; y: number; width: number; height: number };
    config: Record<string, any>;
  }>;
  filters: Record<string, any>;
  refreshInterval: number; // seconds
  isPublic: boolean;
  owners: string[];
  viewers: string[];
  createdAt: string;
  updatedAt: string;
}

// ============================================================================
// Integration & API Domain Types
// ============================================================================

export interface APIKey {
  id: string;
  name: string;
  userId: string;
  key: string; // Hashed in storage
  permissions: string[];
  rateLimit: {
    requestsPerMinute: number;
    requestsPerHour: number;
    requestsPerDay: number;
  };
  usage: {
    totalRequests: number;
    lastUsed?: string;
    monthlyRequests: number;
  };
  restrictions: {
    ipWhitelist?: string[];
    refererWhitelist?: string[];
    allowedEndpoints?: string[];
  };
  expiresAt?: string;
  isActive: boolean;
  createdAt: string;
}

export interface WebhookEndpoint {
  id: string;
  userId: string;
  url: string;
  events: string[];
  secret: string; // For signature verification
  headers: Record<string, string>;
  isActive: boolean;
  failureCount: number;
  lastSuccess?: string;
  lastFailure?: string;
  createdAt: string;
}

export interface ThirdPartyIntegration {
  id: string;
  name: string;
  type: 'oauth2' | 'api_key' | 'webhook' | 'custom';
  provider: string; // e.g., 'stripe', 'firebase', 'sendgrid'
  configuration: Record<string, any>;
  credentials: Record<string, string>; // Encrypted in storage
  permissions: string[];
  isActive: boolean;
  healthStatus: 'healthy' | 'degraded' | 'failing';
  lastHealthCheck: string;
  usage: {
    apiCalls: number;
    dataSync: string;
    errors: number;
  };
}

// ============================================================================
// Business Rules & Automation
// ============================================================================

export interface BusinessRule {
  id: string;
  name: string;
  description: string;
  trigger: {
    type: 'event' | 'schedule' | 'condition';
    configuration: Record<string, any>;
  };
  conditions: Array<{
    field: string;
    operator: 'equals' | 'contains' | 'greater_than' | 'less_than' | 'in' | 'not_in';
    value: any;
  }>;
  actions: Array<{
    type: 'email' | 'webhook' | 'permission_change' | 'user_update' | 'notification';
    configuration: Record<string, any>;
  }>;
  isActive: boolean;
  executionCount: number;
  lastExecuted?: string;
  createdAt: string;
  updatedAt: string;
}

export interface AutomationWorkflow {
  id: string;
  name: string;
  description: string;
  steps: Array<{
    id: string;
    type: 'condition' | 'action' | 'approval' | 'delay';
    configuration: Record<string, any>;
    nextSteps: string[];
  }>;
  triggers: string[]; // Business rule IDs
  status: 'active' | 'paused' | 'error';
  executionHistory: Array<{
    id: string;
    startedAt: string;
    completedAt?: string;
    status: 'running' | 'completed' | 'failed';
    currentStep: string;
    error?: string;
  }>;
  createdAt: string;
  updatedAt: string;
}

// ============================================================================
// Type Utilities for Business Domain
// ============================================================================

export function parseEmbeddedTimestampPermission(permission: string): EmbeddedTimestampPermission | null {
  const parts = permission.split(':');
  if (parts.length !== 4) return null;
  
  const timestamp = parseInt(parts[3], 10);
  if (isNaN(timestamp)) return null;
  
  return {
    basePermission: parts.slice(0, 3).join(':'),
    timestamp,
    fullPermission: permission
  };
}

export function createEmbeddedTimestampPermission(
  basePermission: string,
  expiryDate: Date
): EmbeddedTimestampPermission {
  const timestamp = Math.floor(expiryDate.getTime() / 1000);
  const fullPermission = `${basePermission}:${timestamp}`;
  
  return {
    basePermission,
    timestamp,
    fullPermission
  };
}

export function isPermissionExpired(permission: string): boolean {
  const embedded = parseEmbeddedTimestampPermission(permission);
  if (!embedded) return false;
  
  const now = Math.floor(Date.now() / 1000);
  return embedded.timestamp <= now;
}

export function getPermissionExpiryDate(permission: string): Date | null {
  const embedded = parseEmbeddedTimestampPermission(permission);
  if (!embedded) return null;
  
  return new Date(embedded.timestamp * 1000);
}

export function filterActivePermissions(permissions: string[]): string[] {
  return permissions.filter(permission => !isPermissionExpired(permission));
}

export function groupPermissionsByExpiry(permissions: string[]): {
  permanent: string[];
  temporary: string[];
  expired: string[];
} {
  const result = {
    permanent: [] as string[],
    temporary: [] as string[],
    expired: [] as string[]
  };
  
  permissions.forEach(permission => {
    const embedded = parseEmbeddedTimestampPermission(permission);
    if (!embedded) {
      result.permanent.push(permission);
    } else if (isPermissionExpired(permission)) {
      result.expired.push(permission);
    } else {
      result.temporary.push(permission);
    }
  });
  
  return result;
}