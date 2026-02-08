'use client';

import { useState } from 'react';
import { PACKAGES } from '@/app/constants/packages';
import { Check, ArrowRight, Star, CreditCard, Wallet } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';

interface SelectPackageProps {
  amount: string;
  currency: string;
  onAmountChange: (amount: string) => void;
  onCurrencyChange: (currency: string) => void;
  onNext: () => void;
  className?: string;
}

export function SelectPackage({
  amount,
  currency,
  onAmountChange,
  onCurrencyChange,
  onNext,
  className = '',
}: SelectPackageProps) {
  const [isLoading, setIsLoading] = useState(false);

  // Filter out free packages and API packages for standard plans display  
  const standardPackages = PACKAGES.filter(pkg => 
    pkg.price > 0 && !pkg.id.startsWith('api_')
  );
  
  // API packages separately
  const apiPackages = PACKAGES.filter(pkg => 
    pkg.id.startsWith('api_') && pkg.price > 0
  );

  const handleSelectPackage = (packageId: string) => {
    const selectedPackage = PACKAGES.find(pkg => pkg.id === packageId);
    if (selectedPackage) {
      onAmountChange(selectedPackage.price.toString());
    }
  };

  const handleProceed = async () => {
    setIsLoading(true);
    try {
      await onNext();
    } finally {
      setIsLoading(false);
    }
  };

  // Payment method options
  const paymentMethods = [
    { id: 'USDT_TRC20', name: 'USDT (TRC20)', icon: <Wallet className="h-5 w-5" /> },
    { id: 'USDT_BSC', name: 'USDT (BSC)', icon: <Wallet className="h-5 w-5" /> },
    { id: 'USDT_ERC20', name: 'USDT (ERC20)', icon: <Wallet className="h-5 w-5" /> },
  ];

  return (
    <div className={`space-y-6 ${className}`}>
      <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-gray-900 dark:text-white">
            <Star className="h-5 w-5 text-yellow-500" />
            Select a Plan
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {standardPackages.map((pkg) => (
              <div
                key={pkg.id}
                className={`p-4 rounded-lg border-2 cursor-pointer transition-all hover:scale-[1.02] hover:shadow-md ${
                  amount === pkg.price.toString()
                    ? 'border-primary bg-primary/5 dark:bg-primary/10 shadow-md'
                    : 'border-gray-200 dark:border-gray-600 hover:border-primary/50 dark:hover:border-primary/50 bg-white dark:bg-gray-700'
                }`}
                onClick={() => handleSelectPackage(pkg.id)}
              >
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-semibold text-gray-900 dark:text-white">{pkg.name}</h4>
                  {pkg.displayTier === 'GOLD' && (
                    <div className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-200 text-xs px-2 py-0.5 rounded-full">
                      Popular
                    </div>
                  )}
                </div>
                <p className="text-xl font-bold text-primary mb-2">
                  ${pkg.price}
                </p>
                <div className="text-xs sm:text-sm text-muted-foreground space-y-1">
                  {pkg.features.map((feature, idx) => (
                    <div key={idx} className="flex items-center gap-1">
                      <Check className="h-3 w-3 text-green-500 flex-shrink-0" />
                      <span>{feature}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>

          {apiPackages.length > 0 && (
            <div className="mt-8">
              <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">API Plans</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {apiPackages.map((pkg) => (
                  <div
                    key={pkg.id}
                    className={`p-4 rounded-lg border-2 cursor-pointer transition-all hover:scale-[1.02] hover:shadow-md ${
                      amount === pkg.price.toString()
                        ? 'border-primary bg-primary/5 dark:bg-primary/10 shadow-md'
                        : 'border-gray-200 dark:border-gray-600 hover:border-primary/50 dark:hover:border-primary/50 bg-white dark:bg-gray-700'
                    }`}
                    onClick={() => handleSelectPackage(pkg.id)}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <h4 className="font-semibold text-gray-900 dark:text-white">{pkg.name}</h4>
                    </div>
                    <p className="text-xl font-bold text-primary mb-2">
                      ${pkg.price}
                    </p>
                    <div className="text-xs sm:text-sm text-muted-foreground space-y-1">
                      {pkg.features.map((feature, idx) => (
                        <div key={idx} className="flex items-center gap-1">
                          <Check className="h-3 w-3 text-green-500 flex-shrink-0" />
                          <span>{feature}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-gray-900 dark:text-white">
            <CreditCard className="h-5 w-5 text-blue-500" />
            Payment Method
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {paymentMethods.map((method) => (
              <div 
                key={method.id}
                className={`flex items-center space-x-2 p-3 rounded-lg border cursor-pointer ${
                  currency === method.id
                    ? 'border-primary bg-primary/5 dark:bg-primary/10'
                    : 'border-gray-200 dark:border-gray-600'
                }`}
                onClick={() => onCurrencyChange(method.id)}
              >
                <div className={`w-4 h-4 rounded-full border ${
                  currency === method.id
                    ? 'border-primary bg-primary'
                    : 'border-gray-400'
                }`}>
                  {currency === method.id && (
                    <div className="w-full h-full flex items-center justify-center">
                      <Check className="h-3 w-3 text-white" />
                    </div>
                  )}
                </div>
                <div className="flex-1 flex items-center gap-2">
                  {method.icon}
                  <span className="font-medium">{method.name}</span>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-6 pt-6 border-t border-gray-200 dark:border-gray-600">
            <div className="flex justify-between items-center text-lg font-semibold mb-4">
              <span className="text-gray-900 dark:text-white">Total:</span>
              <span className="text-primary text-xl">${amount}</span>
            </div>
            
            <Button
              onClick={handleProceed}
              disabled={isLoading || !amount}
              className="w-full h-12 text-base font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 dark:from-blue-500 dark:to-purple-500 dark:hover:from-blue-600 dark:hover:to-purple-600"
            >
              {isLoading ? (
                <div className="flex items-center gap-2">
                  <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                  Processing...
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  Continue to Payment
                  <ArrowRight className="h-4 w-4" />
                </div>
              )}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
