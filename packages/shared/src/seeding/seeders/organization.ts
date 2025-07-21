import { Timestamp } from 'firebase/firestore';
import { BaseSeeder } from './base';
import type { Organization, Invitation, UserPreferences, SeedResult } from '../types';

export class OrganizationSeeder extends BaseSeeder {
  get collectionName(): string {
    return 'organizations';
  }

  async seed(): Promise<SeedResult> {
    try {
      await this.seedOrganizations();
      await this.seedInvitations();
      await this.seedUserPreferences();

      return {
        success: true,
        collection: 'organizations',
        count: 2 + 2 + 3 // orgs + invitations + preferences
      };
    } catch (error) {
      return {
        success: false,
        collection: 'organizations',
        count: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  private async seedOrganizations() {
    this.log('Seeding organizations...');
    
    const organizations: Organization[] = [
      {
        id: 'org_001',
        name: 'EPSX Corporation',
        slug: 'epsx-corp',
        logo: '/assets/logos/epsx-logo.png',
        settings: {
          allowUserRegistration: true,
          requireEmailVerification: true,
          allowGuestAccess: false,
          defaultRole: 'viewer',
          passwordPolicy: {
            minLength: 8,
            requireUppercase: true,
            requireLowercase: true,
            requireNumbers: true,
            requireSpecialChars: true,
            maxAge: 90,
            preventReuse: 5
          },
          branding: {
            primaryColor: '#2563eb',
            logo: '/assets/logos/epsx-logo.png',
            favicon: '/assets/favicons/favicon.ico'
          }
        },
        subscription: {
          plan: 'ENTERPRISE',
          status: 'active',
          startDate: Timestamp.fromDate(new Date('2024-01-01')),
          endDate: Timestamp.fromDate(new Date('2025-12-31')),
          autoRenew: true,
          paymentMethod: 'credit_card'
        },
        metadata: {
          industry: 'Technology',
          size: 'medium',
          country: 'United States',
          timezone: 'America/Los_Angeles'
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'org_002',
        name: 'Demo Organization',
        slug: 'demo-org',
        logo: undefined,
        settings: {
          allowUserRegistration: true,
          requireEmailVerification: false,
          allowGuestAccess: true,
          defaultRole: 'viewer',
          passwordPolicy: {
            minLength: 6,
            requireUppercase: false,
            requireLowercase: true,
            requireNumbers: true,
            requireSpecialChars: false
          },
          branding: {
            primaryColor: '#059669'
          }
        },
        subscription: {
          plan: 'FREE',
          status: 'trial',
          startDate: Timestamp.now(),
          endDate: Timestamp.fromDate(this.addDays(new Date(), 30)),
          autoRenew: false
        },
        metadata: {
          industry: 'Demo',
          size: 'small',
          country: 'United States',
          timezone: 'America/New_York'
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('organizations', organizations, 'id');
  }

  private async seedInvitations() {
    this.log('Seeding invitations...');
    
    const now = new Date();
    const invitations: Invitation[] = [
      {
        id: 'inv_001',
        email: 'newuser@epsx.com',
        organizationId: 'org_001',
        invitedBy: 'admin-001',
        roles: ['editor'],
        status: 'pending',
        token: 'inv_token_' + Math.random().toString(36).substr(2, 16),
        expiresAt: Timestamp.fromDate(this.addDays(now, 7)),
        metadata: {
          personalMessage: 'Welcome to our EPSX team! We are excited to have you join us.',
          customData: {
            department: 'Content',
            startDate: this.addDays(now, 3).toISOString()
          }
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'inv_002',
        email: 'developer@example.com',
        organizationId: 'org_001',
        invitedBy: 'manager-001',
        roles: ['api-user', 'viewer'],
        status: 'pending',
        token: 'inv_token_' + Math.random().toString(36).substr(2, 16),
        expiresAt: Timestamp.fromDate(this.addDays(now, 14)),
        metadata: {
          personalMessage: 'Join our development team for API integration work.',
          customData: {
            department: 'Engineering',
            projectCode: 'EPSX-2025'
          }
        },
        createdAt: Timestamp.fromDate(this.addDays(now, -1)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -1))
      }
    ];

    await this.seedCollection('invitations', invitations, 'id');
  }

  private async seedUserPreferences() {
    this.log('Seeding user preferences...');
    
    const userPreferences: UserPreferences[] = [
      {
        id: 'pref_admin_001',
        userId: 'admin-001',
        ui: {
          theme: 'dark',
          language: 'en',
          timezone: 'America/Los_Angeles',
          dateFormat: 'MM/DD/YYYY',
          timeFormat: '12',
          notifications: {
            email: true,
            push: true,
            sms: false,
            desktop: true
          }
        },
        privacy: {
          profileVisibility: 'organization',
          showEmail: false,
          showPhone: false,
          allowMarketing: true,
          allowAnalytics: true
        },
        features: {
          enableBetaFeatures: true,
          enableAdvancedMode: true,
          customSettings: {
            dashboardLayout: 'grid',
            autoSave: true,
            showTooltips: false
          }
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'pref_manager_001',
        userId: 'manager-001',
        ui: {
          theme: 'light',
          language: 'en',
          timezone: 'America/New_York',
          dateFormat: 'YYYY-MM-DD',
          timeFormat: '24',
          notifications: {
            email: true,
            push: true,
            sms: true,
            desktop: false
          }
        },
        privacy: {
          profileVisibility: 'organization',
          showEmail: true,
          showPhone: true,
          allowMarketing: false,
          allowAnalytics: true
        },
        features: {
          enableBetaFeatures: false,
          enableAdvancedMode: true,
          customSettings: {
            dashboardLayout: 'list',
            autoSave: true,
            showTooltips: true
          }
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'pref_beta_001',
        userId: 'beta-001',
        ui: {
          theme: 'auto',
          language: 'en',
          timezone: 'America/Chicago',
          dateFormat: 'DD/MM/YYYY',
          timeFormat: '24',
          notifications: {
            email: true,
            push: true,
            sms: false,
            desktop: true
          }
        },
        privacy: {
          profileVisibility: 'public',
          showEmail: false,
          showPhone: false,
          allowMarketing: true,
          allowAnalytics: true
        },
        features: {
          enableBetaFeatures: true,
          enableAdvancedMode: false,
          customSettings: {
            dashboardLayout: 'grid',
            autoSave: false,
            showTooltips: true
          }
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('userPreferences', userPreferences, 'id');
  }
}
