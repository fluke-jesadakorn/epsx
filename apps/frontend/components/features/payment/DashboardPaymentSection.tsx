'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  CreditCard, 
  TrendingUp, 
  Zap, 
  Star,
  ArrowRight,
  Crown
} from 'lucide-react';
import PaymentWidget from '@/components/features/payment/PaymentWidget';
import QuickPayButton from '@/components/features/payment/QuickPayButton';

interface DashboardPaymentSectionProps {
  currentPlan?: string;
  expiryDate?: string;
  className?: string;
}

export default function DashboardPaymentSection({ 
  currentPlan = 'Basic',
  expiryDate,
  className = ''
}: DashboardPaymentSectionProps) {
  const [showUpgrade, setShowUpgrade] = useState(false);
  const [timeLeft, setTimeLeft] = useState<string>('');

  // Calculate time until expiry
  useEffect(() => {
    if (expiryDate) {
      const updateTimer = () => {
        const now = new Date();
        const expiry = new Date(expiryDate);
        const diff = expiry.getTime() - now.getTime();
        
        if (diff > 0) {
          const days = Math.floor(diff / (1000 * 60 * 60 * 24));
          const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
          setTimeLeft(`${days}d ${hours}h`);
        } else {
          setTimeLeft('Expired');
        }
      };

      updateTimer();
      const interval = setInterval(updateTimer, 1000 * 60); // Update every minute
      return () => clearInterval(interval);
    }
  }, [expiryDate]);

  const isBasicPlan = currentPlan === 'Basic';
  const isExpiringSoon = expiryDate && new Date(expiryDate).getTime() - Date.now() < 7 * 24 * 60 * 60 * 1000;

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Current Plan Status */}
      <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-gray-900 dark:text-white">
            <CreditCard className="h-5 w-5" />
            Current Plan
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div>
              <div className="flex items-center gap-2 mb-1">
                <span className="text-xl sm:text-2xl font-bold text-gray-900 dark:text-white">{currentPlan}</span>
                {!isBasicPlan && (
                  <Badge className="bg-gradient-to-r from-yellow-400 to-orange-500 text-white flex items-center gap-1">
                    <Crown className="h-3 w-3" />
                    Premium
                  </Badge>
                )}
              </div>
              {expiryDate && (
                <div className="text-sm text-muted-foreground">
                  {timeLeft === 'Expired' ? (
                    <span className="text-red-600 dark:text-red-400 font-medium">Plan Expired</span>
                  ) : isExpiringSoon ? (
                    <span className="text-yellow-600 dark:text-yellow-400 font-medium">
                      Expires in {timeLeft}
                    </span>
                  ) : (
                    <span>Expires in {timeLeft}</span>
                  )}
                </div>
              )}
            </div>
            <div className="text-left sm:text-right">
              {isBasicPlan ? (
                <Button
                  onClick={() => setShowUpgrade(true)}
                  className="w-full sm:w-auto bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 dark:from-blue-500 dark:to-purple-500 dark:hover:from-blue-600 dark:hover:to-purple-600 flex items-center gap-2"
                >
                  <TrendingUp className="h-4 w-4" />
                  Upgrade Now
                </Button>
              ) : (
                <Button
                  variant="outline"
                  onClick={() => setShowUpgrade(true)}
                  className="w-full sm:w-auto border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2"
                >
                  <Zap className="h-4 w-4" />
                  Extend Plan
                </Button>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Upgrade Prompt for Basic Users */}
      {isBasicPlan && (
        <Card className="border-blue-200 dark:border-blue-800 bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20">
          <CardContent className="p-6">
            <div className="flex flex-col sm:flex-row items-start sm:items-center gap-4">
              <div className="flex-shrink-0">
                <div className="w-12 h-12 bg-gradient-to-r from-blue-600 to-purple-600 rounded-lg flex items-center justify-center">
                  <Star className="h-6 w-6 text-white" />
                </div>
              </div>
              <div className="flex-1">
                <h3 className="font-semibold text-lg mb-1 text-gray-900 dark:text-white">
                  Unlock Premium Features
                </h3>
                <p className="text-sm text-muted-foreground mb-3">
                  Get access to advanced analytics, priority support, and unlimited API calls.
                </p>
                <div className="max-w-xs">
                  <QuickPayButton
                    packageId="gold"
                    packageName="Gold Plan"
                    amount={99}
                  />
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Expiry Warning */}
      {isExpiringSoon && !isBasicPlan && (
        <Card className="border-yellow-200 dark:border-yellow-800 bg-yellow-50 dark:bg-yellow-900/20">
          <CardContent className="p-4">
            <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3">
              <div className="w-8 h-8 bg-yellow-100 dark:bg-yellow-900 rounded-full flex items-center justify-center flex-shrink-0">
                <Zap className="h-4 w-4 text-yellow-600 dark:text-yellow-400" />
              </div>
              <div className="flex-1">
                <h4 className="font-semibold text-yellow-800 dark:text-yellow-200">
                  Plan Expiring Soon
                </h4>
                <p className="text-sm text-yellow-700 dark:text-yellow-300">
                  Your {currentPlan} plan expires in {timeLeft}. Renew now to continue enjoying premium features.
                </p>
              </div>
              <Button
                size="sm"
                className="w-full sm:w-auto bg-yellow-600 hover:bg-yellow-700 dark:bg-yellow-500 dark:hover:bg-yellow-600 flex items-center gap-1"
                onClick={() => setShowUpgrade(true)}
              >
                Renew
                <ArrowRight className="h-4 w-4" />
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Upgrade Modal/Section */}
      {showUpgrade && (
        <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
          <div className="bg-white dark:bg-gray-800 rounded-lg max-w-md w-full max-h-[90vh] overflow-y-auto border border-gray-200 dark:border-gray-700">
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-bold text-gray-900 dark:text-white">
                  {isBasicPlan ? 'Upgrade Your Plan' : 'Extend Your Plan'}
                </h2>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowUpgrade(false)}
                  className="text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300"
                >
                  ×
                </Button>
              </div>
            </div>
            <div className="p-4">
              <PaymentWidget
                title={isBasicPlan ? 'Choose Your Plan' : 'Extend Your Plan'}
                subtitle={isBasicPlan ? 'Unlock premium features' : 'Continue enjoying premium features'}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
