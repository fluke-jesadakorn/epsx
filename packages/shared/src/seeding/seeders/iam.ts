import { readFileSync } from 'fs';
import { join } from 'path';
import { Timestamp } from 'firebase/firestore';
import { BaseSeeder } from './base';
import type { Role, Permission, User, UserSession, AuditLog, SeedResult } from '../types';

export class IAMSeeder extends BaseSeeder {
  get collectionName(): string {
    return 'iam';
  }

  async seed(): Promise<SeedResult> {
    try {
      // Load static data from JSON files
      const { roles, permissions } = this.loadStaticData();

      // Seed roles and permissions first
      await this.seedRoles(roles);
      await this.seedPermissions(permissions);

      // Then seed dynamic user data
      await this.seedUsers();
      await this.seedUserSessions();
      await this.seedAuditLogs();

      return {
        success: true,
        collection: 'iam',
        count: roles.length + permissions.length + 3 + 2 + 3 // roles + perms + users + sessions + audits
      };
    } catch (error) {
      return {
        success: false,
        collection: 'iam',
        count: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  private loadStaticData() {
    const dataPath = join(__dirname, '..', 'data');
    
    const roles = JSON.parse(readFileSync(join(dataPath, 'roles.json'), 'utf8'));
    const permissions = JSON.parse(readFileSync(join(dataPath, 'permissions.json'), 'utf8'));
    
    return { roles, permissions };
  }

  private async seedRoles(rolesData: any[]) {
    this.log('Seeding roles...');
    
    const roles: Role[] = rolesData.map(role => ({
      ...role,
      createdAt: Timestamp.now(),
      updatedAt: Timestamp.now()
    }));

    await this.seedCollection('roles', roles, 'id');
  }

  private async seedPermissions(permissionsData: any[]) {
    this.log('Seeding permissions...');
    
    const permissions: Permission[] = permissionsData.map(permission => ({
      ...permission,
      createdAt: Timestamp.now(),
      updatedAt: Timestamp.now()
    }));

    await this.seedCollection('permissions', permissions, 'id');
  }

  private async seedUsers() {
    this.log('Seeding users...');
    
    const users: User[] = [
      {
        id: 'user_admin_001',
        uid: 'admin-001',
        email: 'admin@epsx.com',
        profile: {
          firstName: 'System',
          lastName: 'Administrator',
          avatar: null,
          department: 'IT',
          position: 'System Administrator',
          phone: '+1-555-0001',
          bio: 'System administrator with full access to all platform features.',
          location: 'San Francisco, CA',
          lastLogin: null
        },
        roles: ['admin'],
        packageLevel: 'ENTERPRISE',
        permissions: {
          computed: ['*'],
          explicit: [],
          inherited: ['*']
        },
        organizationId: 'org_001',
        isActive: true,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'user_manager_001',
        uid: 'manager-001',
        email: 'manager@epsx.com',
        profile: {
          firstName: 'Sarah',
          lastName: 'Johnson',
          avatar: null,
          department: 'Operations',
          position: 'Operations Manager',
          phone: '+1-555-0002',
          bio: 'Operations manager responsible for team coordination and project oversight.',
          location: 'New York, NY',
          lastLogin: null
        },
        roles: ['manager'],
        packageLevel: 'GOLD',
        permissions: {
          computed: ['user:read', 'user:write', 'content:read', 'content:write', 'analytics:read', 'organization:read'],
          explicit: [],
          inherited: ['user:read', 'user:write', 'content:read', 'content:write', 'analytics:read', 'organization:read']
        },
        organizationId: 'org_001',
        isActive: true,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'user_beta_001',
        uid: 'beta-001',
        email: 'beta@epsx.com',
        profile: {
          firstName: 'Alex',
          lastName: 'Chen',
          avatar: null,
          department: 'QA',
          position: 'Beta Tester',
          phone: '+1-555-0003',
          bio: 'Beta tester with access to experimental features and early releases.',
          location: 'Austin, TX',
          lastLogin: null
        },
        roles: ['beta-tester'],
        packageLevel: 'SILVER',
        permissions: {
          computed: ['content:read', 'content:write', 'feature:beta', 'feature:advanced-analytics', 'analytics:read'],
          explicit: [],
          inherited: ['content:read', 'content:write', 'feature:beta', 'feature:advanced-analytics', 'analytics:read']
        },
        organizationId: 'org_001',
        isActive: true,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('users', users, 'id');
  }

  private async seedUserSessions() {
    this.log('Seeding user sessions...');
    
    const now = new Date();
    const sessions: UserSession[] = [
      {
        id: 'session_001',
        sessionId: 'sess_admin_001',
        userId: 'admin-001',
        deviceInfo: {
          userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
          ip: '192.168.1.100',
          device: 'MacBook Pro',
          browser: 'Chrome',
          os: 'macOS',
          location: {
            country: 'United States',
            city: 'San Francisco',
            timezone: 'America/Los_Angeles'
          }
        },
        isActive: true,
        lastActivity: Timestamp.now(),
        expiresAt: Timestamp.fromDate(this.addHours(now, 24)),
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now(),
        metadata: {
          loginMethod: 'email_password',
          twoFactorUsed: false
        }
      },
      {
        id: 'session_002',
        sessionId: 'sess_manager_001',
        userId: 'manager-001',
        deviceInfo: {
          userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
          ip: '192.168.1.101',
          device: 'Windows PC',
          browser: 'Chrome',
          os: 'Windows',
          location: {
            country: 'United States',
            city: 'New York',
            timezone: 'America/New_York'
          }
        },
        isActive: true,
        lastActivity: Timestamp.fromDate(this.addHours(now, -2)),
        expiresAt: Timestamp.fromDate(this.addHours(now, 22)),
        createdAt: Timestamp.fromDate(this.addHours(now, -2)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -1)),
        metadata: {
          loginMethod: 'sso',
          provider: 'google'
        }
      }
    ];

    await this.seedCollection('userSessions', sessions, 'id');
  }

  private async seedAuditLogs() {
    this.log('Seeding audit logs...');
    
    const now = new Date();
    const auditLogs: AuditLog[] = [
      {
        id: 'audit_001',
        userId: 'admin-001',
        action: 'SYSTEM_INITIALIZED',
        resource: 'IAM_SYSTEM',
        details: {
          message: 'IAM system initialized with seed data',
          version: '1.0.0',
          collections: ['roles', 'permissions', 'users']
        },
        timestamp: Timestamp.now(),
        metadata: {
          ip: '127.0.0.1',
          userAgent: 'System/Initialization',
          sessionId: 'system',
          organizationId: 'org_001'
        },
        severity: 'low',
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'audit_002',
        userId: 'admin-001',
        action: 'USER_LOGIN',
        resource: 'AUTH_SYSTEM',
        details: {
          method: 'email_password',
          success: true,
          ipAddress: '192.168.1.100'
        },
        timestamp: Timestamp.fromDate(this.addHours(now, -1)),
        metadata: {
          ip: '192.168.1.100',
          userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
          sessionId: 'sess_admin_001',
          organizationId: 'org_001'
        },
        severity: 'low',
        createdAt: Timestamp.fromDate(this.addHours(now, -1)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -1))
      },
      {
        id: 'audit_003',
        userId: 'manager-001',
        action: 'USER_LOGIN',
        resource: 'AUTH_SYSTEM',
        details: {
          method: 'sso',
          provider: 'google',
          success: true,
          ipAddress: '192.168.1.101'
        },
        timestamp: Timestamp.fromDate(this.addHours(now, -2)),
        metadata: {
          ip: '192.168.1.101',
          userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
          sessionId: 'sess_manager_001',
          organizationId: 'org_001'
        },
        severity: 'low',
        createdAt: Timestamp.fromDate(this.addHours(now, -2)),
        updatedAt: Timestamp.fromDate(this.addHours(now, -2))
      }
    ];

    await this.seedCollection('auditLogs', auditLogs, 'id');
  }
}
