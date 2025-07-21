// Define shared types and interfaces
export interface SharedConfig {
  environment: 'development' | 'production' | 'test';
  version: string;
}

// Common response type
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

// SystemMetrics and related types for seeding
export interface SystemMetrics {
  id: string;
  timestamp: any; // Timestamp from Firestore
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
  createdAt: any;
  updatedAt: any;
}

export interface UsageAnalytics {
  id: string;
  userId: string;
  date: any;
  metrics: any;
  breakdown: any;
  createdAt: any;
  updatedAt: any;
}

export interface FeatureUsage {
  userId: string;
  feature: string;
  timestamp: any;
  metadata: {
    duration: number;
    interactions: number;
    context: string;
    data: {
      sessionId: string;
      referrer: string;
      userAgent: string;
    };
  };
}

export interface UsageTracking {
  id: string;
  userId: string;
  periodStart: any;
  periodEnd: any;
  metrics: any;
}

export interface SeedResult {
  success: boolean;
  collection: string;
  count: number;
  error?: string;
}
