'use client';

import { useState } from 'react';
import { 
  Shield, 
  Lock, 
  Crown, 
  User, 
  CreditCard, 
  MessageSquare, 
  ExternalLink,
  CheckCircle,
  XCircle,
  TrendingUp,
  Star
} from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Card } from '@/components/ui/card';

interface PermissionRequestModalProps {
  isOpen: boolean;
  onClose: () => void;
  
  // Permission details
  requiredPermission?: string;
  requiredTier?: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'ENTERPRISE';
  featureName: string;
  restrictionReason: string;
  
  // User context
  userEmail?: string;
  currentTier?: 'FREE' | 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'ENTERPRISE';
  isLoggedIn?: boolean;
  
  // Actions
  onUpgrade?: (targetTier: string) => void;
  onLogin?: () => void;
  onContactSupport?: () => void;
  onTryFeature?: () => void; // For free trials
}

type ModalStep = 'overview' | 'upgrade' | 'login';

const TIER_COLORS = {
  FREE: 'text-gray-600 bg-gray-100',
  BRONZE: 'text-amber-700 bg-amber-100',
  SILVER: 'text-slate-600 bg-slate-100',
  GOLD: 'text-yellow-700 bg-yellow-100',
  PLATINUM: 'text-purple-700 bg-purple-100',
  ENTERPRISE: 'text-blue-700 bg-blue-100'
};

const TIER_FEATURES = {
  BRONZE: ['Basic analytics', 'Email support', 'Monthly reports'],
  SILVER: ['Advanced analytics', 'Data export', 'Priority support', 'Custom alerts'],
  GOLD: ['Real-time data', 'Advanced filters', 'API access', 'Custom dashboards'],
  PLATINUM: ['Unlimited everything', 'White-label options', 'Dedicated support', 'Custom integrations'],
  ENTERPRISE: ['Custom solutions', 'On-premise deployment', 'SLA guarantee', '24/7 phone support']
};

export function PermissionRequestModal({
  isOpen,
  onClose,
  requiredPermission,
  requiredTier,
  featureName,
  restrictionReason,
  userEmail,
  currentTier = 'FREE',
  isLoggedIn = false,
  onUpgrade,
  onLogin,
  onContactSupport,
  onTryFeature
}: PermissionRequestModalProps) {
  const [currentStep, setCurrentStep] = useState<ModalStep>('overview');

  // Reset form when modal closes
  const handleModalOpenChange = (open: boolean) => {
    if (!open) {
      onClose();
      setTimeout(() => {
        setCurrentStep('overview');
      }, 300);
    }
  };

  const getIcon = () => {
    if (requiredTier) return <Crown className="h-8 w-8 text-yellow-500" />;
    if (!isLoggedIn) return <User className="h-6 w-6 text-blue-500" />;
    return <Lock className="h-8 w-8 text-gray-500" />;
  };

  const getTitle = () => {
    switch (currentStep) {
      case 'upgrade': return 'Choose Your Plan';
      case 'login': return 'Sign In Required';
      default: return `Unlock: ${featureName}`;
    }
  };

  const renderOverview = () => (
    <div className="space-y-6">
      {/* Feature info */}
      <div className="flex items-start space-x-4 p-4 bg-gradient-to-br from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20 rounded-lg border">
        {getIcon()}
        <div className="flex-1">
          <h3 className="font-semibold text-lg">{featureName}</h3>
          <p className="text-muted-foreground text-sm mt-1">{restrictionReason}</p>
          
          {/* Tier badges */}
          {requiredTier && (
            <div className="flex items-center gap-2 mt-3">
              <Badge className={`text-xs ${TIER_COLORS[requiredTier]}`}>
                Requires: {requiredTier}
              </Badge>
              <span className="text-xs text-muted-foreground">→</span>
              <Badge variant="outline" className="text-xs">
                You have: {currentTier}
              </Badge>
            </div>
          )}
        </div>
      </div>

      {/* Not logged in */}
      {!isLoggedIn && (
        <Card className="p-4 border-blue-200 bg-blue-50/50">
          <div className="flex items-center space-x-3">
            <User className="h-5 w-5 text-blue-600" />
            <div className="flex-1">
              <h4 className="font-medium">Sign In to Continue</h4>
              <p className="text-sm text-muted-foreground">
                Create a free account or sign in to access this feature and track your analytics.
              </p>
            </div>
          </div>
          <Button onClick={onLogin} className="w-full mt-3">
            Sign In / Create Account
          </Button>
        </Card>
      )}

      {/* Upgrade required */}
      {isLoggedIn && requiredTier && (
        <Card className="p-4 border-yellow-200 bg-yellow-50/50">
          <div className="flex items-center space-x-3">
            <Crown className="h-5 w-5 text-yellow-600" />
            <div className="flex-1">
              <h4 className="font-medium">Upgrade Required</h4>
              <p className="text-sm text-muted-foreground">
                This feature is available to {requiredTier}+ subscribers. Upgrade now to unlock advanced capabilities.
              </p>
            </div>
          </div>
          
          {/* Feature preview */}
          {TIER_FEATURES[requiredTier] && (
            <div className="mt-3 p-3 bg-white/60 rounded-md">
              <p className="text-xs font-medium text-muted-foreground mb-2">{requiredTier} includes:</p>
              <ul className="text-xs space-y-1">
                {TIER_FEATURES[requiredTier].slice(0, 3).map((feature, index) => (
                  <li key={index} className="flex items-center text-muted-foreground">
                    <Star className="h-3 w-3 mr-2 text-yellow-500" />
                    {feature}
                  </li>
                ))}
              </ul>
            </div>
          )}
          
          <div className="flex gap-2 mt-3">
            <Button onClick={() => onUpgrade?.(requiredTier)} className="flex-1">
              <CreditCard className="h-4 w-4 mr-2" />
              Upgrade to {requiredTier}
            </Button>
            {onTryFeature && (
              <Button onClick={onTryFeature} variant="outline">
                Free Trial
              </Button>
            )}
          </div>
        </Card>
      )}

      {/* General access info */}
      <div className="space-y-3">
        <h4 className="font-medium text-sm">What you can do:</h4>
        
        <div className="grid gap-3">
          {onUpgrade && requiredTier && (
            <div className="flex items-center p-3 bg-muted/50 rounded-lg">
              <TrendingUp className="h-4 w-4 mr-3 text-green-600" />
              <div className="flex-1">
                <p className="text-sm font-medium">Upgrade your plan</p>
                <p className="text-xs text-muted-foreground">Get instant access to this feature and more</p>
              </div>
              <Button size="sm" onClick={() => setCurrentStep('upgrade')}>
                View Plans
              </Button>
            </div>
          )}
          
          {onTryFeature && (
            <div className="flex items-center p-3 bg-muted/50 rounded-lg">
              <Star className="h-4 w-4 mr-3 text-blue-600" />
              <div className="flex-1">
                <p className="text-sm font-medium">Try it free</p>
                <p className="text-xs text-muted-foreground">Limited time trial access</p>
              </div>
              <Button size="sm" variant="outline" onClick={onTryFeature}>
                Start Trial
              </Button>
            </div>
          )}
          
          {onContactSupport && (
            <div className="flex items-center p-3 bg-muted/50 rounded-lg">
              <MessageSquare className="h-4 w-4 mr-3 text-purple-600" />
              <div className="flex-1">
                <p className="text-sm font-medium">Contact support</p>
                <p className="text-xs text-muted-foreground">Get help from our team</p>
              </div>
              <Button size="sm" variant="outline" onClick={onContactSupport}>
                Contact Us
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Current user info */}
      {userEmail && (
        <div className="text-xs text-muted-foreground border-t pt-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <User className="h-3 w-3 mr-1" />
              {userEmail}
            </div>
            <Badge variant="outline" className="text-xs">
              {currentTier}
            </Badge>
          </div>
        </div>
      )}
    </div>
  );

  const renderUpgradePage = () => (
    <div className="space-y-4">
      <div className="text-center mb-6">
        <Crown className="h-12 w-12 text-yellow-500 mx-auto mb-2" />
        <h3 className="font-semibold text-lg">Choose Your Plan</h3>
        <p className="text-muted-foreground text-sm">
          Upgrade to unlock {featureName} and other premium features
        </p>
      </div>

      {/* Plan comparison */}
      <div className="grid gap-3">
        {(['BRONZE', 'SILVER', 'GOLD', 'PLATINUM'] as const).map((tier) => (
          <Card 
            key={tier} 
            className={`p-4 cursor-pointer transition-all hover:shadow-md ${
              tier === requiredTier ? 'ring-2 ring-yellow-400 bg-yellow-50/50' : ''
            }`}
            onClick={() => onUpgrade?.(tier)}
          >
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <div className="flex items-center space-x-2">
                  <Badge className={`text-xs ${TIER_COLORS[tier]}`}>
                    {tier}
                  </Badge>
                  {tier === requiredTier && (
                    <Badge variant="secondary" className="text-xs">
                      Recommended
                    </Badge>
                  )}
                </div>
                
                {TIER_FEATURES[tier] && (
                  <ul className="text-xs text-muted-foreground mt-2 space-y-1">
                    {TIER_FEATURES[tier].slice(0, 2).map((feature, index) => (
                      <li key={index} className="flex items-center">
                        <Star className="h-3 w-3 mr-1 text-yellow-500" />
                        {feature}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
              
              <Button size="sm" variant={tier === requiredTier ? 'default' : 'outline'}>
                {tier === currentTier ? 'Current' : 'Select'}
              </Button>
            </div>
          </Card>
        ))}
      </div>

      <div className="flex gap-2 pt-4">
        <Button variant="outline" onClick={() => setCurrentStep('overview')} className="flex-1">
          Back
        </Button>
        <Button onClick={() => onContactSupport?.()} className="flex-1">
          Need Help?
        </Button>
      </div>
    </div>
  );

  return (
    <Dialog open={isOpen} onOpenChange={handleModalOpenChange}>
      <DialogContent className="sm:max-w-[500px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-center">{getTitle()}</DialogTitle>
          <DialogDescription className="text-center">
            {currentStep === 'overview' && 'Unlock this feature to enhance your analytics experience'}
            {currentStep === 'upgrade' && 'Compare plans and choose what works best for you'}
            {currentStep === 'login' && 'Sign in to access your account and premium features'}
          </DialogDescription>
        </DialogHeader>

        <div className="mt-4">
          {currentStep === 'overview' && renderOverview()}
          {currentStep === 'upgrade' && renderUpgradePage()}
        </div>
      </DialogContent>
    </Dialog>
  );
}