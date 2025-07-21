import { Timestamp } from 'firebase/firestore';
import { BaseSeeder } from './base';
import type { UsageAnalytics, SystemMetrics, FeatureUsage, UsageTracking, SeedResult } from '../types';

export class AnalyticsSeeder extends BaseSeeder {
  get collectionName(): string {
    return 'analytics';
  }

  async seed(): Promise<SeedResult> {
    try {
      await this.seedUsageAnalytics();
      await this.seedSystemMetrics();
      await this.seedFeatureUsage();
      await this.seedUsageTracking();

      return {
        success: true,
        collection: 'analytics',
        count: 9 + 7 + 15 + 3 // usage + system + feature + tracking
      };
    } catch (error) {
      return {
        success: false,
        collection: 'analytics',
        count: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  private async seedUsageAnalytics() {
    this.log('Seeding usage analytics...');
    
    const now = new Date();
    const usageAnalytics: UsageAnalytics[] = [];

    // Generate analytics for the past 7 days for each user
    const users = ['admin-001', 'manager-001', 'beta-001'];
    
    for (let dayOffset = 0; dayOffset < 7; dayOffset++) {
      const date = this.addDays(now, -dayOffset);
      
      for (const userId of users) {
        const baseMetrics = this.generateUserMetrics(userId, dayOffset);
        
        usageAnalytics.push({
          id: `usage_${userId}_${dayOffset}`,
          userId,
          date: Timestamp.fromDate(date),
          metrics: baseMetrics,
          breakdown: {
            byHour: this.generateHourlyBreakdown(baseMetrics.pageViews),
            byFeature: {
              dashboard: Math.floor(baseMetrics.pageViews * 0.4),
              analytics: Math.floor(baseMetrics.pageViews * 0.3),
              content: Math.floor(baseMetrics.pageViews * 0.2),
              settings: Math.floor(baseMetrics.pageViews * 0.1)
            },
            byDevice: {
              desktop: Math.floor(baseMetrics.pageViews * 0.7),
              mobile: Math.floor(baseMetrics.pageViews * 0.2),
              tablet: Math.floor(baseMetrics.pageViews * 0.1)
            }
          },
          createdAt: Timestamp.fromDate(date),
          updatedAt: Timestamp.fromDate(date)
        });
      }
    }

    await this.seedCollection('usageAnalytics', usageAnalytics, 'id');
  }

  private async seedSystemMetrics() {
    this.log('Seeding system metrics...');
    
    const now = new Date();
    const systemMetrics: SystemMetrics[] = [];

    // Generate metrics for the past 7 days (daily snapshots)
    for (let dayOffset = 0; dayOffset < 7; dayOffset++) {
      const date = this.addDays(now, -dayOffset);
      const isWeekend = date.getDay() === 0 || date.getDay() === 6;
      
      systemMetrics.push({
        id: `sys_${date.getTime()}`,
        timestamp: Timestamp.fromDate(date),
        metrics: {
          activeUsers: isWeekend ? 15 + Math.floor(Math.random() * 10) : 45 + Math.floor(Math.random() * 20),
          totalSessions: isWeekend ? 25 + Math.floor(Math.random() * 15) : 85 + Math.floor(Math.random() * 30),
          avgResponseTime: 150 + Math.floor(Math.random() * 50), // ms
          errorRate: Math.random() * 2, // percentage
          uptime: 99.5 + Math.random() * 0.5 // percentage
        },
        performance: {
          cpuUsage: 20 + Math.random() * 40, // percentage
          memoryUsage: 45 + Math.random() * 30, // percentage
          diskUsage: 60 + Math.random() * 15, // percentage
          networkLatency: 10 + Math.random() * 20 // ms
        },
        alerts: dayOffset === 0 ? [
          {
            type: 'performance',
            message: 'CPU usage above 80% for 5 minutes',
            severity: 'medium' as const
          }
        ] : undefined,
        createdAt: Timestamp.fromDate(date),
        updatedAt: Timestamp.fromDate(date)
      });
    }

    await this.seedCollection('systemMetrics', systemMetrics, 'id');
  }

  private async seedFeatureUsage() {
    this.log('Seeding feature usage...');
    
    const now = new Date();
    const featureUsage: FeatureUsage[] = [];
    const users = ['admin-001', 'manager-001', 'beta-001'];
    const features = [
      'dashboard', 'analytics', 'content_management', 'user_management',
      'api_access', 'reports', 'exports', 'settings', 'beta_features'
    ];

    // Generate feature usage events for the past 24 hours
    for (let hourOffset = 0; hourOffset < 24; hourOffset++) {
      const timestamp = this.addHours(now, -hourOffset);
      
      // Generate random feature usage events for this hour
      const eventsThisHour = Math.floor(Math.random() * 10) + 1;
      
      for (let event = 0; event < eventsThisHour; event++) {
        const userId = users[Math.floor(Math.random() * users.length)];
        const feature = features[Math.floor(Math.random() * features.length)];
        
        featureUsage.push({
          userId,
          feature,
          timestamp: Timestamp.fromDate(new Date(timestamp.getTime() + Math.random() * 3600000)), // Random within the hour
          metadata: {
            duration: Math.floor(Math.random() * 300) + 30, // 30-330 seconds
            interactions: Math.floor(Math.random() * 20) + 1,
            context: this.getFeatureContext(feature),
            data: {
              sessionId: `sess_${userId}`,
              referrer: Math.random() > 0.5 ? 'dashboard' : 'direct',
              userAgent: 'Mozilla/5.0 (compatible)'
            }
          }
        });
      }
    }

    await this.seedCollection('featureUsage', featureUsage);
  }

  private async seedUsageTracking() {
    this.log('Seeding usage tracking...');
    
    const now = new Date();
    const periodStart = new Date(now.getFullYear(), now.getMonth(), 1); // Start of current month
    const periodEnd = this.addDays(periodStart, 30); // 30 days from start
    
    const usageTracking: UsageTracking[] = [
      {
        id: 'track_admin_001',
        userId: 'admin-001',
        packageLevel: 'ENTERPRISE',
        currentPeriod: {
          startDate: Timestamp.fromDate(periodStart),
          endDate: Timestamp.fromDate(periodEnd),
          apiCalls: 15420,
          exports: 89,
          storage: 2147483648, // 2GB in bytes
          features: {
            reports: 45,
            dashboards: 12,
            customReports: 8,
            apiKeys: 3
          }
        },
        limits: {
          apiCalls: -1, // unlimited
          exports: -1,
          storage: -1,
          features: {
            reports: -1,
            dashboards: -1,
            customReports: -1,
            apiKeys: -1
          }
        },
        createdAt: Timestamp.fromDate(periodStart),
        updatedAt: Timestamp.now()
      },
      {
        id: 'track_manager_001',
        userId: 'manager-001',
        packageLevel: 'GOLD',
        currentPeriod: {
          startDate: Timestamp.fromDate(periodStart),
          endDate: Timestamp.fromDate(periodEnd),
          apiCalls: 8750,
          exports: 156,
          storage: 1073741824, // 1GB in bytes
          features: {
            reports: 28,
            dashboards: 6,
            customReports: 12,
            apiKeys: 2
          }
        },
        limits: {
          apiCalls: 25000,
          exports: 1000,
          storage: 21474836480, // 20GB
          features: {
            reports: -1,
            dashboards: -1,
            customReports: -1,
            apiKeys: 10
          }
        },
        warnings: {
          apiCallsWarning: false,
          exportsWarning: false,
          storageWarning: false
        },
        createdAt: Timestamp.fromDate(periodStart),
        updatedAt: Timestamp.now()
      },
      {
        id: 'track_beta_001',
        userId: 'beta-001',
        packageLevel: 'SILVER',
        currentPeriod: {
          startDate: Timestamp.fromDate(periodStart),
          endDate: Timestamp.fromDate(periodEnd),
          apiCalls: 3250,
          exports: 45,
          storage: 536870912, // 512MB in bytes
          features: {
            reports: 18,
            dashboards: 4,
            customReports: 6
          }
        },
        limits: {
          apiCalls: 5000,
          exports: 200,
          storage: 5368709120, // 5GB
          features: {
            reports: 50,
            dashboards: 10,
            customReports: 25
          }
        },
        warnings: {
          apiCallsWarning: true, // Close to limit
          exportsWarning: false,
          storageWarning: false
        },
        createdAt: Timestamp.fromDate(periodStart),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('usageTracking', usageTracking, 'id');
  }

  private generateUserMetrics(userId: string, dayOffset: number) {
    // Different usage patterns based on user type
    const userMultipliers = {
      'admin-001': { pages: 1.5, session: 1.8, actions: 2.0, api: 3.0 },
      'manager-001': { pages: 1.2, session: 1.3, actions: 1.5, api: 1.8 },
      'beta-001': { pages: 0.8, session: 1.0, actions: 1.2, api: 0.9 }
    };

    const multiplier = userMultipliers[userId as keyof typeof userMultipliers] || { pages: 1, session: 1, actions: 1, api: 1 };
    
    // Reduce activity for older days
    const dayMultiplier = 1 - (dayOffset * 0.1);
    
    return {
      pageViews: Math.floor((20 + Math.random() * 30) * multiplier.pages * dayMultiplier),
      sessionDuration: Math.floor((1800 + Math.random() * 3600) * multiplier.session * dayMultiplier), // seconds
      actionsPerformed: Math.floor((50 + Math.random() * 100) * multiplier.actions * dayMultiplier),
      apiCalls: Math.floor((100 + Math.random() * 500) * multiplier.api * dayMultiplier),
      features: {
        dashboard: Math.floor((5 + Math.random() * 10) * dayMultiplier),
        analytics: Math.floor((3 + Math.random() * 8) * dayMultiplier),
        content: Math.floor((2 + Math.random() * 6) * dayMultiplier),
        reports: Math.floor((1 + Math.random() * 4) * dayMultiplier)
      }
    };
  }

  private generateHourlyBreakdown(totalViews: number): number[] {
    const hourlyData = new Array(24).fill(0);
    let remaining = totalViews;
    
    // Simulate realistic usage patterns (higher during work hours)
    const workHours = [9, 10, 11, 12, 13, 14, 15, 16, 17];
    const weights = hourlyData.map((_, hour) => {
      if (workHours.includes(hour)) return 3;
      if (hour >= 7 && hour <= 19) return 2;
      return 1;
    });
    
    const totalWeight = weights.reduce((sum, weight) => sum + weight, 0);
    
    for (let hour = 0; hour < 24; hour++) {
      const proportion = weights[hour] / totalWeight;
      const views = Math.floor(totalViews * proportion);
      hourlyData[hour] = views;
      remaining -= views;
    }
    
    // Distribute remaining views randomly
    while (remaining > 0) {
      const randomHour = Math.floor(Math.random() * 24);
      hourlyData[randomHour]++;
      remaining--;
    }
    
    return hourlyData;
  }

  private getFeatureContext(feature: string): string {
    const contexts = {
      dashboard: 'main_view',
      analytics: 'data_exploration',
      content_management: 'content_creation',
      user_management: 'admin_task',
      api_access: 'integration',
      reports: 'report_generation',
      exports: 'data_export',
      settings: 'configuration',
      beta_features: 'testing'
    };
    
    return contexts[feature as keyof typeof contexts] || 'general_usage';
  }
}
