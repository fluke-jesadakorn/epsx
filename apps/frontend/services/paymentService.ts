import { PaymentTier, type UserSubscription, type PaymentLimits } from '@/types/payment/plans';

export class PaymentService {
  static getApiFeaturesByTier(tier: PaymentTier): string[] {
    const featureMap = {
      [PaymentTier.BASIC]: ['Basic API access', 'Limited rankings', 'Community support'],
      [PaymentTier.SILVER]: ['Full access for 1 month', 'Priority support', 'Advanced features'],
      [PaymentTier.GOLD]: ['Extended access', 'Premium features', 'Priority support', 'Early access to new features'],
      [PaymentTier.PLATINUM]: ['Unlimited access', 'All premium features', 'VIP support', 'Early access to new features', 'Custom analytics']
    };
    return featureMap[tier] || [];
  }

  static getApiLimitsByTier(tier: PaymentTier): PaymentLimits {
    const limitsMap = {
      [PaymentTier.BASIC]: {
        requestsPerMinute: 10,
        requestsPerDay: 100,
        maxRankings: 5,
        maxFileSize: 1024 * 1024 // 1MB
      },
      [PaymentTier.SILVER]: {
        requestsPerMinute: 50,
        requestsPerDay: 5000,
        maxRankings: 25,
        maxFileSize: 10 * 1024 * 1024 // 10MB
      },
      [PaymentTier.GOLD]: {
        requestsPerMinute: 100,
        requestsPerDay: 10000,
        maxRankings: 50,
        maxFileSize: 50 * 1024 * 1024 // 50MB
      },
      [PaymentTier.PLATINUM]: {
        requestsPerMinute: 500,
        requestsPerDay: 100000,
        maxRankings: 100,
        maxFileSize: 100 * 1024 * 1024 // 100MB
      }
    };
    return limitsMap[tier] || limitsMap[PaymentTier.BASIC];
  }

  static hasFeatureAccess(userTier: PaymentTier, feature: string): boolean {
    const features = this.getApiFeaturesByTier(userTier);
    return features.includes(feature);
  }

  static canAccessRankings(userTier: PaymentTier, requestedCount: number): boolean {
    const limits = this.getApiLimitsByTier(userTier);
    return requestedCount <= limits.maxRankings;
  }

  static getTierPriority(tier: PaymentTier): number {
    const priority = {
      [PaymentTier.BASIC]: 0,
      [PaymentTier.SILVER]: 1,
      [PaymentTier.GOLD]: 2,
      [PaymentTier.PLATINUM]: 3
    };
    return priority[tier] || 0;
  }

  static hasMinimumTier(userTier: PaymentTier, requiredTier: PaymentTier): boolean {
    return this.getTierPriority(userTier) >= this.getTierPriority(requiredTier);
  }

  static isSubscriptionActive(subscription: UserSubscription): boolean {
    if (!subscription.isActive) return false;
    if (!subscription.validUntil) return true; // No expiration date means permanent
    return new Date(subscription.validUntil) > new Date();
  }
}
