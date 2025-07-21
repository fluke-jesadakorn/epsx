import { Timestamp } from 'firebase/firestore';
import { BaseSeeder } from './base';
import type { SystemSettings, FeatureFlag, Integration, SeedResult } from '../types';

export class ConfigurationSeeder extends BaseSeeder {
  get collectionName(): string {
    return 'configuration';
  }

  async seed(): Promise<SeedResult> {
    try {
      await this.seedSystemSettings();
      await this.seedFeatureFlags();
      await this.seedIntegrations();

      return {
        success: true,
        collection: 'configuration',
        count: 6 + 8 + 5 // settings + flags + integrations
      };
    } catch (error) {
      return {
        success: false,
        collection: 'configuration',
        count: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  private async seedSystemSettings() {
    this.log('Seeding system settings...');
    
    const systemSettings: SystemSettings[] = [
      {
        id: 'general',
        category: 'general',
        settings: {
          siteName: 'EPSX Platform',
          siteUrl: 'https://epsx.com',
          adminEmail: 'admin@epsx.com',
          supportEmail: 'support@epsx.com',
          timezone: 'UTC',
          dateFormat: 'YYYY-MM-DD',
          timeFormat: '24',
          language: 'en',
          currency: 'USD',
          allowRegistration: true,
          requireEmailVerification: true,
          defaultUserRole: 'viewer',
          maintenanceMode: false
        },
        isPublic: true,
        description: 'General platform settings and configuration',
        updatedBy: 'admin-001',
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'security',
        category: 'security',
        settings: {
          sessionTimeout: 3600, // 1 hour in seconds
          maxLoginAttempts: 5,
          lockoutDuration: 900, // 15 minutes in seconds
          passwordExpiry: 90, // days
          twoFactorRequired: false,
          twoFactorRequiredForAdmins: true,
          allowedDomains: ['epsx.com', 'example.com'],
          ipWhitelist: [],
          enforceHttps: true,
          corsOrigins: ['https://epsx.com', 'https://app.epsx.com'],
          apiRateLimit: {
            windowMs: 900000, // 15 minutes
            maxRequests: 100,
            skipSuccessfulRequests: false
          }
        },
        isPublic: false,
        description: 'Security and authentication settings',
        updatedBy: 'admin-001',
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'email',
        category: 'email',
        settings: {
          provider: 'sendgrid',
          fromName: 'EPSX Platform',
          fromEmail: 'noreply@epsx.com',
          replyToEmail: 'support@epsx.com',
          smtpHost: 'smtp.sendgrid.net',
          smtpPort: 587,
          smtpSecure: true,
          enableEmailLogs: true,
          emailRetryAttempts: 3,
          emailRetryDelay: 300, // 5 minutes in seconds
          bounceHandling: true,
          unsubscribeUrl: 'https://epsx.com/unsubscribe'
        },
        isPublic: false,
        description: 'Email service configuration and SMTP settings',
        updatedBy: 'admin-001',
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'storage',
        category: 'storage',
        settings: {
          provider: 'aws_s3',
          bucket: 'epsx-uploads',
          region: 'us-west-2',
          maxFileSize: 52428800, // 50MB in bytes
          allowedFileTypes: [
            'image/jpeg', 'image/png', 'image/gif', 'image/webp',
            'application/pdf', 'text/csv', 'application/json',
            'application/vnd.ms-excel',
            'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
          ],
          imageResizing: true,
          thumbnailSizes: [150, 300, 600],
          compressionEnabled: true,
          compressionQuality: 85,
          cdnEnabled: true,
          cdnUrl: 'https://cdn.epsx.com'
        },
        isPublic: false,
        description: 'File storage and CDN configuration',
        updatedBy: 'admin-001',
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'analytics',
        category: 'analytics',
        settings: {
          enableTracking: true,
          anonymizeIps: true,
          cookieConsent: true,
          dataRetentionDays: 730, // 2 years
          realTimeEnabled: true,
          exportFormats: ['csv', 'json', 'pdf'],
          scheduledReports: true,
          alertsEnabled: true,
          alertThresholds: {
            errorRate: 5, // percentage
            responseTime: 2000, // milliseconds
            cpuUsage: 80, // percentage
            memoryUsage: 85 // percentage
          },
          customDimensions: ['user_role', 'package_level', 'organization']
        },
        isPublic: false,
        description: 'Analytics and monitoring configuration',
        updatedBy: 'admin-001',
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'billing',
        category: 'billing',
        settings: {
          provider: 'stripe',
          currency: 'USD',
          taxCalculation: true,
          invoicePrefix: 'EPSX',
          gracePeriodDays: 7,
          trialPeriodDays: 14,
          allowDowngrades: true,
          prorateCharges: true,
          webhookRetries: 3,
          paymentMethods: ['card', 'bank_transfer'],
          plans: {
            FREE: { price: 0, features: ['basic_analytics'], limits: { users: 3, apiCalls: 100 } },
            BRONZE: { price: 29, features: ['basic_analytics', 'reports'], limits: { users: 10, apiCalls: 1000 } },
            SILVER: { price: 79, features: ['basic_analytics', 'reports', 'beta_features'], limits: { users: 25, apiCalls: 5000 } },
            GOLD: { price: 199, features: ['all'], limits: { users: 100, apiCalls: 25000 } },
            ENTERPRISE: { price: 499, features: ['all', 'white_label', 'priority_support'], limits: { users: -1, apiCalls: -1 } }
          }
        },
        isPublic: false,
        description: 'Billing and subscription management settings',
        updatedBy: 'admin-001',
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('systemSettings', systemSettings, 'id');
  }

  private async seedFeatureFlags() {
    this.log('Seeding feature flags...');
    
    const now = new Date();
    const featureFlags: FeatureFlag[] = [
      {
        id: 'beta_features',
        name: 'Beta Features Access',
        description: 'Enable access to beta features for testing users',
        isEnabled: true,
        rolloutPercentage: 100,
        conditions: {
          userRoles: ['beta-tester', 'admin'],
          packageLevels: ['SILVER', 'GOLD', 'ENTERPRISE']
        },
        metadata: {
          owner: 'product-team',
          jiraTicket: 'EPSX-1234',
          expiresAt: Timestamp.fromDate(this.addDays(now, 90))
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -30)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -5))
      },
      {
        id: 'advanced_analytics',
        name: 'Advanced Analytics Dashboard',
        description: 'Enable ML-powered advanced analytics features',
        isEnabled: true,
        rolloutPercentage: 75,
        conditions: {
          packageLevels: ['SILVER', 'GOLD', 'ENTERPRISE'],
          environment: ['production']
        },
        metadata: {
          owner: 'analytics-team',
          jiraTicket: 'EPSX-2345'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -20)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -2))
      },
      {
        id: 'new_dashboard_ui',
        name: 'New Dashboard UI',
        description: 'Enable the redesigned dashboard interface',
        isEnabled: false,
        rolloutPercentage: 25,
        conditions: {
          userRoles: ['beta-tester'],
          users: ['admin-001', 'beta-001']
        },
        metadata: {
          owner: 'ui-team',
          jiraTicket: 'EPSX-3456',
          expiresAt: Timestamp.fromDate(this.addDays(now, 60))
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -10)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -1))
      },
      {
        id: 'real_time_collaboration',
        name: 'Real-time Collaboration',
        description: 'Enable real-time collaborative editing features',
        isEnabled: false,
        rolloutPercentage: 10,
        conditions: {
          packageLevels: ['GOLD', 'ENTERPRISE'],
          organizations: ['org_001']
        },
        metadata: {
          owner: 'collaboration-team',
          jiraTicket: 'EPSX-4567'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -15)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -3))
      },
      {
        id: 'mobile_app_access',
        name: 'Mobile App Access',
        description: 'Enable access to mobile application features',
        isEnabled: true,
        rolloutPercentage: 100,
        conditions: {
          packageLevels: ['BRONZE', 'SILVER', 'GOLD', 'ENTERPRISE']
        },
        metadata: {
          owner: 'mobile-team',
          jiraTicket: 'EPSX-5678'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -45)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -10))
      },
      {
        id: 'api_v2',
        name: 'API v2 Access',
        description: 'Enable access to API version 2 with enhanced features',
        isEnabled: true,
        rolloutPercentage: 90,
        conditions: {
          userRoles: ['api-user', 'admin'],
          packageLevels: ['SILVER', 'GOLD', 'ENTERPRISE']
        },
        metadata: {
          owner: 'api-team',
          jiraTicket: 'EPSX-6789'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -25)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -7))
      },
      {
        id: 'white_label_branding',
        name: 'White Label Branding',
        description: 'Enable custom branding and white label features',
        isEnabled: true,
        rolloutPercentage: 100,
        conditions: {
          packageLevels: ['ENTERPRISE']
        },
        metadata: {
          owner: 'enterprise-team',
          jiraTicket: 'EPSX-7890'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -60)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -20))
      },
      {
        id: 'enhanced_security',
        name: 'Enhanced Security Features',
        description: 'Enable advanced security features like SSO and SCIM',
        isEnabled: false,
        rolloutPercentage: 0,
        conditions: {
          packageLevels: ['ENTERPRISE'],
          organizations: ['org_001']
        },
        metadata: {
          owner: 'security-team',
          jiraTicket: 'EPSX-8901',
          expiresAt: Timestamp.fromDate(this.addDays(now, 120))
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -5)),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('featureFlags', featureFlags, 'id');
  }

  private async seedIntegrations() {
    this.log('Seeding integrations...');
    
    const now = new Date();
    const integrations: Integration[] = [
      {
        id: 'stripe_integration',
        name: 'Stripe Payment Gateway',
        type: 'payment',
        config: {
          publicKey: 'pk_test_51234567890abcdef',
          webhookEndpoint: '/api/webhooks/stripe',
          webhookSecret: 'whsec_test_1234567890abcdef',
          isLive: false,
          supportedCurrencies: ['USD', 'EUR', 'GBP'],
          paymentMethods: ['card', 'bank_transfer']
        },
        isActive: true,
        healthCheck: {
          lastChecked: Timestamp.fromDate(this.addHours(now, -1)),
          status: 'healthy',
          message: 'All systems operational'
        },
        metadata: {
          version: '2023-10-16',
          documentation: 'https://stripe.com/docs',
          supportContact: 'stripe-support@epsx.com'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -90)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -1))
      },
      {
        id: 'sendgrid_integration',
        name: 'SendGrid Email Service',
        type: 'communication',
        config: {
          apiKey: 'SG.1234567890abcdef',
          fromEmail: 'noreply@epsx.com',
          fromName: 'EPSX Platform',
          templateEngine: 'handlebars',
          trackingEnabled: true,
          clickTracking: true,
          openTracking: true,
          unsubscribeTracking: true
        },
        isActive: true,
        healthCheck: {
          lastChecked: Timestamp.fromDate(this.addHours(now, -2)),
          status: 'healthy',
          message: 'Email delivery operational'
        },
        metadata: {
          version: 'v3',
          documentation: 'https://sendgrid.com/docs',
          supportContact: 'sendgrid-support@epsx.com'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -60)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -2))
      },
      {
        id: 'aws_s3_integration',
        name: 'AWS S3 Storage',
        type: 'storage',
        config: {
          region: 'us-west-2',
          bucket: 'epsx-uploads',
          accessKeyId: 'AKIA1234567890ABCDEF',
          cdnEnabled: true,
          cdnUrl: 'https://cdn.epsx.com',
          encryption: 'AES256',
          versioning: true,
          lifecycleRules: {
            transitionToIA: 30, // days
            transitionToGlacier: 90, // days
            expiration: 2555 // days (7 years)
          }
        },
        isActive: true,
        healthCheck: {
          lastChecked: Timestamp.fromDate(this.addHours(now, -3)),
          status: 'healthy',
          message: 'Storage service operational'
        },
        metadata: {
          version: '2006-03-01',
          documentation: 'https://docs.aws.amazon.com/s3/',
          supportContact: 'aws-support@epsx.com'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -75)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -3))
      },
      {
        id: 'google_analytics',
        name: 'Google Analytics 4',
        type: 'analytics',
        config: {
          measurementId: 'G-1234567890',
          apiSecret: 'abcdef1234567890',
          streamId: '1234567890',
          enableEcommerce: true,
          enableEnhancedMeasurement: true,
          customDimensions: [
            { name: 'user_role', scope: 'user' },
            { name: 'package_level', scope: 'user' },
            { name: 'organization_id', scope: 'user' }
          ],
          customMetrics: [
            { name: 'api_calls', type: 'integer' },
            { name: 'report_exports', type: 'integer' }
          ]
        },
        isActive: true,
        healthCheck: {
          lastChecked: Timestamp.fromDate(this.addHours(now, -4)),
          status: 'healthy',
          message: 'Analytics tracking active'
        },
        metadata: {
          version: 'GA4',
          documentation: 'https://developers.google.com/analytics/devguides/collection/ga4',
          supportContact: 'analytics-support@epsx.com'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -50)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -4))
      },
      {
        id: 'slack_integration',
        name: 'Slack Notifications',
        type: 'communication',
        config: {
          botToken: 'xoxb-1234567890-abcdefghijklmnop',
          signingSecret: '1234567890abcdef',
          channels: {
            alerts: '#alerts',
            general: '#general',
            support: '#support'
          },
          enableMentions: true,
          enableThreads: true,
          messageFormat: 'blocks',
          enableEmojis: true
        },
        isActive: false,
        healthCheck: {
          lastChecked: Timestamp.fromDate(this.addDays(now, -1)),
          status: 'down',
          message: 'Bot token expired'
        },
        metadata: {
          version: '1.0',
          documentation: 'https://api.slack.com/start',
          supportContact: 'slack-support@epsx.com'
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -30)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -1))
      }
    ];

    await this.seedCollection('integrations', integrations, 'id');
  }
}
