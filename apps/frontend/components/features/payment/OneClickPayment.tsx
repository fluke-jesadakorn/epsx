'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { createPaymentService } from '@/services/payment.service';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Check, CreditCard, Smartphone, Wallet, ArrowRight, Shield, Clock, Star } from 'lucide-react';
import { PACKAGES, LEVEL_BENEFITS, validatePayment } from '@/app/constants/packages';
import type { CurrencyType, PaymentError } from '@/app/constants/packages';
import PaymentDetails from './PaymentDetails';

interface OneClickPaymentProps {
  preselectedPackage?: string;
  preselectedAmount?: string;
  className?: string;
}

interface PaymentMethod {
  id: string;
  name: string;
  icon: React.ReactNode;
  description: string;
  processingTime: string;
  fees: string;
  popular?: boolean;
  networks?: string[];
}

const PAYMENT_METHODS: PaymentMethod[] = [
  {
    id: 'USDT_TRC20',
    name: 'USDT (TRC20)',
    icon: <Wallet className="h-5 w-5" />,
    description: 'Fastest & cheapest',
    processingTime: '1-3 min',
    fees: '$0.1',
    popular: true,
    networks: ['TRC20']
  },
  {
    id: 'USDT_BSC',
    name: 'USDT (BSC)',
    icon: <Wallet className="h-5 w-5" />,
    description: 'Low fees',
    processingTime: '1-5 min',
    fees: '$0.2',
    networks: ['BSC']
  },
  {
    id: 'USDT_ERC20',
    name: 'USDT (ERC20)',
    icon: <Wallet className="h-5 w-5" />,
    description: 'Most secure',
    processingTime: '2-10 min',
    fees: '$2-15',
    networks: ['ERC20']
  },
  {
    id: 'credit_card',
    name: 'Credit Card',
    icon: <CreditCard className="h-5 w-5" />,
    description: 'Instant payment',
    processingTime: 'Instant',
    fees: '2.9%',
    popular: true
  },
  {
    id: 'apple_pay',
    name: 'Apple Pay',
    icon: <Smartphone className="h-5 w-5" />,
    description: 'Quick & secure',
    processingTime: 'Instant',
    fees: '2.9%'
  }
];

export default function OneClickPayment({ 
  preselectedPackage = '', 
  preselectedAmount = '',
  className = '' 
}: OneClickPaymentProps) {
  const router = useRouter();
  const paymentService = createPaymentService();

  // State management
  const [step, setStep] = useState<'select' | 'details' | 'success'>('select');
  const [selectedPackage, setSelectedPackage] = useState(preselectedPackage);
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState('USDT_TRC20');
  const [amount, setAmount] = useState(preselectedAmount);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState(false);

  // Get popular packages (skip free basic)
  const popularPackages = PACKAGES.filter(pkg => 
    pkg.price > 0 && !pkg.id.startsWith('api_')
  ).slice(0, 3);

  // Auto-select first package if none selected
  useEffect(() => {
    if (!selectedPackage && popularPackages.length > 0) {
      setSelectedPackage(popularPackages[0].id);
      setAmount(popularPackages[0].price.toString());
    }
  }, [selectedPackage, popularPackages]);

  const selectedPackageData = PACKAGES.find(pkg => pkg.id === selectedPackage);
  const selectedMethodData = PAYMENT_METHODS.find(method => method.id === selectedPaymentMethod);

  const handleQuickPay = async () => {
    if (!selectedPackageData || !selectedMethodData) return;

    // For crypto payments, go to details step
    if (selectedPaymentMethod.startsWith('USDT_')) {
      setStep('details');
      return;
    }

    // For card payments, process directly
    setIsProcessing(true);
    setError('');

    try {
      // Validate payment
      const validationError = validatePayment(
        Number(amount),
        selectedPaymentMethod as CurrencyType
      );

      if (validationError) {
        throw new Error(getValidationErrorMessage(validationError));
      }

      // Process payment
      const transactionId = await paymentService.recordPayment(
        Number(amount),
        selectedPaymentMethod,
        `${selectedPackageData.name} purchase`
      );

      if (transactionId) {
        setSuccess(true);
        setTimeout(() => {
          router.push('/dashboard?payment=success');
        }, 2000);
      } else {
        throw new Error('Payment processing failed');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Payment failed');
    } finally {
      setIsProcessing(false);
    }
  };

  const getValidationErrorMessage = (error: PaymentError): string => {
    switch (error.type) {
      case 'INSUFFICIENT_AMOUNT':
        return `Minimum amount: ${error.minAmount} ${error.currency}`;
      case 'INVALID_CURRENCY':
        return 'Selected payment method is not supported';
      case 'NETWORK_ERROR':
        return 'Network error. Please try again.';
      case 'TRANSACTION_FAILED':
        return error.reason || 'Transaction failed';
      default:
        return 'Payment validation failed';
    }
  };

  if (success) {
    return (
      <Card className={`max-w-md mx-auto ${className} border-green-200 dark:border-green-800`}>
        <CardContent className="pt-8 text-center">
          <div className="w-16 h-16 mx-auto mb-4 bg-green-100 dark:bg-green-900 rounded-full flex items-center justify-center">
            <Check className="h-8 w-8 text-green-600 dark:text-green-400" />
          </div>
          <h3 className="text-xl font-semibold mb-2 text-gray-900 dark:text-white">Payment Successful!</h3>
          <p className="text-muted-foreground mb-4">
            Your {selectedPackageData?.name} has been activated.
          </p>
          <div className="text-sm text-muted-foreground">
            Redirecting to dashboard...
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`max-w-4xl mx-auto space-y-6 ${className}`}>
      {step === 'details' && (
        <PaymentDetails
          selectedPackage={selectedPackage}
          selectedMethod={selectedPaymentMethod}
          amount={amount}
          onBack={() => setStep('select')}
          onSuccess={() => {
            setSuccess(true);
            setTimeout(() => {
              router.push('/dashboard?payment=success');
            }, 2000);
          }}
        />
      )}

      {step === 'select' && (
        <>
          {/* Quick Package Selection */}
          <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-gray-900 dark:text-white">
                <Star className="h-5 w-5 text-yellow-500" />
                Choose Your Plan
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {popularPackages.map((pkg) => (
                  <div
                    key={pkg.id}
                    className={`p-4 rounded-lg border-2 cursor-pointer transition-all hover:scale-[1.02] hover:shadow-md ${
                      selectedPackage === pkg.id
                        ? 'border-primary bg-primary/5 dark:bg-primary/10 shadow-md'
                        : 'border-gray-200 dark:border-gray-600 hover:border-primary/50 dark:hover:border-primary/50 bg-white dark:bg-gray-700'
                    }`}
                    onClick={() => {
                      setSelectedPackage(pkg.id);
                      setAmount(pkg.price.toString());
                    }}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <h4 className="font-semibold text-gray-900 dark:text-white text-sm sm:text-base">{pkg.name}</h4>
                      {pkg.level === 'GOLD' && (
                        <Badge className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 text-xs">
                          Popular
                        </Badge>
                      )}
                    </div>
                    <p className="text-xl sm:text-2xl font-bold text-primary mb-2">
                      ${pkg.price}
                    </p>
                    <div className="text-xs sm:text-sm text-muted-foreground space-y-1">
                      {LEVEL_BENEFITS[pkg.level].slice(0, 2).map((benefit, idx) => (
                        <div key={idx} className="flex items-center gap-1">
                          <Check className="h-3 w-3 text-green-500 flex-shrink-0" />
                          <span className="line-clamp-1">{benefit}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Payment Method Selection */}
          <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-gray-900 dark:text-white">
                <Shield className="h-5 w-5 text-blue-500" />
                Payment Method
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {PAYMENT_METHODS.map((method) => (
                  <div 
                    key={method.id}
                    className={`p-3 sm:p-4 rounded-lg border-2 cursor-pointer transition-all hover:shadow-md ${
                      selectedPaymentMethod === method.id
                        ? 'border-primary bg-primary/5 dark:bg-primary/10 shadow-md'
                        : 'border-gray-200 dark:border-gray-600 hover:border-primary/50 dark:hover:border-primary/50 bg-white dark:bg-gray-700'
                    }`}
                    onClick={() => setSelectedPaymentMethod(method.id)}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="text-primary">{method.icon}</div>
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2 flex-wrap">
                            <span className="font-medium text-gray-900 dark:text-white text-sm sm:text-base">{method.name}</span>
                            {method.popular && (
                              <Badge variant="secondary" className="text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                Popular
                              </Badge>
                            )}
                          </div>
                          <div className="text-xs sm:text-sm text-muted-foreground">
                            {method.description}
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-3">
                        <div className="text-right text-xs sm:text-sm hidden sm:block">
                          <div className="flex items-center gap-1 text-muted-foreground">
                            <Clock className="h-3 w-3" />
                            {method.processingTime}
                          </div>
                          <div className="text-xs text-muted-foreground">
                            Fee: {method.fees}
                          </div>
                        </div>
                        {selectedPaymentMethod === method.id && (
                          <div className="w-4 h-4 rounded-full bg-primary flex items-center justify-center">
                            <Check className="h-3 w-3 text-white" />
                          </div>
                        )}
                      </div>
                    </div>
                    {/* Mobile-friendly fee display */}
                    <div className="sm:hidden mt-2 text-xs text-muted-foreground">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          {method.processingTime}
                        </div>
                        <div>Fee: {method.fees}</div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Payment Summary & Action */}
          <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
            <CardHeader>
              <CardTitle className="text-gray-900 dark:text-white">Payment Summary</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span className="text-gray-600 dark:text-gray-300">Package:</span>
                  <span className="font-semibold text-gray-900 dark:text-white">{selectedPackageData?.name}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-gray-600 dark:text-gray-300">Payment Method:</span>
                  <span className="font-semibold text-gray-900 dark:text-white">{selectedMethodData?.name}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-gray-600 dark:text-gray-300">Processing Time:</span>
                  <span className="text-sm text-muted-foreground">
                    {selectedMethodData?.processingTime}
                  </span>
                </div>
              </div>
              <Separator className="dark:bg-gray-600" />
              <div className="flex justify-between items-center text-lg font-semibold">
                <span className="text-gray-900 dark:text-white">Total:</span>
                <span className="text-primary text-xl">${amount}</span>
              </div>

              {error && (
                <Alert className="border-destructive bg-red-50 dark:bg-red-900/20">
                  <AlertDescription className="text-destructive">
                    {error}
                  </AlertDescription>
                </Alert>
              )}

              <Button
                onClick={handleQuickPay}
                disabled={isProcessing || !selectedPackageData || !amount}
                className="w-full h-12 text-lg font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 dark:from-blue-500 dark:to-purple-500 dark:hover:from-blue-600 dark:hover:to-purple-600"
              >
                {isProcessing ? (
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    Processing...
                  </div>
                ) : (
                  <div className="flex items-center gap-2">
                    {selectedPaymentMethod.startsWith('USDT_') ? 'Continue' : `Pay $${amount}`}
                    <ArrowRight className="h-4 w-4" />
                  </div>
                )}
              </Button>

              <div className="text-xs text-center text-muted-foreground">
                <Shield className="h-3 w-3 inline mr-1" />
                Secure payment powered by blockchain technology
              </div>
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}
