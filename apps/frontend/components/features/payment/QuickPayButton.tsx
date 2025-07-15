'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Zap, CreditCard, Wallet, ArrowRight, Check, X } from 'lucide-react';
import { useRouter } from 'next/navigation';

interface QuickPayButtonProps {
  packageId: string;
  packageName: string;
  amount: number;
  onPaymentStart?: () => void;
  className?: string;
}

export default function QuickPayButton({
  packageId,
  packageName,
  amount,
  onPaymentStart,
  className = '',
}: QuickPayButtonProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [selectedMethod, setSelectedMethod] = useState<'crypto' | 'card'>(
    'crypto',
  );
  const router = useRouter();

  const handleQuickPay = () => {
    onPaymentStart?.();
    const paymentUrl = `/quick-payment?package=${packageId}&amount=${amount}`;
    window.open(paymentUrl, '_blank');
  };

  const handleInlinePayment = () => {
    if (selectedMethod === 'card') {
      // Redirect to card payment
      router.push(`/payment/card?package=${packageId}&amount=${amount}`);
    } else {
      // Open crypto payment
      handleQuickPay();
    }
  };

  return (
    <div className={`w-full ${className}`}>
      {/* Main quick pay button */}
      <Button
        onClick={handleQuickPay}
        className="w-full h-12 text-base sm:text-lg font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 dark:from-blue-500 dark:to-purple-500 dark:hover:from-blue-600 dark:hover:to-purple-600 shadow-lg hover:shadow-xl transition-all duration-300 flex items-center gap-2"
      >
        <Zap className="h-5 w-5" />
        Quick Pay ${amount}
      </Button>

      {/* Expandable payment options */}
      <div className="mt-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => setIsExpanded(!isExpanded)}
          className="w-full text-xs border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700"
        >
          {isExpanded ? 'Hide' : 'More'} Payment Options
          {isExpanded ? (
            <X className="ml-1 h-3 w-3" />
          ) : (
            <ArrowRight className="ml-1 h-3 w-3" />
          )}
        </Button>
      </div>

      {/* Expanded payment options */}
      {isExpanded && (
        <Card className="mt-2 border-dashed border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 shadow-lg">
          <CardContent className="p-4 space-y-3">
            <div className="text-sm font-medium text-center text-gray-900 dark:text-white">
              Choose Payment Method
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <div
                className={`p-3 rounded-lg border-2 cursor-pointer transition-all ${
                  selectedMethod === 'crypto'
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 dark:border-blue-400'
                    : 'border-gray-200 dark:border-gray-600 hover:border-blue-300 dark:hover:border-blue-400 bg-white dark:bg-gray-700'
                }`}
                onClick={() => setSelectedMethod('crypto')}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Wallet className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      Crypto
                    </span>
                  </div>
                  {selectedMethod === 'crypto' && (
                    <Check className="h-4 w-4 text-blue-500" />
                  )}
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  USDT • 1-3 min
                </div>
              </div>

              <div
                className={`p-3 rounded-lg border-2 cursor-pointer transition-all ${
                  selectedMethod === 'card'
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 dark:border-blue-400'
                    : 'border-gray-200 dark:border-gray-600 hover:border-blue-300 dark:hover:border-blue-400 bg-white dark:bg-gray-700'
                }`}
                onClick={() => setSelectedMethod('card')}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <CreditCard className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      Card
                    </span>
                  </div>
                  {selectedMethod === 'card' && (
                    <Check className="h-4 w-4 text-blue-500" />
                  )}
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  Instant • 2.9% fee
                </div>
              </div>
            </div>

            <div className="flex items-center justify-between text-sm border-t border-gray-200 dark:border-gray-600 pt-3">
              <span className="text-gray-600 dark:text-gray-300">
                Total for {packageName}:
              </span>
              <span className="font-semibold text-gray-900 dark:text-white">
                ${amount}
              </span>
            </div>

            <Button
              onClick={handleInlinePayment}
              className="w-full bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 dark:from-blue-500 dark:to-purple-500 dark:hover:from-blue-600 dark:hover:to-purple-600"
              size="sm"
            >
              Pay with {selectedMethod === 'crypto' ? 'Crypto' : 'Card'}
              <ArrowRight className="ml-2 h-4 w-4" />
            </Button>

            <div className="text-xs text-center text-muted-foreground">
              <Badge
                variant="outline"
                className="text-xs border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/20"
              >
                <Check className="h-3 w-3 mr-1 text-green-600 dark:text-green-400" />
                Secure & Instant
              </Badge>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
