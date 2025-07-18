import type { Policy } from './types';
import { ResourceType, ActionType, SystemPolicy } from './types';

/**
 * AWS IAM-inspired built-in policy templates
 */
export class PolicyTemplates {
  /**
   * Full access policy - equivalent to AWS AdministratorAccess
   */
  static createFullAccessPolicy(): Policy {
    return {
      id: 'epsx-full-access',
      name: 'EPSXFullAccess',
      description: 'Provides full access to all EPSX services and resources',
      version: '2024-01-01',
      statement: [
        {
          sid: 'FullAccess',
          effect: 'Allow',
          actions: ['*'],
          resources: ['*'],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Read-only access policy
   */
  static createReadOnlyPolicy(): Policy {
    return {
      id: 'epsx-read-only',
      name: 'EPSXReadOnly',
      description: 'Provides read-only access to all EPSX services',
      version: '2024-01-01',
      statement: [
        {
          sid: 'ReadOnlyAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
          ],
          resources: ['*'],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Stock analytics access policy
   */
  static createStockAnalyticsPolicy(): Policy {
    return {
      id: 'epsx-stock-analytics',
      name: 'EPSXStockAnalytics',
      description: 'Provides access to stock analytics and research features',
      version: '2024-01-01',
      statement: [
        {
          sid: 'StockAnalyticsAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
            `${ActionType.ANALYZE}`,
            `${ActionType.RANK}`,
            `${ActionType.SCREEN}`,
            `${ActionType.EXPORT}`,
          ],
          resources: [
            `epsx:stock:*:*:${ResourceType.STOCK_RANKINGS}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_ANALYTICS}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_RESEARCH}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_SCREENER}:*`,
          ],
        },
        {
          sid: 'MarketDataAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
          ],
          resources: [
            `epsx:market:*:*:${ResourceType.MARKET_DATA}:*`,
          ],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Bronze tier policy (limited access)
   */
  static createBronzeTierPolicy(): Policy {
    return {
      id: 'epsx-bronze-tier',
      name: 'EPSXBronzeTier',
      description: 'Bronze tier access with limited stock rankings (5 items)',
      version: '2024-01-01',
      statement: [
        {
          sid: 'BronzeStockAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
          ],
          resources: [
            `epsx:stock:*:*:${ResourceType.STOCK_RANKINGS}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_ANALYTICS}:*`,
          ],
          conditions: {
            'NumericLessThan': {
              key: 'epsx:RequestedLimit',
              operator: 'NumericLessThan',
              value: 5,
            },
          },
        },
        {
          sid: 'BasicUserAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.UPDATE}`,
          ],
          resources: [
            `epsx:user:*:*:${ResourceType.USER_PROFILE}:\${epsx:userid}`,
            `epsx:user:*:*:${ResourceType.USER_SETTINGS}:\${epsx:userid}`,
          ],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Silver tier policy (enhanced access)
   */
  static createSilverTierPolicy(): Policy {
    return {
      id: 'epsx-silver-tier',
      name: 'EPSXSilverTier',
      description: 'Silver tier access with enhanced stock rankings (25 items)',
      version: '2024-01-01',
      statement: [
        {
          sid: 'SilverStockAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
            `${ActionType.ANALYZE}`,
            `${ActionType.EXPORT}`,
          ],
          resources: [
            `epsx:stock:*:*:${ResourceType.STOCK_RANKINGS}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_ANALYTICS}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_RESEARCH}:*`,
          ],
          conditions: {
            'NumericLessThan': {
              key: 'epsx:RequestedLimit',
              operator: 'NumericLessThan',
              value: 25,
            },
          },
        },
        {
          sid: 'EnhancedUserAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.UPDATE}`,
            `${ActionType.CREATE}`,
          ],
          resources: [
            `epsx:user:*:*:${ResourceType.USER_PROFILE}:\${epsx:userid}`,
            `epsx:user:*:*:${ResourceType.USER_SETTINGS}:\${epsx:userid}`,
          ],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Gold tier policy (premium access)
   */
  static createGoldTierPolicy(): Policy {
    return {
      id: 'epsx-gold-tier',
      name: 'EPSXGoldTier',
      description: 'Gold tier access with premium stock rankings (50 items)',
      version: '2024-01-01',
      statement: [
        {
          sid: 'GoldStockAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
            `${ActionType.ANALYZE}`,
            `${ActionType.RANK}`,
            `${ActionType.SCREEN}`,
            `${ActionType.EXPORT}`,
          ],
          resources: [
            `epsx:stock:*:*:${ResourceType.STOCK_RANKINGS}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_ANALYTICS}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_RESEARCH}:*`,
            `epsx:stock:*:*:${ResourceType.STOCK_SCREENER}:*`,
          ],
          conditions: {
            'NumericLessThan': {
              key: 'epsx:RequestedLimit',
              operator: 'NumericLessThan',
              value: 50,
            },
          },
        },
        {
          sid: 'PremiumMarketAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
          ],
          resources: [
            `epsx:market:*:*:${ResourceType.MARKET_DATA}:*`,
            `epsx:market:*:*:${ResourceType.REALTIME_DATA}:*`,
          ],
        },
        {
          sid: 'APIAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
          ],
          resources: [
            `epsx:api:*:*:${ResourceType.API_LIMITS}:*`,
          ],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Platinum tier policy (enterprise access)
   */
  static createPlatinumTierPolicy(): Policy {
    return {
      id: 'epsx-platinum-tier',
      name: 'EPSXPlatinumTier',
      description: 'Platinum tier access with enterprise stock rankings (100 items)',
      version: '2024-01-01',
      statement: [
        {
          sid: 'PlatinumStockAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
            `${ActionType.ANALYZE}`,
            `${ActionType.RANK}`,
            `${ActionType.SCREEN}`,
            `${ActionType.EXPORT}`,
            `${ActionType.IMPORT}`,
          ],
          resources: [
            `epsx:stock:*:*:*:*`, // All stock resources
          ],
          conditions: {
            'NumericLessThan': {
              key: 'epsx:RequestedLimit',
              operator: 'NumericLessThan',
              value: 100,
            },
          },
        },
        {
          sid: 'EnterpriseMarketAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.VIEW}`,
            `${ActionType.MANAGE}`,
          ],
          resources: [
            `epsx:market:*:*:*:*`, // All market resources
          ],
        },
        {
          sid: 'AdvancedAPIAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.CREATE}`,
            `${ActionType.UPDATE}`,
            `${ActionType.DELETE}`,
          ],
          resources: [
            `epsx:api:*:*:${ResourceType.API_LIMITS}:*`,
            `epsx:api:*:*:${ResourceType.API_WEBHOOKS}:*`,
          ],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Admin policy
   */
  static createAdminPolicy(): Policy {
    return {
      id: 'epsx-admin',
      name: 'EPSXAdmin',
      description: 'Administrative access to user and system management',
      version: '2024-01-01',
      statement: [
        {
          sid: 'AdminUserManagement',
          effect: 'Allow',
          actions: [
            `${ActionType.CREATE}`,
            `${ActionType.READ}`,
            `${ActionType.UPDATE}`,
            `${ActionType.DELETE}`,
            `${ActionType.LIST}`,
            `${ActionType.MANAGE}`,
          ],
          resources: [
            `epsx:admin:*:*:${ResourceType.ADMIN_USERS}:*`,
            `epsx:admin:*:*:${ResourceType.ADMIN_ANALYTICS}:*`,
            `epsx:admin:*:*:${ResourceType.ADMIN_SYSTEM}:*`,
          ],
        },
        {
          sid: 'AdminBillingAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.MANAGE}`,
          ],
          resources: [
            `epsx:billing:*:*:${ResourceType.PAYMENT_HISTORY}:*`,
            `epsx:billing:*:*:${ResourceType.BILLING_MANAGEMENT}:*`,
          ],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Billing access policy
   */
  static createBillingPolicy(): Policy {
    return {
      id: 'epsx-billing',
      name: 'EPSXBilling',
      description: 'Access to billing and payment features',
      version: '2024-01-01',
      statement: [
        {
          sid: 'BillingAccess',
          effect: 'Allow',
          actions: [
            `${ActionType.READ}`,
            `${ActionType.LIST}`,
            `${ActionType.UPDATE}`,
            `${ActionType.CREATE}`,
          ],
          resources: [
            `epsx:billing:*:*:${ResourceType.PAYMENT_HISTORY}:\${epsx:userid}`,
            `epsx:billing:*:*:${ResourceType.PAYMENT_METHODS}:\${epsx:userid}`,
            `epsx:user:*:*:${ResourceType.USER_SUBSCRIPTION}:\${epsx:userid}`,
          ],
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Time-based access policy (e.g., business hours only)
   */
  static createTimeBasedPolicy(): Policy {
    return {
      id: 'epsx-time-based',
      name: 'EPSXTimeBasedAccess',
      description: 'Access restricted to business hours',
      version: '2024-01-01',
      statement: [
        {
          sid: 'BusinessHoursAccess',
          effect: 'Allow',
          actions: ['*'],
          resources: ['*'],
          conditions: {
            'DateGreaterThan': {
              key: 'epsx:CurrentTime',
              operator: 'DateGreaterThan',
              value: '09:00:00',
            },
            'DateLessThan': {
              key: 'epsx:CurrentTime',
              operator: 'DateLessThan',
              value: '17:00:00',
            },
          },
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * IP-based access policy
   */
  static createIPBasedPolicy(allowedIPs: string[]): Policy {
    return {
      id: 'epsx-ip-based',
      name: 'EPSXIPBasedAccess',
      description: 'Access restricted to specific IP addresses',
      version: '2024-01-01',
      statement: [
        {
          sid: 'IPRestrictedAccess',
          effect: 'Allow',
          actions: ['*'],
          resources: ['*'],
          conditions: {
            'IpAddress': {
              key: 'epsx:SourceIp',
              operator: 'IpAddress',
              value: allowedIPs.join(','),
            },
          },
        },
      ],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Get policy by system policy enum
   */
  static getPolicyBySystemType(systemPolicy: SystemPolicy): Policy {
    switch (systemPolicy) {
      case SystemPolicy.FULL_ACCESS:
        return this.createFullAccessPolicy();
      case SystemPolicy.READ_ONLY:
        return this.createReadOnlyPolicy();
      case SystemPolicy.ANALYTICS_ACCESS:
        return this.createStockAnalyticsPolicy();
      case SystemPolicy.ADMIN_ACCESS:
        return this.createAdminPolicy();
      case SystemPolicy.BILLING_ACCESS:
        return this.createBillingPolicy();
      case SystemPolicy.STOCK_TRADER_ACCESS:
        return this.createGoldTierPolicy();
      case SystemPolicy.MARKET_DATA_ACCESS:
        return this.createPlatinumTierPolicy();
      case SystemPolicy.POWER_USER:
        return this.createPlatinumTierPolicy();
      default:
        return this.createReadOnlyPolicy();
    }
  }

  /**
   * Get policy by user tier
   */
  static getPolicyByTier(tier: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM'): Policy {
    switch (tier) {
      case 'BRONZE':
        return this.createBronzeTierPolicy();
      case 'SILVER':
        return this.createSilverTierPolicy();
      case 'GOLD':
        return this.createGoldTierPolicy();
      case 'PLATINUM':
        return this.createPlatinumTierPolicy();
      default:
        return this.createBronzeTierPolicy();
    }
  }
}

export default PolicyTemplates;
