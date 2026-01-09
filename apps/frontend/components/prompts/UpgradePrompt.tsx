'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import {
  ArrowRight,
  CheckCircle,
  Crown,
  Sparkles,
  Star,
  TrendingUp,
  X,
  Zap
} from 'lucide-react';
import { useState } from 'react';

interface UpgradePromptProps {
  // Required tier info
  requiredTier: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'ENTERPRISE';
  currentTier?: 'FREE' | 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'ENTERPRISE';

  // Feature context
  featureName?: string;
  description?: string;

  // Display options
  variant?: 'banner' | 'card' | 'inline' | 'tooltip';
  size?: 'sm' | 'md' | 'lg';
  dismissible?: boolean;
  showFeatures?: boolean;
  showPricing?: boolean;

  // Styling
  className?: string;

  // Actions
  onUpgrade?: (targetTier: string) => void;
  onDismiss?: () => void;
  onLearnMore?: () => void;
}

const TIER_INFO = {
  BRONZE: {
    color: 'from-amber-500 to-orange-500',
    bgColor: 'bg-amber-50 dark:bg-amber-900/20',
    borderColor: 'border-amber-200 dark:border-amber-800',
    textColor: 'text-amber-800 dark:text-amber-200',
    icon: Star,
    features: ['Advanced Analytics', 'Email Support', 'Monthly Reports', 'Basic Filters'],
    price: '$9/month'
  },
  SILVER: {
    color: 'from-slate-400 to-slate-600',
    bgColor: 'bg-slate-50 dark:bg-slate-900/20',
    borderColor: 'border-slate-200 dark:border-slate-800',
    textColor: 'text-slate-800 dark:text-slate-200',
    icon: TrendingUp,
    features: ['Data Export', 'Priority Support', 'Custom Alerts', 'Advanced Filters'],
    price: '$19/month'
  },
  GOLD: {
    color: 'from-yellow-500 to-orange-500',
    bgColor: 'bg-yellow-50 dark:bg-yellow-900/20',
    borderColor: 'border-yellow-200 dark:border-yellow-800',
    textColor: 'text-yellow-800 dark:text-yellow-200',
    icon: Crown,
    features: ['Real-time Data', 'API Access', 'Custom Dashboards', 'Webhooks'],
    price: '$49/month'
  },
  PLATINUM: {
    color: 'from-purple-500 to-pink-500',
    bgColor: 'bg-purple-50 dark:bg-purple-900/20',
    borderColor: 'border-purple-200 dark:border-purple-800',
    textColor: 'text-purple-800 dark:text-purple-200',
    icon: Zap,
    features: ['Unlimited Everything', 'White-label', 'Dedicated Support', 'Custom Integrations'],
    price: '$99/month'
  },
  ENTERPRISE: {
    color: 'from-blue-500 to-indigo-500',
    bgColor: 'bg-blue-50 dark:bg-blue-900/20',
    borderColor: 'border-blue-200 dark:border-blue-800',
    textColor: 'text-blue-800 dark:text-blue-200',
    icon: Sparkles,
    features: ['Custom Solutions', 'On-premise', 'SLA', '24/7 Support'],
    price: 'Custom'
  }
};

export function UpgradePrompt({
  requiredTier,
  currentTier: _currentTier = 'FREE',
  featureName,
  description,
  variant = 'card',
  size: _size = 'md',
  dismissible = true,
  showFeatures = true,
  showPricing = false,
  className = '',
  onUpgrade,
  onDismiss,
  onLearnMore
}: UpgradePromptProps) {
  const [isDismissed, setIsDismissed] = useState(false);

  if (isDismissed) return null;

  const tierInfo = TIER_INFO[requiredTier];
  const IconComponent = tierInfo.icon;

  const handleDismiss = () => {
    setIsDismissed(true);
    onDismiss?.();
  };

  const handleUpgrade = () => {
    onUpgrade?.(requiredTier);
  };

  // Banner variant - full width, prominent
  if (variant === 'banner') {
    return (
      <div className={`relative overflow-hidden ${tierInfo.bgColor} ${tierInfo.borderColor} border rounded-lg p-4 ${className}`}>
        <div className={`absolute inset-0 bg-gradient-to-r ${tierInfo.color} opacity-5`} />

        <div className="relative flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className={`p-2 rounded-full bg-gradient-to-r ${tierInfo.color}`}>
              <IconComponent className="h-5 w-5 text-white" />
            </div>
            <div>
              <h3 className="font-semibold">
                {featureName ? `${featureName} requires ${requiredTier}` : `Upgrade to ${requiredTier}`}
              </h3>
              <p className="text-sm text-muted-foreground">
                {description || `Unlock advanced features and take your analytics to the next level`}
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <Button onClick={handleUpgrade} size="sm">
              Upgrade Now
              <ArrowRight className="h-4 w-4 ml-1" />
            </Button>
            {dismissible && (
              <Button onClick={handleDismiss} variant="ghost" size="sm">
                <X className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
      </div>
    );
  }

  // Card variant - contained, detailed
  if (variant === 'card') {
    return (
      <Card className={`${tierInfo.borderColor} ${className}`}>
        <CardContent className="p-4">
          {dismissible && (
            <button
              onClick={handleDismiss}
              className="absolute top-2 right-2 p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded"
            >
              <X className="h-4 w-4" />
            </button>
          )}

          <div className="flex items-start space-x-4">
            <div className={`p-3 rounded-lg bg-gradient-to-br ${tierInfo.color}`}>
              <IconComponent className="h-6 w-6 text-white" />
            </div>

            <div className="flex-1">
              <div className="flex items-center space-x-2 mb-2">
                <Badge className={`${tierInfo.bgColor} ${tierInfo.textColor} border-0`}>
                  {requiredTier}
                </Badge>
                {showPricing && (
                  <Badge variant="outline" className="text-xs">
                    {tierInfo.price}
                  </Badge>
                )}
              </div>

              <h3 className="font-semibold mb-1">
                {featureName ? `Unlock ${featureName}` : `Upgrade to ${requiredTier}`}
              </h3>

              <p className="text-sm text-muted-foreground mb-3">
                {description || `Get access to premium features and enhanced capabilities.`}
              </p>

              {showFeatures && (
                <div className="mb-4">
                  <p className="text-xs font-medium text-muted-foreground mb-2">
                    {requiredTier} includes:
                  </p>
                  <div className="grid grid-cols-2 gap-1">
                    {tierInfo.features.slice(0, 4).map((feature, index) => (
                      <div key={index} className="flex items-center text-xs text-muted-foreground">
                        <CheckCircle className="h-3 w-3 mr-1 text-green-500" />
                        {feature}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              <div className="flex gap-2">
                <Button onClick={handleUpgrade} size="sm" className="flex-1">
                  Upgrade to {requiredTier}
                </Button>
                {onLearnMore && (
                  <Button onClick={onLearnMore} variant="outline" size="sm">
                    Learn More
                  </Button>
                )}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  // Inline variant - minimal, integrated
  if (variant === 'inline') {
    return (
      <div className={`flex items-center justify-between p-3 ${tierInfo.bgColor} ${tierInfo.borderColor} border rounded-md ${className}`}>
        <div className="flex items-center space-x-2">
          <IconComponent className={`h-4 w-4 ${tierInfo.textColor}`} />
          <span className="text-sm font-medium">
            {featureName ? `${featureName} ` : ''}
            <Badge className={`${tierInfo.bgColor} ${tierInfo.textColor} border-0 text-xs ml-1`}>
              {requiredTier}
            </Badge>
          </span>
        </div>

        <div className="flex items-center space-x-2">
          <Button onClick={handleUpgrade} size="sm" variant="outline">
            Upgrade
          </Button>
          {dismissible && (
            <Button onClick={handleDismiss} variant="ghost" size="sm">
              <X className="h-3 w-3" />
            </Button>
          )}
        </div>
      </div>
    );
  }

  // Tooltip variant - minimal overlay
  if (variant === 'tooltip') {
    return (
      <div className={`absolute inset-0 flex items-center justify-center bg-black/60 backdrop-blur-sm rounded ${className}`}>
        <div className={`p-4 ${tierInfo.bgColor} rounded-lg border ${tierInfo.borderColor} shadow-lg max-w-xs text-center`}>
          <IconComponent className={`h-8 w-8 mx-auto mb-2 ${tierInfo.textColor}`} />
          <h3 className="font-semibold text-sm mb-1">
            {requiredTier} Required
          </h3>
          <p className="text-xs text-muted-foreground mb-3">
            {featureName && `${featureName} requires `}a {requiredTier} subscription
          </p>
          <div className="flex gap-2">
            <Button onClick={handleUpgrade} size="sm" className="flex-1 text-xs">
              Upgrade
            </Button>
            {dismissible && (
              <Button onClick={handleDismiss} variant="outline" size="sm">
                <X className="h-3 w-3" />
              </Button>
            )}
          </div>
        </div>
      </div>
    );
  }

  return null;
}

// Convenience components for common use cases
export const BronzeUpgradePrompt = (props: Omit<UpgradePromptProps, 'requiredTier'>) => (
  <UpgradePrompt requiredTier="BRONZE" {...props} />
);

export const SilverUpgradePrompt = (props: Omit<UpgradePromptProps, 'requiredTier'>) => (
  <UpgradePrompt requiredTier="SILVER" {...props} />
);

export const GoldUpgradePrompt = (props: Omit<UpgradePromptProps, 'requiredTier'>) => (
  <UpgradePrompt requiredTier="GOLD" {...props} />
);

export const PlatinumUpgradePrompt = (props: Omit<UpgradePromptProps, 'requiredTier'>) => (
  <UpgradePrompt requiredTier="PLATINUM" {...props} />
);

export const EnterpriseUpgradePrompt = (props: Omit<UpgradePromptProps, 'requiredTier'>) => (
  <UpgradePrompt requiredTier="ENTERPRISE" {...props} />
);