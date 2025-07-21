import { Timestamp } from 'firebase/firestore';

// Base interfaces
export interface BaseEntity {
  id: string;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

export interface SeedOptions {
  environment: 'development' | 'production' | 'test';
  force?: boolean;
  verbose?: boolean;
  collections?: string[];
}

export interface SeedResult {
  success: boolean;
  collection: string;
  count: number;
  error?: string;
}

// IAM Types
export interface Role {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  isSystem?: boolean;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

export interface Permission {
  id: string;
  featureId: string;
  permission: 'read' | 'write' | 'admin' | 'access';
  description: string;
  category?: string;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

export interface User extends BaseEntity {
  uid: string;
  email: string;
  profile: UserProfile;
  roles: string[];
  packageLevel: PackageLevel;
  permissions?: {
    computed: string[];
    explicit: string[];
    inherited: string[];
  };
  organizationId?: string;
  isActive: boolean;
}

export interface UserProfile {
  firstName: string;
  lastName: string;
  avatar?: string | null;
  department?: string;
  position?: string;
  phone?: string;
  bio?: string;
  location?: string;
  lastLogin?: Timestamp | null;
}

export type PackageLevel = 'FREE' | 'BRONZE' | 'SILVER' | 'GOLD' | 'ENTERPRISE';

export interface PackagePermission {
  featureId: string;
  permission: string;
  limits?: Record<string, number | string>;
  conditions?: Record<string, any>;
}

// Organization Types
export interface Organization extends BaseEntity {
  name: string;
  slug: string;
  logo?: string;
  settings: OrganizationSettings;
  subscription: Subscription;
  metadata?: Record<string, any>;
}

export interface OrganizationSettings {
  allowUserRegistration: boolean;
  requireEmailVerification: boolean;
  allowGuestAccess: boolean;
  defaultRole: string;
  passwordPolicy: PasswordPolicy;
  branding?: {
    primaryColor?: string;
    logo?: string;
    favicon?: string;
  };
}

export interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSpecialChars: boolean;
  maxAge?: number;
  preventReuse?: number;
}

export interface Subscription {
  plan: PackageLevel;
  status: 'active' | 'inactive' | 'trial' | 'cancelled' | 'expired';
  startDate: Timestamp;
  endDate: Timestamp;
  autoRenew?: boolean;
  paymentMethod?: string;
}

// Session & Audit Types
export interface UserSession extends BaseEntity {
  sessionId: string;
  userId: string;
  deviceInfo: DeviceInfo;
  isActive: boolean;
  lastActivity: Timestamp;
  expiresAt: Timestamp;
  metadata?: Record<string, any>;
}

export interface DeviceInfo {
  userAgent: string;
  ip: string;
  device?: string;
  browser?: string;
  os?: string;
  location?: {
    country?: string;
    city?: string;
    timezone?: string;
  };
}

export interface AuditLog extends BaseEntity {
  userId: string;
  action: string;
  resource: string;
  resourceId?: string;
  details: Record<string, any>;
  timestamp: Timestamp;
  metadata: {
    ip: string;
    userAgent: string;
    sessionId?: string;
    organizationId?: string;
  };
  severity?: 'low' | 'medium' | 'high' | 'critical';
}

// Content Management Types
export interface Content extends BaseEntity {
  type: 'article' | 'page' | 'post' | 'documentation';
  title: string;
  slug: string;
  content: {
    body: string;
    excerpt?: string;
    featuredImage?: string;
    gallery?: string[];
  };
  metadata: {
    author: string;
    category?: string;
    tags: string[];
    seo: SEOMetadata;
    customFields?: Record<string, any>;
  };
  status: 'draft' | 'published' | 'archived' | 'private';
  publishedAt?: Timestamp;
  scheduledAt?: Timestamp;
  version: number;
}

export interface SEOMetadata {
  metaTitle?: string;
  metaDescription?: string;
  canonicalUrl?: string;
  keywords?: string[];
  ogImage?: string;
}

export interface Media extends BaseEntity {
  filename: string;
  originalName: string;
  mimeType: string;
  size: number;
  url: string;
  thumbnailUrl?: string;
  metadata: {
    width?: number;
    height?: number;
    duration?: number;
    uploadedBy: string;
    folder?: string;
    alt?: string;
    caption?: string;
  };
  isPublic: boolean;
}

export interface Category extends BaseEntity {
  name: string;
  slug: string;
  description?: string;
  parentId?: string | null;
  sortOrder: number;
  metadata?: {
    color?: string;
    icon?: string;
    isVisible?: boolean;
  };
}

// Analytics Types
export interface UsageAnalytics extends BaseEntity {
  userId: string;
  date: Timestamp;
  metrics: {
    pageViews: number;
    sessionDuration: number;
    actionsPerformed: number;
    apiCalls: number;
    features: Record<string, number>;
  };
  breakdown: {
    byHour: number[];
    byFeature: Record<string, number>;
    byDevice: Record<string, number>;
  };
}

export interface SystemMetrics extends BaseEntity {
  timestamp: Timestamp;
  metrics: {
    activeUsers: number;
    totalSessions: number;
    avgResponseTime: number;
    errorRate: number;
    uptime: number;
  };
  performance: {
    cpuUsage: number;
    memoryUsage: number;
    diskUsage: number;
    networkLatency: number;
  };
  alerts?: Array<{
    type: string;
    message: string;
    severity: 'low' | 'medium' | 'high' | 'critical';
  }>;
}

export interface FeatureUsage {
  userId: string;
  feature: string;
  timestamp: Timestamp;
  metadata: {
    duration?: number;
    interactions?: number;
    context?: string;
    data?: Record<string, any>;
  };
}

// Notification Types
export interface Notification extends BaseEntity {
  userId: string;
  type: 'system' | 'user' | 'marketing' | 'security';
  title: string;
  message: string;
  data?: {
    actionUrl?: string;
    actionText?: string;
    metadata?: Record<string, any>;
  };
  status: 'unread' | 'read' | 'archived';
  priority: 'low' | 'normal' | 'high' | 'urgent';
  readAt?: Timestamp | null;
  expiresAt?: Timestamp;
}

export interface EmailTemplate extends BaseEntity {
  name: string;
  subject: string;
  body: string;
  variables: string[];
  isActive: boolean;
  category?: string;
  metadata?: {
    lastUsed?: Timestamp;
    usageCount?: number;
  };
}

export interface MessageQueue extends BaseEntity {
  type: 'email' | 'sms' | 'push' | 'webhook';
  recipient: string;
  templateId?: string;
  data: Record<string, any>;
  status: 'pending' | 'processing' | 'sent' | 'failed' | 'cancelled';
  scheduledAt: Timestamp;
  sentAt?: Timestamp;
  attempts: number;
  maxAttempts?: number;
  lastError?: string;
}

// Configuration Types
export interface SystemSettings extends BaseEntity {
  category: string;
  settings: Record<string, any>;
  updatedBy: string;
  isPublic?: boolean;
  description?: string;
}

export interface FeatureFlag extends BaseEntity {
  name: string;
  description: string;
  isEnabled: boolean;
  rolloutPercentage: number;
  conditions?: {
    userRoles?: string[];
    packageLevels?: PackageLevel[];
    organizations?: string[];
    users?: string[];
    environment?: string[];
  };
  metadata?: {
    owner?: string;
    jiraTicket?: string;
    expiresAt?: Timestamp;
  };
}

export interface Integration extends BaseEntity {
  name: string;
  type: 'payment' | 'analytics' | 'communication' | 'storage' | 'auth' | 'other';
  config: Record<string, any>;
  isActive: boolean;
  healthCheck?: {
    lastChecked?: Timestamp;
    status?: 'healthy' | 'degraded' | 'down';
    message?: string;
  };
  metadata?: {
    version?: string;
    documentation?: string;
    supportContact?: string;
  };
}

// Invitation Types
export interface Invitation extends BaseEntity {
  email: string;
  organizationId: string;
  invitedBy: string;
  roles: string[];
  status: 'pending' | 'accepted' | 'declined' | 'expired';
  token: string;
  expiresAt: Timestamp;
  metadata?: {
    personalMessage?: string;
    customData?: Record<string, any>;
  };
}

export interface UserPreferences extends BaseEntity {
  userId: string;
  ui: {
    theme: 'light' | 'dark' | 'auto';
    language: string;
    timezone: string;
    dateFormat: string;
    timeFormat: '12' | '24';
    notifications: {
      email: boolean;
      push: boolean;
      sms: boolean;
      desktop: boolean;
    };
  };
  privacy: {
    profileVisibility: 'public' | 'organization' | 'private';
    showEmail: boolean;
    showPhone: boolean;
    allowMarketing: boolean;
    allowAnalytics: boolean;
  };
  features?: {
    enableBetaFeatures: boolean;
    enableAdvancedMode: boolean;
    customSettings: Record<string, any>;
  };
}

// Usage Tracking Types
export interface UsageTracking extends BaseEntity {
  userId: string;
  packageLevel: PackageLevel;
  currentPeriod: {
    startDate: Timestamp;
    endDate: Timestamp;
    apiCalls: number;
    exports: number;
    storage: number;
    features: Record<string, number>;
  };
  limits: {
    apiCalls: number; // -1 for unlimited
    exports: number;
    storage: number;
    features?: Record<string, number>;
  };
  warnings?: {
    apiCallsWarning?: boolean;
    exportsWarning?: boolean;
    storageWarning?: boolean;
  };
}
