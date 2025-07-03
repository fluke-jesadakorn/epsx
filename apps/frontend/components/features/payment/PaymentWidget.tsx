'use client';

import { useState } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  Zap, 
  ArrowRight, 
  Check, 
  Star, 
  Shield,
  Clock
} from 'lucide-react';

interface PaymentWidgetProps {
  title?: string;
  subtitle?: string;
  packages?: Array<{
    id: string;
    name: string;
    price: number;
    popular?: boolean;
    features: string[];
  }>;
  className?: string;
}

const defaultPackages = [
  {
    id: 'silver',
    name: 'Silver Plan',
    price: 29,
    features: ['5 API calls/day', 'Email support', 'Basic analytics']
  },
  {
    id: 'gold',
    name: 'Gold Plan',
    price: 99,
    popular: true,
    features: ['50 API calls/day', 'Priority support', 'Advanced analytics', 'Real-time data']
  },
  {
    id: 'platinum',
    name: 'Platinum Plan',
    price: 299,
    features: ['Unlimited API calls', '24/7 support', 'Premium analytics', 'Custom integrations']
  }
];

export default function PaymentWidget({ 
  title = 'Upgrade Your Plan',
  subtitle = 'Choose the perfect plan for your needs',
  packages = defaultPackages,
  className = ''
}: PaymentWidgetProps) {
  const [selectedPackage, setSelectedPackage] = useState(packages.find(p => p.popular)?.id || packages[0]?.id);

  const selectedPkg = packages.find(p => p.id === selectedPackage);

  const handlePayNow = () => {
    if (selectedPkg) {
      window.open(`/quick-payment?package=${selectedPkg.id}&amount=${selectedPkg.price}`, '_blank');
    }
  };

  return (
    <Card className={`w-full max-w-md mx-auto shadow-lg border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 ${className}`}>
      <CardContent className="p-6">
        <div className="text-center mb-6">
          <h3 className="text-xl font-bold mb-2 text-gray-900 dark:text-white">{title}</h3>
          <p className="text-sm text-muted-foreground">{subtitle}</p>
        </div>

        {/* Package Selection */}
        <div className="space-y-2 mb-6">
          {packages.map((pkg) => (
            <div
              key={pkg.id}
              className={`p-3 rounded-lg border-2 cursor-pointer transition-all hover:shadow-md ${
                selectedPackage === pkg.id
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 dark:border-blue-400 shadow-md'
                  : 'border-gray-200 dark:border-gray-600 hover:border-blue-300 dark:hover:border-blue-400 bg-white dark:bg-gray-700'
              }`}
              onClick={() => setSelectedPackage(pkg.id)}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 min-w-0 flex-1">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1 flex-wrap">
                      <span className="font-semibold text-gray-900 dark:text-white text-sm sm:text-base">{pkg.name}</span>
                      {pkg.popular && (
                        <Badge className="text-xs bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                          <Star className="h-3 w-3 mr-1" />
                          Popular
                        </Badge>
                      )}
                    </div>
                    <div className="text-xs sm:text-sm text-muted-foreground truncate">
                      {pkg.features[0]}
                    </div>
                  </div>
                </div>
                <div className="text-right flex items-center gap-2">
                  <div className="text-lg font-bold text-blue-600 dark:text-blue-400">
                    ${pkg.price}
                  </div>
                  {selectedPackage === pkg.id && (
                    <Check className="h-4 w-4 text-blue-500 flex-shrink-0" />
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Selected Package Details */}
        {selectedPkg && (
          <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-6">
            <h4 className="font-semibold mb-2 text-gray-900 dark:text-white">What&apos;s included:</h4>
            <ul className="space-y-1">
              {selectedPkg.features.map((feature, index) => (
                <li key={index} className="text-sm flex items-center gap-2">
                  <Check className="h-3 w-3 text-green-500 flex-shrink-0" />
                  <span className="text-gray-600 dark:text-gray-300">{feature}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Payment CTA */}
        <div className="space-y-3">
          <Button
            onClick={handlePayNow}
            className="w-full h-12 text-base sm:text-lg font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 dark:from-blue-500 dark:to-purple-500 dark:hover:from-blue-600 dark:hover:to-purple-600 shadow-lg hover:shadow-xl transition-all duration-300"
          >
            <Zap className="mr-2 h-5 w-5" />
            Pay ${selectedPkg?.price} Now
            <ArrowRight className="ml-2 h-4 w-4" />
          </Button>

          <div className="flex items-center justify-center gap-4 text-xs text-muted-foreground">
            <div className="flex items-center gap-1">
              <Shield className="h-3 w-3" />
              <span>Secure</span>
            </div>
            <div className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              <span>Instant</span>
            </div>
            <div className="flex items-center gap-1">
              <Check className="h-3 w-3" />
              <span>Guaranteed</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
