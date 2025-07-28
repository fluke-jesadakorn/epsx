import { featureFlags } from './feature-flags';

export interface DeploymentHealth {
  healthy: boolean;
  errorRate: number;
  responseTime: number;
  cacheHitRate: number;
  timestamp: number;
  version: string;
}

export interface RollbackTrigger {
  type: 'error_rate' | 'response_time' | 'manual' | 'health_check';
  threshold: number;
  timeWindow: number; // minutes
  description: string;
}

export interface RollbackPlan {
  triggers: RollbackTrigger[];
  fallbackVersion: string;
  rollbackSteps: string[];
  notifications: string[];
}

class RollbackService {
  private healthChecks: DeploymentHealth[] = [];
  private rollbackPlan: RollbackPlan = {
    triggers: [
      {
        type: 'error_rate',
        threshold: 5, // 5% error rate
        timeWindow: 5,
        description: 'Rollback if error rate exceeds 5% for 5 minutes',
      },
      {
        type: 'response_time',
        threshold: 3000, // 3 seconds
        timeWindow: 10,
        description: 'Rollback if average response time exceeds 3s for 10 minutes',
      },
    ],
    fallbackVersion: 'legacy-api-client',
    rollbackSteps: [
      'Disable new server actions feature flag',
      'Enable legacy API client fallback',
      'Clear server-side cache',
      'Restart application servers',
      'Verify health checks pass',
    ],
    notifications: [
      'Send alert to dev team',
      'Update status page',
      'Log rollback event',
    ],
  };

  public recordHealthCheck(health: DeploymentHealth): void {
    this.healthChecks.push(health);
    
    // Keep only last 100 health checks
    if (this.healthChecks.length > 100) {
      this.healthChecks = this.healthChecks.slice(-100);
    }

    // Check if rollback is needed
    this.evaluateRollbackTriggers(health);
  }

  private evaluateRollbackTriggers(currentHealth: DeploymentHealth): void {
    const now = Date.now();
    
    for (const trigger of this.rollbackPlan.triggers) {
      const timeWindowMs = trigger.timeWindow * 60 * 1000;
      const recentChecks = this.healthChecks.filter(
        check => (now - check.timestamp) <= timeWindowMs
      );

      if (recentChecks.length === 0) continue;

      const shouldRollback = this.shouldTriggerRollback(trigger, recentChecks);
      
      if (shouldRollback) {
        console.warn(`Rollback trigger activated: ${trigger.description}`);
        this.initiateRollback(trigger);
        break;
      }
    }
  }

  private shouldTriggerRollback(trigger: RollbackTrigger, healthChecks: DeploymentHealth[]): boolean {
    switch (trigger.type) {
      case 'error_rate':
        const avgErrorRate = healthChecks.reduce((sum, check) => sum + check.errorRate, 0) / healthChecks.length;
        return avgErrorRate > trigger.threshold;
        
      case 'response_time':
        const avgResponseTime = healthChecks.reduce((sum, check) => sum + check.responseTime, 0) / healthChecks.length;
        return avgResponseTime > trigger.threshold;
        
      case 'health_check':
        return healthChecks.some(check => !check.healthy);
        
      default:
        return false;
    }
  }

  public async initiateRollback(trigger: RollbackTrigger): Promise<void> {
    console.warn('🚨 INITIATING ROLLBACK 🚨', {
      trigger: trigger.type,
      description: trigger.description,
      timestamp: new Date().toISOString(),
    });

    try {
      // Step 1: Emergency disable problematic features
      await this.emergencyFeatureDisable();
      
      // Step 2: Enable fallback mechanisms
      await this.enableFallbacks();
      
      // Step 3: Clear caches to ensure clean state
      await this.clearCaches();
      
      // Step 4: Send notifications
      await this.sendRollbackNotifications(trigger);
      
      // Step 5: Log rollback event
      await this.logRollbackEvent(trigger);
      
      console.log('✅ Rollback completed successfully');
      
    } catch (error) {
      console.error('❌ Rollback failed:', error);
      await this.sendFailedRollbackAlert(error, trigger);
    }
  }

  private async emergencyFeatureDisable(): Promise<void> {
    // Disable risky features immediately
    featureFlags.emergencyDisable('server-side-migration');
    featureFlags.updateFlag('legacy-fallback', { enabled: true, rolloutPercentage: 100 });
    
    console.log('Disabled server-side migration, enabled legacy fallback');
  }

  private async enableFallbacks(): Promise<void> {
    // Enable all fallback mechanisms
    featureFlags.updateFlag('legacy-fallback', { 
      enabled: true, 
      rolloutPercentage: 100 
    });
    
    // Reduce rollout of new features to 0%
    featureFlags.updateFlag('gradual-rollout', { 
      enabled: false, 
      rolloutPercentage: 0 
    });
    
    console.log('Enabled fallback mechanisms');
  }

  private async clearCaches(): Promise<void> {
    try {
      // Clear server-side caches
      await fetch('/api/cache/clear', { 
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });
      
      console.log('Cleared server-side caches');
    } catch (error) {
      console.error('Failed to clear caches:', error);
    }
  }

  private async sendRollbackNotifications(trigger: RollbackTrigger): Promise<void> {
    const message = {
      type: 'rollback_initiated',
      trigger: trigger.type,
      description: trigger.description,
      timestamp: new Date().toISOString(),
      version: process.env.npm_package_version || 'unknown',
    };

    // Send to monitoring endpoint
    try {
      await fetch('/api/monitoring/alerts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(message),
      });
    } catch (error) {
      console.error('Failed to send rollback notification:', error);
    }

    // In production, also send to Slack, PagerDuty, etc.
    console.warn('Rollback notification sent:', message);
  }

  private async logRollbackEvent(trigger: RollbackTrigger): Promise<void> {
    const event = {
      event: 'rollback_executed',
      trigger: trigger.type,
      description: trigger.description,
      timestamp: new Date().toISOString(),
      version: process.env.npm_package_version || 'unknown',
      healthChecks: this.healthChecks.slice(-10), // Last 10 health checks
    };

    try {
      await fetch('/api/monitoring/events', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(event),
      });
    } catch (error) {
      console.error('Failed to log rollback event:', error);
    }
  }

  private async sendFailedRollbackAlert(error: any, trigger: RollbackTrigger): Promise<void> {
    const alert = {
      type: 'rollback_failed',
      error: error.message,
      trigger: trigger.type,
      timestamp: new Date().toISOString(),
      severity: 'critical',
    };

    try {
      await fetch('/api/monitoring/alerts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(alert),
      });
    } catch (e) {
      console.error('Failed to send failed rollback alert:', e);
    }
  }

  // Manual rollback for emergency situations
  public async manualRollback(reason: string): Promise<void> {
    const trigger: RollbackTrigger = {
      type: 'manual',
      threshold: 0,
      timeWindow: 0,
      description: `Manual rollback: ${reason}`,
    };

    await this.initiateRollback(trigger);
  }

  // Gradual re-enable after rollback
  public async gradualReEnable(featureKey: string, stepPercentage: number = 10): Promise<void> {
    const flag = featureFlags.getAllFlags()[featureKey];
    
    if (!flag) {
      console.error(`Feature flag '${featureKey}' not found`);
      return;
    }

    console.log(`Gradually re-enabling '${featureKey}' by ${stepPercentage}%`);
    featureFlags.increaseRollout(featureKey, stepPercentage);
    
    // Monitor for 5 minutes before allowing next increase
    setTimeout(() => {
      const currentHealth = this.getCurrentHealth();
      if (currentHealth.healthy) {
        console.log(`Health check passed for '${featureKey}', safe to continue gradual rollout`);
      } else {
        console.warn(`Health check failed for '${featureKey}', reverting rollout`);
        featureFlags.decreaseRollout(featureKey, stepPercentage);
      }
    }, 5 * 60 * 1000); // 5 minutes
  }

  private getCurrentHealth(): DeploymentHealth {
    return this.healthChecks[this.healthChecks.length - 1] || {
      healthy: false,
      errorRate: 100,
      responseTime: 0,
      cacheHitRate: 0,
      timestamp: Date.now(),
      version: 'unknown',
    };
  }

  public getRollbackStatus(): {
    lastRollback: string | null;
    activeTriggers: RollbackTrigger[];
    currentHealth: DeploymentHealth;
  } {
    return {
      lastRollback: null, // Would track last rollback from persistent storage
      activeTriggers: this.rollbackPlan.triggers,
      currentHealth: this.getCurrentHealth(),
    };
  }
}

// Global rollback service
export const rollbackService = new RollbackService();

// Utility functions
export function isRollbackActive(): boolean {
  return !featureFlags.isEnabled('server-side-migration');
}

export function enableEmergencyMode(): void {
  rollbackService.manualRollback('Emergency mode activated');
}

export function gradualReEnableFeature(featureKey: string): void {
  rollbackService.gradualReEnable(featureKey);
}