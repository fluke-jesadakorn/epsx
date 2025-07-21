import { Timestamp } from 'firebase/firestore';
import { BaseSeeder } from './base';
import type { Notification, EmailTemplate, MessageQueue, SeedResult } from '../types';

export class NotificationSeeder extends BaseSeeder {
  get collectionName(): string {
    return 'notifications';
  }

  async seed(): Promise<SeedResult> {
    try {
      await this.seedEmailTemplates();
      await this.seedNotifications();
      await this.seedMessageQueue();

      return {
        success: true,
        collection: 'notifications',
        count: 6 + 8 + 4 // templates + notifications + queue
      };
    } catch (error) {
      return {
        success: false,
        collection: 'notifications',
        count: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  private async seedEmailTemplates() {
    this.log('Seeding email templates...');
    
    const emailTemplates: EmailTemplate[] = [
      {
        id: 'welcome_email',
        name: 'Welcome Email',
        subject: 'Welcome to {{organizationName}} - Get Started with EPSX',
        body: `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Welcome to {{organizationName}}</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #2563eb;">Welcome {{firstName}}!</h1>
        
        <p>We're excited to have you join {{organizationName}} on the EPSX platform.</p>
        
        <p>Here's what you can do to get started:</p>
        <ul>
            <li>Complete your profile setup</li>
            <li>Explore the dashboard</li>
            <li>Connect your first data source</li>
            <li>Create your first report</li>
        </ul>
        
        <a href="{{dashboardUrl}}" style="background: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">
            Go to Dashboard
        </a>
        
        <p>If you have any questions, don't hesitate to reach out to our support team.</p>
        
        <p>Best regards,<br>The EPSX Team</p>
    </div>
</body>
</html>`,
        variables: ['organizationName', 'firstName', 'dashboardUrl'],
        isActive: true,
        category: 'onboarding',
        metadata: {
          lastUsed: Timestamp.now(),
          usageCount: 25
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'invitation_email',
        name: 'Team Invitation',
        subject: '{{inviterName}} invited you to join {{organizationName}}',
        body: `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Team Invitation</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #2563eb;">You're Invited!</h1>
        
        <p>{{inviterName}} has invited you to join {{organizationName}} on the EPSX platform.</p>
        
        {{#if personalMessage}}
        <blockquote style="border-left: 4px solid #2563eb; padding-left: 16px; margin: 20px 0; font-style: italic;">
            "{{personalMessage}}"
        </blockquote>
        {{/if}}
        
        <p>As a team member, you'll have access to:</p>
        <ul>
            <li>Real-time analytics and dashboards</li>
            <li>Collaborative reporting tools</li>
            <li>Team data sources and insights</li>
            <li>Advanced features based on your role</li>
        </ul>
        
        <a href="{{invitationUrl}}" style="background: #10b981; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">
            Accept Invitation
        </a>
        
        <p><small>This invitation expires on {{expiryDate}}.</small></p>
        
        <p>Best regards,<br>The EPSX Team</p>
    </div>
</body>
</html>`,
        variables: ['inviterName', 'organizationName', 'personalMessage', 'invitationUrl', 'expiryDate'],
        isActive: true,
        category: 'invitation',
        metadata: {
          lastUsed: Timestamp.fromDate(this.addDays(new Date(), -1)),
          usageCount: 12
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'password_reset',
        name: 'Password Reset',
        subject: 'Reset your EPSX password',
        body: `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Password Reset</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #ef4444;">Password Reset Request</h1>
        
        <p>Hi {{firstName}},</p>
        
        <p>We received a request to reset your password for your EPSX account.</p>
        
        <a href="{{resetUrl}}" style="background: #ef4444; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">
            Reset Password
        </a>
        
        <p><strong>This link will expire in 1 hour.</strong></p>
        
        <p>If you didn't request this password reset, please ignore this email or contact support if you have concerns.</p>
        
        <p>For security, this link can only be used once.</p>
        
        <p>Best regards,<br>The EPSX Team</p>
    </div>
</body>
</html>`,
        variables: ['firstName', 'resetUrl'],
        isActive: true,
        category: 'security',
        metadata: {
          lastUsed: Timestamp.fromDate(this.addDays(new Date(), -3)),
          usageCount: 8
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'usage_warning',
        name: 'Usage Limit Warning',
        subject: 'EPSX Usage Alert - Approaching Your {{limitType}} Limit',
        body: `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Usage Warning</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #f59e0b;">Usage Alert</h1>
        
        <p>Hi {{firstName}},</p>
        
        <p>You're approaching your {{limitType}} limit for this billing period.</p>
        
        <div style="background: #fef3c7; border: 1px solid #f59e0b; border-radius: 6px; padding: 16px; margin: 20px 0;">
            <strong>Current Usage:</strong> {{currentUsage}} / {{limit}}<br>
            <strong>Percentage Used:</strong> {{percentage}}%<br>
            <strong>Billing Period:</strong> {{billingPeriod}}
        </div>
        
        <p>To avoid service interruption:</p>
        <ul>
            <li>Consider upgrading your plan</li>
            <li>Monitor your usage more closely</li>
            <li>Contact support for assistance</li>
        </ul>
        
        <a href="{{upgradeUrl}}" style="background: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">
            Upgrade Plan
        </a>
        
        <p>Best regards,<br>The EPSX Team</p>
    </div>
</body>
</html>`,
        variables: ['firstName', 'limitType', 'currentUsage', 'limit', 'percentage', 'billingPeriod', 'upgradeUrl'],
        isActive: true,
        category: 'billing',
        metadata: {
          lastUsed: Timestamp.fromDate(this.addDays(new Date(), -5)),
          usageCount: 3
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'report_ready',
        name: 'Report Generated',
        subject: 'Your {{reportName}} report is ready',
        body: `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Report Ready</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #10b981;">Report Generated Successfully</h1>
        
        <p>Hi {{firstName}},</p>
        
        <p>Your report "{{reportName}}" has been generated and is ready for download.</p>
        
        <div style="background: #f0f9f4; border: 1px solid #10b981; border-radius: 6px; padding: 16px; margin: 20px 0;">
            <strong>Report Details:</strong><br>
            <strong>Name:</strong> {{reportName}}<br>
            <strong>Format:</strong> {{format}}<br>
            <strong>Generated:</strong> {{generatedAt}}<br>
            <strong>Expires:</strong> {{expiresAt}}
        </div>
        
        <a href="{{downloadUrl}}" style="background: #10b981; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">
            Download Report
        </a>
        
        <p><small>This download link will expire in 7 days.</small></p>
        
        <p>Best regards,<br>The EPSX Team</p>
    </div>
</body>
</html>`,
        variables: ['firstName', 'reportName', 'format', 'generatedAt', 'expiresAt', 'downloadUrl'],
        isActive: true,
        category: 'reports',
        metadata: {
          lastUsed: Timestamp.now(),
          usageCount: 45
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'maintenance_notice',
        name: 'Maintenance Notice',
        subject: 'Scheduled Maintenance - {{maintenanceDate}}',
        body: `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Maintenance Notice</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #f59e0b;">Scheduled Maintenance Notice</h1>
        
        <p>Hi {{firstName}},</p>
        
        <p>We're writing to inform you about upcoming scheduled maintenance for the EPSX platform.</p>
        
        <div style="background: #fef3c7; border: 1px solid #f59e0b; border-radius: 6px; padding: 16px; margin: 20px 0;">
            <strong>Maintenance Details:</strong><br>
            <strong>Date:</strong> {{maintenanceDate}}<br>
            <strong>Time:</strong> {{maintenanceTime}}<br>
            <strong>Duration:</strong> {{duration}}<br>
            <strong>Timezone:</strong> {{timezone}}
        </div>
        
        <p>During this time:</p>
        <ul>
            <li>The platform will be temporarily unavailable</li>
            <li>No data will be lost</li>
            <li>All services will resume automatically</li>
            <li>{{additionalInfo}}</li>
        </ul>
        
        <p>We apologize for any inconvenience and appreciate your patience.</p>
        
        <p>Best regards,<br>The EPSX Team</p>
    </div>
</body>
</html>`,
        variables: ['firstName', 'maintenanceDate', 'maintenanceTime', 'duration', 'timezone', 'additionalInfo'],
        isActive: true,
        category: 'system',
        metadata: {
          lastUsed: Timestamp.fromDate(this.addDays(new Date(), -30)),
          usageCount: 1
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('emailTemplates', emailTemplates, 'id');
  }

  private async seedNotifications() {
    this.log('Seeding notifications...');
    
    const now = new Date();
    const notifications: Notification[] = [
      {
        id: 'notif_001',
        userId: 'admin-001',
        type: 'system',
        title: 'Welcome to EPSX Platform',
        message: 'Your account has been successfully created. Complete your profile to get started.',
        data: {
          actionUrl: '/profile/setup',
          actionText: 'Complete Profile',
          metadata: {
            category: 'onboarding',
            importance: 'high'
          }
        },
        status: 'read',
        priority: 'normal',
        readAt: Timestamp.fromDate(this.addHours(now, -2)),
        createdAt: Timestamp.fromDate(this.addHours(now, -24)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -2))
      },
      {
        id: 'notif_002',
        userId: 'admin-001',
        type: 'system',
        title: 'New Feature: Advanced Analytics',
        message: 'Check out our new advanced analytics dashboard with ML-powered insights.',
        data: {
          actionUrl: '/analytics/advanced',
          actionText: 'Explore Analytics',
          metadata: {
            feature: 'advanced_analytics',
            version: '2.0'
          }
        },
        status: 'unread',
        priority: 'normal',
        createdAt: Timestamp.fromDate(this.addHours(now, -6)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -6))
      },
      {
        id: 'notif_003',
        userId: 'manager-001',
        type: 'user',
        title: 'Report Generation Complete',
        message: 'Your monthly analytics report has been generated and is ready for download.',
        data: {
          actionUrl: '/reports/download/monthly-analytics-2025-01',
          actionText: 'Download Report',
          metadata: {
            reportId: 'monthly-analytics-2025-01',
            format: 'PDF'
          }
        },
        status: 'unread',
        priority: 'normal',
        createdAt: Timestamp.fromDate(this.addHours(now, -1)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -1))
      },
      {
        id: 'notif_004',
        userId: 'manager-001',
        type: 'security',
        title: 'New Login Detected',
        message: 'A new login from Windows PC in New York was detected on your account.',
        data: {
          actionUrl: '/security/sessions',
          actionText: 'Review Sessions',
          metadata: {
            location: 'New York, NY',
            device: 'Windows PC',
            ip: '192.168.1.101'
          }
        },
        status: 'read',
        priority: 'high',
        readAt: Timestamp.fromDate(this.addHours(now, -1)),
        createdAt: Timestamp.fromDate(this.addHours(now, -2)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -1))
      },
      {
        id: 'notif_005',
        userId: 'beta-001',
        type: 'user',
        title: 'Beta Feature Feedback Request',
        message: 'We\'d love your feedback on the new dashboard widgets you\'ve been testing.',
        data: {
          actionUrl: '/feedback/beta-dashboard-widgets',
          actionText: 'Give Feedback',
          metadata: {
            feature: 'dashboard_widgets',
            testingPhase: 'beta'
          }
        },
        status: 'unread',
        priority: 'normal',
        createdAt: Timestamp.fromDate(this.addHours(now, -12)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -12))
      },
      {
        id: 'notif_006',
        userId: 'beta-001',
        type: 'system',
        title: 'Usage Limit Warning',
        message: 'You\'ve used 80% of your API calls for this billing period.',
        data: {
          actionUrl: '/billing/usage',
          actionText: 'View Usage',
          metadata: {
            limitType: 'apiCalls',
            percentage: 80,
            current: 4000,
            limit: 5000
          }
        },
        status: 'unread',
        priority: 'high',
        createdAt: Timestamp.fromDate(this.addHours(now, -4)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -4))
      },
      {
        id: 'notif_007',
        userId: 'admin-001',
        type: 'marketing',
        title: 'EPSX 2.1 Coming Soon',
        message: 'Get ready for enhanced collaboration tools and improved performance in our next release.',
        data: {
          actionUrl: '/changelog/v2.1',
          actionText: 'View Changelog',
          metadata: {
            version: '2.1',
            releaseDate: '2025-02-15'
          }
        },
        status: 'unread',
        priority: 'low',
        expiresAt: Timestamp.fromDate(this.addDays(now, 7)),
        createdAt: Timestamp.fromDate(this.addHours(now, -8)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -8))
      },
      {
        id: 'notif_008',
        userId: 'manager-001',
        type: 'system',
        title: 'Scheduled Maintenance Tonight',
        message: 'The platform will be offline for 30 minutes starting at 2:00 AM EST for routine maintenance.',
        data: {
          actionUrl: '/status',
          actionText: 'System Status',
          metadata: {
            maintenanceStart: '2025-01-22T02:00:00Z',
            duration: '30 minutes',
            timezone: 'EST'
          }
        },
        status: 'unread',
        priority: 'urgent',
        expiresAt: Timestamp.fromDate(this.addDays(now, 1)),
        createdAt: Timestamp.fromDate(this.addHours(now, -3)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -3))
      }
    ];

    await this.seedCollection('notifications', notifications, 'id');
  }

  private async seedMessageQueue() {
    this.log('Seeding message queue...');
    
    const now = new Date();
    const messageQueue: MessageQueue[] = [
      {
        id: 'msg_001',
        type: 'email',
        recipient: 'newuser@epsx.com',
        templateId: 'invitation_email',
        data: {
          inviterName: 'Sarah Johnson',
          organizationName: 'EPSX Corporation',
          personalMessage: 'Welcome to our EPSX team! We are excited to have you join us.',
          invitationUrl: 'https://epsx.com/invitations/accept/inv_token_abc123',
          expiryDate: 'January 29, 2025'
        },
        status: 'pending',
        scheduledAt: Timestamp.fromDate(this.addHours(now, 1)),
        attempts: 0,
        maxAttempts: 3,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'msg_002',
        type: 'email',
        recipient: 'beta@epsx.com',
        templateId: 'usage_warning',
        data: {
          firstName: 'Alex',
          limitType: 'API calls',
          currentUsage: '4000',
          limit: '5000',
          percentage: '80',
          billingPeriod: 'January 2025',
          upgradeUrl: 'https://epsx.com/billing/upgrade'
        },
        status: 'sent',
        scheduledAt: Timestamp.fromDate(this.addHours(now, -4)),
        sentAt: Timestamp.fromDate(this.addHours(now, -4)),
        attempts: 1,
        maxAttempts: 3,
        createdAt: Timestamp.fromDate(this.addHours(now, -5)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -4))
      },
      {
        id: 'msg_003',
        type: 'email',
        recipient: 'manager@epsx.com',
        templateId: 'report_ready',
        data: {
          firstName: 'Sarah',
          reportName: 'Monthly Analytics Report',
          format: 'PDF',
          generatedAt: 'January 21, 2025 at 3:30 PM',
          expiresAt: 'January 28, 2025',
          downloadUrl: 'https://epsx.com/reports/download/monthly-analytics-2025-01'
        },
        status: 'sent',
        scheduledAt: Timestamp.fromDate(this.addHours(now, -1)),
        sentAt: Timestamp.fromDate(this.addHours(now, -1)),
        attempts: 1,
        maxAttempts: 3,
        createdAt: Timestamp.fromDate(this.addHours(now, -2)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -1))
      },
      {
        id: 'msg_004',
        type: 'push',
        recipient: 'admin-001',
        data: {
          title: 'System Alert',
          body: 'CPU usage above 80% for 5 minutes',
          icon: '/icons/alert.png',
          badge: '/icons/badge.png',
          url: '/system/metrics'
        },
        status: 'failed',
        scheduledAt: Timestamp.fromDate(this.addHours(now, -6)),
        attempts: 3,
        maxAttempts: 3,
        lastError: 'Push service unavailable',
        createdAt: Timestamp.fromDate(this.addHours(now, -7)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -6))
      }
    ];

    await this.seedCollection('messageQueue', messageQueue, 'id');
  }
}
