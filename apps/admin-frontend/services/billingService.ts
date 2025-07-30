import type { 
  getModuleUsageAnalytics,
  createApiKey,
  listApiKeys 
} from '@epsx/server-actions';

export interface PricingTier {
  name: string;
  pricePerRequest: number;
  includedRequests: number;
  maxRequests: number;
  features: string[];
}

export interface UsageBill {
  id: string;
  userId: string;
  apiKeyId?: string;
  period: {
    start: string;
    end: string;
  };
  usage: {
    totalRequests: number;
    moduleBreakdown: Array<{
      moduleId: string;
      moduleName: string;
      requests: number;
      cost: number;
    }>;
  };
  pricing: {
    basePrice: number;
    overagePrice: number;
    totalCost: number;
    discounts: Array<{
      type: string;
      amount: number;
      description: string;
    }>;
  };
  status: 'draft' | 'pending' | 'paid' | 'overdue';
  invoiceUrl?: string;
  paymentDue: string;
  createdAt: string;
}

export interface BillingAlert {
  id: string;
  type: 'quota_warning' | 'quota_exceeded' | 'payment_due' | 'usage_spike';
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  message: string;
  actionRequired: boolean;
  metadata: Record<string, any>;
  createdAt: string;
  dismissedAt?: string;
}

// Pricing configuration for different modules and access levels
export const PRICING_CONFIG = {
  modules: {
    'stock-ranking': {
      bronze: { pricePerRequest: 0.001, includedRequests: 1000 },
      silver: { pricePerRequest: 0.0008, includedRequests: 5000 },
      gold: { pricePerRequest: 0.0006, includedRequests: 20000 },
      platinum: { pricePerRequest: 0.0004, includedRequests: 100000 },
      enterprise: { pricePerRequest: 0.0002, includedRequests: 1000000 }
    },
    'market-data': {
      bronze: { pricePerRequest: 0.0005, includedRequests: 2000 },
      silver: { pricePerRequest: 0.0004, includedRequests: 10000 },
      gold: { pricePerRequest: 0.0003, includedRequests: 40000 },
      platinum: { pricePerRequest: 0.0002, includedRequests: 200000 },
      enterprise: { pricePerRequest: 0.0001, includedRequests: 2000000 }
    },
    'portfolio-analysis': {
      bronze: { pricePerRequest: 0.002, includedRequests: 500 },
      silver: { pricePerRequest: 0.0015, includedRequests: 2500 },
      gold: { pricePerRequest: 0.001, includedRequests: 10000 },
      platinum: { pricePerRequest: 0.0008, includedRequests: 50000 },
      enterprise: { pricePerRequest: 0.0005, includedRequests: 500000 }
    },
    'trading-signals': {
      bronze: { pricePerRequest: 0.003, includedRequests: 200 },
      silver: { pricePerRequest: 0.0025, includedRequests: 1000 },
      gold: { pricePerRequest: 0.002, includedRequests: 5000 },
      platinum: { pricePerRequest: 0.0015, includedRequests: 25000 },
      enterprise: { pricePerRequest: 0.001, includedRequests: 250000 }
    }
  },
  discounts: {
    volume: [
      { threshold: 100000, discount: 0.05, description: '5% volume discount' },
      { threshold: 500000, discount: 0.10, description: '10% volume discount' },
      { threshold: 1000000, discount: 0.15, description: '15% volume discount' }
    ],
    loyalty: [
      { months: 6, discount: 0.02, description: '2% loyalty discount' },
      { months: 12, discount: 0.05, description: '5% loyalty discount' },
      { months: 24, discount: 0.10, description: '10% loyalty discount' }
    ]
  }
};

export class BillingService {
  private static instance: BillingService;
  
  public static getInstance(): BillingService {
    if (!BillingService.instance) {
      BillingService.instance = new BillingService();
    }
    return BillingService.instance;
  }

  /**
   * Calculate usage cost for a specific period and module
   */
  async calculateUsageCost(
    moduleId: string,
    accessLevel: string,
    requestCount: number,
    loyaltyMonths: number = 0
  ): Promise<{
    baseCost: number;
    overageCost: number;
    discounts: Array<{ type: string; amount: number; description: string }>;
    totalCost: number;
  }> {
    const moduleConfig = PRICING_CONFIG.modules[moduleId as keyof typeof PRICING_CONFIG.modules];
    if (!moduleConfig) {
      throw new Error(`Pricing configuration not found for module: ${moduleId}`);
    }

    const tierConfig = moduleConfig[accessLevel as keyof typeof moduleConfig];
    if (!tierConfig) {
      throw new Error(`Pricing tier not found for access level: ${accessLevel}`);
    }

    const { pricePerRequest, includedRequests } = tierConfig;
    
    // Calculate base cost (included requests)
    const billableRequests = Math.max(0, requestCount - includedRequests);
    const baseCost = Math.min(requestCount, includedRequests) * 0; // Included requests are free
    const overageCost = billableRequests * pricePerRequest;
    
    let totalCost = baseCost + overageCost;
    const discounts: Array<{ type: string; amount: number; description: string }> = [];

    // Apply volume discounts
    for (const volumeDiscount of PRICING_CONFIG.discounts.volume) {
      if (requestCount >= volumeDiscount.threshold) {
        const discountAmount = totalCost * volumeDiscount.discount;
        discounts.push({
          type: 'volume',
          amount: discountAmount,
          description: volumeDiscount.description
        });
        totalCost -= discountAmount;
        break; // Apply only the highest applicable discount
      }
    }

    // Apply loyalty discounts
    for (const loyaltyDiscount of PRICING_CONFIG.discounts.loyalty) {
      if (loyaltyMonths >= loyaltyDiscount.months) {
        const discountAmount = totalCost * loyaltyDiscount.discount;
        discounts.push({
          type: 'loyalty',
          amount: discountAmount,
          description: loyaltyDiscount.description
        });
        totalCost -= discountAmount;
        break; // Apply only the highest applicable discount
      }
    }

    return {
      baseCost,
      overageCost,
      discounts,
      totalCost: Math.max(0, totalCost) // Ensure non-negative cost
    };
  }

  /**
   * Generate a comprehensive usage bill for a user
   */
  async generateUsageBill(
    userId: string,
    period: { start: string; end: string },
    apiKeyId?: string
  ): Promise<UsageBill> {
    try {
      // Fetch usage data for the period
      // const usageData = await getModuleUsageAnalytics({
      //   startDate: period.start,
      //   endDate: period.end,
      //   userId,
      //   apiKeyId
      // });

      // Mock usage data for demonstration
      const mockUsageData = {
        totalRequests: 150000,
        moduleBreakdown: [
          { moduleId: 'stock-ranking', moduleName: 'Stock Ranking', requests: 80000, accessLevel: 'silver' },
          { moduleId: 'market-data', moduleName: 'Market Data', requests: 45000, accessLevel: 'gold' },
          { moduleId: 'portfolio-analysis', moduleName: 'Portfolio Analysis', requests: 15000, accessLevel: 'bronze' },
          { moduleId: 'trading-signals', moduleName: 'Trading Signals', requests: 10000, accessLevel: 'silver' }
        ]
      };

      let totalCost = 0;
      const moduleBreakdown: UsageBill['usage']['moduleBreakdown'] = [];
      const allDiscounts: Array<{ type: string; amount: number; description: string }> = [];

      // Calculate cost for each module
      for (const moduleUsage of mockUsageData.moduleBreakdown) {
        const costCalculation = await this.calculateUsageCost(
          moduleUsage.moduleId,
          moduleUsage.accessLevel,
          moduleUsage.requests,
          6 // Assume 6 months loyalty for demo
        );

        moduleBreakdown.push({
          moduleId: moduleUsage.moduleId,
          moduleName: moduleUsage.moduleName,
          requests: moduleUsage.requests,
          cost: costCalculation.totalCost
        });

        totalCost += costCalculation.totalCost;
        allDiscounts.push(...costCalculation.discounts);
      }

      const bill: UsageBill = {
        id: `bill_${Date.now()}`,
        userId,
        apiKeyId,
        period,
        usage: {
          totalRequests: mockUsageData.totalRequests,
          moduleBreakdown
        },
        pricing: {
          basePrice: totalCost + allDiscounts.reduce((sum, d) => sum + d.amount, 0),
          overagePrice: 0, // Calculated separately per module
          totalCost,
          discounts: allDiscounts
        },
        status: 'draft',
        paymentDue: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(), // 30 days from now
        createdAt: new Date().toISOString()
      };

      return bill;
    } catch (error) {
      console.error('Error generating usage bill:', error);
      throw new Error('Failed to generate usage bill');
    }
  }

  /**
   * Generate billing alerts based on usage patterns
   */
  async generateBillingAlerts(userId: string): Promise<BillingAlert[]> {
    const alerts: BillingAlert[] = [];

    try {
      // Mock current usage data
      const currentUsage = {
        'stock-ranking': { used: 4800, quota: 5000, accessLevel: 'silver' },
        'market-data': { used: 8500, quota: 40000, accessLevel: 'gold' },
        'portfolio-analysis': { used: 480, quota: 500, accessLevel: 'bronze' },
        'trading-signals': { used: 900, quota: 1000, accessLevel: 'silver' }
      };

      // Check for quota warnings
      Object.entries(currentUsage).forEach(([moduleId, usage]) => {
        const usagePercentage = (usage.used / usage.quota) * 100;
        
        if (usagePercentage >= 95) {
          alerts.push({
            id: `alert_${moduleId}_critical_${Date.now()}`,
            type: 'quota_exceeded',
            severity: 'critical',
            title: 'Quota Exceeded',
            message: `Your ${moduleId.replace('-', ' ')} quota has been exceeded. Additional charges will apply.`,
            actionRequired: true,
            metadata: { moduleId, usagePercentage, quota: usage.quota, used: usage.used },
            createdAt: new Date().toISOString()
          });
        } else if (usagePercentage >= 80) {
          alerts.push({
            id: `alert_${moduleId}_warning_${Date.now()}`,
            type: 'quota_warning',
            severity: usagePercentage >= 90 ? 'high' : 'medium',
            title: 'Quota Warning',
            message: `You have used ${usagePercentage.toFixed(1)}% of your ${moduleId.replace('-', ' ')} quota.`,
            actionRequired: false,
            metadata: { moduleId, usagePercentage, quota: usage.quota, used: usage.used },
            createdAt: new Date().toISOString()
          });
        }
      });

      // Check for payment due alerts
      const upcomingPayment = {
        amount: 1250.75,
        dueDate: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000).toISOString() // 5 days from now
      };

      alerts.push({
        id: `alert_payment_due_${Date.now()}`,
        type: 'payment_due',
        severity: 'medium',
        title: 'Payment Due Soon',
        message: `Your payment of $${upcomingPayment.amount} is due in 5 days.`,
        actionRequired: true,
        metadata: { amount: upcomingPayment.amount, dueDate: upcomingPayment.dueDate },
        createdAt: new Date().toISOString()
      });

      return alerts.sort((a, b) => {
        const severityOrder = { critical: 4, high: 3, medium: 2, low: 1 };
        return severityOrder[b.severity] - severityOrder[a.severity];
      });

    } catch (error) {
      console.error('Error generating billing alerts:', error);
      return [];
    }
  }

  /**
   * Process payment for a bill
   */
  async processPayment(billId: string, paymentMethod: string): Promise<{
    success: boolean;
    transactionId?: string;
    error?: string;
  }> {
    try {
      // Mock payment processing
      await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate API call
      
      // In a real implementation, this would integrate with Stripe, PayPal, etc.
      const mockTransaction = {
        id: `txn_${Date.now()}`,
        status: 'completed',
        amount: 1250.75
      };

      return {
        success: true,
        transactionId: mockTransaction.id
      };
    } catch (error) {
      console.error('Payment processing error:', error);
      return {
        success: false,
        error: 'Payment processing failed. Please try again.'
      };
    }
  }

  /**
   * Generate invoice PDF
   */
  async generateInvoicePDF(bill: UsageBill): Promise<{
    success: boolean;
    downloadUrl?: string;
    error?: string;
  }> {
    try {
      // Mock PDF generation
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      return {
        success: true,
        downloadUrl: `/api/invoices/${bill.id}/download`
      };
    } catch (error) {
      console.error('Invoice generation error:', error);
      return {
        success: false,
        error: 'Failed to generate invoice PDF'
      };
    }
  }

  /**
   * Get pricing estimate for usage projection
   */
  async getPricingEstimate(
    moduleId: string,
    accessLevel: string,
    projectedRequests: number
  ): Promise<{
    estimatedCost: number;
    breakdown: {
      includedRequests: number;
      overageRequests: number;
      includedCost: number;
      overageCost: number;
    };
    recommendations?: string[];
  }> {
    const costCalculation = await this.calculateUsageCost(moduleId, accessLevel, projectedRequests);
    const moduleConfig = PRICING_CONFIG.modules[moduleId as keyof typeof PRICING_CONFIG.modules];
    const tierConfig = moduleConfig[accessLevel as keyof typeof moduleConfig];
    
    const includedRequests = tierConfig.includedRequests;
    const overageRequests = Math.max(0, projectedRequests - includedRequests);
    
    const recommendations: string[] = [];
    
    // Suggest tier upgrades if beneficial
    const accessLevels = ['bronze', 'silver', 'gold', 'platinum', 'enterprise'];
    const currentIndex = accessLevels.indexOf(accessLevel);
    
    if (currentIndex < accessLevels.length - 1) {
      const nextTier = accessLevels[currentIndex + 1];
      const nextTierConfig = moduleConfig[nextTier as keyof typeof moduleConfig];
      const nextTierCost = await this.calculateUsageCost(moduleId, nextTier, projectedRequests);
      
      if (nextTierCost.totalCost < costCalculation.totalCost * 0.9) {
        recommendations.push(
          `Consider upgrading to ${nextTier} tier to save ${((costCalculation.totalCost - nextTierCost.totalCost) / costCalculation.totalCost * 100).toFixed(1)}%`
        );
      }
    }

    return {
      estimatedCost: costCalculation.totalCost,
      breakdown: {
        includedRequests,
        overageRequests,
        includedCost: 0, // Included requests are free
        overageCost: overageRequests * tierConfig.pricePerRequest
      },
      recommendations
    };
  }
}