import React from 'react';

export interface FeatureFlag {
  key: string;
  enabled: boolean;
  rolloutPercentage: number;
  userGroups?: string[];
  environment?: string[];
  description: string;
}

export interface FeatureFlagConfig {
  [key: string]: FeatureFlag;
}

const defaultFlags: FeatureFlagConfig = {
  'server-side-migration': {
    key: 'server-side-migration',
    enabled: true,
    rolloutPercentage: 100,
    environment: ['development', 'staging', 'production'],
    description: 'Enable server-side architecture migration',
  },
  'enhanced-caching': {
    key: 'enhanced-caching',
    enabled: true,
    rolloutPercentage: 100,
    environment: ['development', 'staging', 'production'],
    description: 'Enable ISR and server-side caching',
  },
  'performance-monitoring': {
    key: 'performance-monitoring',
    enabled: true,
    rolloutPercentage: 100,
    environment: ['development', 'staging', 'production'],
    description: 'Enable comprehensive performance monitoring',
  },
  'gradual-rollout': {
    key: 'gradual-rollout',
    enabled: true,
    rolloutPercentage: 10, // Start with 10% rollout
    userGroups: ['beta-testers', 'premium-users'],
    environment: ['production'],
    description: 'Gradual rollout of new architecture to production users',
  },
  'legacy-fallback': {
    key: 'legacy-fallback',
    enabled: true,
    rolloutPercentage: 100,
    environment: ['production'],
    description: 'Enable fallback to legacy API client when server actions fail',
  },
  'dynamic-imports': {
    key: 'dynamic-imports',
    enabled: true,
    rolloutPercentage: 100,
    environment: ['development', 'staging', 'production'],
    description: 'Enable dynamic imports for analytics components',
  },
};

class FeatureFlagService {
  private flags: FeatureFlagConfig = defaultFlags;
  private userContext: {
    userId?: string;
    userGroup?: string;
    environment: string;
  };

  constructor() {
    this.userContext = {
      environment: process.env.NODE_ENV as string,
    };

    // Load flags from environment or external service
    void this.loadFlags();
  }

  private async loadFlags(): Promise<void> {
    try {
      // In production, load from configuration service
      const endpoint = process.env.FEATURE_FLAGS_ENDPOINT;
      if (endpoint !== undefined && endpoint !== '') {
        const response = await fetch(endpoint, {
          headers: {
            'Authorization': `Bearer ${process.env.FEATURE_FLAGS_API_KEY ?? ''}`,
          },
        });

        if (response.ok) {
          const remoteFlags: unknown = await response.json();
          this.flags = { ...this.flags, ...(remoteFlags as Record<string, boolean>) };
        }
      }

      // Override with environment variables
      this.loadFromEnvironment();
    } catch (_error) {
      // Error loading flags, continue with defaults
    }
  }

  private loadFromEnvironment(): void {
    Object.keys(this.flags).forEach(key => {
      const envKey = `FEATURE_${key.toUpperCase().replace(/-/g, '_')}`;
      const envValue = process.env[envKey];
      
      if (envValue !== undefined) {
        this.flags[key] = {
          ...this.flags[key],
          enabled: envValue === 'true',
          rolloutPercentage: parseInt(envValue) || this.flags[key].rolloutPercentage,
        };
      }
    });
  }

  public setUserContext(context: Partial<typeof this.userContext>): void {
    this.userContext = { ...this.userContext, ...context };
  }

  public isEnabled(flagKey: string, userId?: string): boolean {
    const flag = this.flags[flagKey];

    if (!flag) {
      return false;
    }

    // Check if disabled
    if (!flag.enabled) {
      return false;
    }

    // Check environment
    if (flag.environment !== undefined && !flag.environment.includes(this.userContext.environment)) {
      return false;
    }

    // Check user group
    const userGroup = this.userContext.userGroup;
    if (flag.userGroups !== undefined && userGroup !== undefined && !flag.userGroups.includes(userGroup)) {
      return false;
    }

    // Check rollout percentage
    if (flag.rolloutPercentage < 100) {
      const userHash = this.hashUserId(userId ?? this.userContext.userId ?? 'anonymous');
      const userPercentile = userHash % 100;
      return userPercentile < flag.rolloutPercentage;
    }

    return true;
  }

  private hashUserId(userId: string): number {
    let hash = 0;
    for (let i = 0; i < userId.length; i++) {
      const char = userId.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }

  public getAllFlags(): FeatureFlagConfig {
    return { ...this.flags };
  }

  public updateFlag(key: string, updates: Partial<FeatureFlag>): void {
    const existingFlag = this.flags[key];
    if (existingFlag) {
      this.flags[key] = { ...existingFlag, ...updates };

      // In production, persist changes to configuration service
      if (process.env.NODE_ENV === 'production') {
        void this.persistFlag(key, this.flags[key]);
      }
    }
  }

  private async persistFlag(key: string, flag: FeatureFlag): Promise<void> {
    try {
      const endpoint = process.env.FEATURE_FLAGS_ENDPOINT;
      if (endpoint !== undefined && endpoint !== '') {
        await fetch(`${endpoint}/${key}`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${process.env.FEATURE_FLAGS_API_KEY ?? ''}`,
          },
          body: JSON.stringify(flag),
        });
      }
    } catch (_error) {
      // Error persisting flag, continue
    }
  }

  // Gradual rollout helpers
  public increaseRollout(flagKey: string, percentage: number): void {
    const flag = this.flags[flagKey];
    if (flag) {
      const newPercentage = Math.min(100, flag.rolloutPercentage + percentage);
      this.updateFlag(flagKey, { rolloutPercentage: newPercentage });
      // Increased rollout for feature flag
    }
  }

  public decreaseRollout(flagKey: string, percentage: number): void {
    const flag = this.flags[flagKey];
    if (flag) {
      const newPercentage = Math.max(0, flag.rolloutPercentage - percentage);
      this.updateFlag(flagKey, { rolloutPercentage: newPercentage });
      // Decreased rollout for feature flag
    }
  }

  public emergencyDisable(flagKey: string): void {
    this.updateFlag(flagKey, { enabled: false, rolloutPercentage: 0 });
  }
}

// Global instance
export const featureFlags = new FeatureFlagService();

// React hook for feature flags
export function useFeatureFlag(flagKey: string, userId?: string): boolean {
  // In a real app, this would be a proper React hook with state management
  return featureFlags.isEnabled(flagKey, userId);
}

// HOC for conditional rendering based on feature flags
export function withFeatureFlag<P extends object>(
  Component: React.ComponentType<P>,
  flagKey: string,
  fallback?: React.ComponentType<P>
) {
  return function FeatureFlagWrapper(props: P) {
    const isEnabled = useFeatureFlag(flagKey);
    
    if (isEnabled) {
      return <Component {...props} />;
    }
    
    if (fallback) {
      const FallbackComponent = fallback;
      return <FallbackComponent {...props} />;
    }
    
    return null;
  };
}

// Utility for server-side feature flag checking
export function checkFeatureFlag(flagKey: string, userId?: string): boolean {
  return featureFlags.isEnabled(flagKey, userId);
}