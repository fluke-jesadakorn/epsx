'use client';

import { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { createPaymentService } from '@/services/payment.service';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Check,
  Wallet,
  ArrowRight,
  Shield,
  Clock,
  Star,
  AlertCircle,
  Loader2,
  Copy,
} from 'lucide-react';
import {
  PACKAGES,
  LEVEL_BENEFITS,
  validatePayment,
} from '@/app/constants/packages';
import type { CurrencyType, PaymentError } from '@/app/constants/packages';
import { useAuth } from '@/context/auth-context';
import PaymentDetails from './PaymentDetails';
import { QRCodeCanvas } from 'qrcode.react';
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
    networks: ['TRC20'],
  },
  {
    id: 'USDT_BSC',
    name: 'USDT (BSC)',
    icon: <Wallet className="h-5 w-5" />,
    description: 'Low fees',
    processingTime: '1-5 min',
    fees: '$0.2',
    networks: ['BSC'],
  },
  {
    id: 'USDT_ERC20',
    name: 'USDT (ERC20)',
    icon: <Wallet className="h-5 w-5" />,
    description: 'Most secure',
    processingTime: '2-10 min',
    fees: '$2-15',
    networks: ['ERC20'],
  },
  // Temporarily disabled as per request
  // {
  //   id: 'credit_card',
  //   name: 'Credit Card',
  //   icon: <CreditCard className="h-5 w-5" />,
  //   description: 'Instant payment',
  //   processingTime: 'Instant',
  //   fees: '2.9%',
  //   popular: true
  // },
  // Temporarily disabled as per request
  // {
  //   id: 'apple_pay',
  //   name: 'Apple Pay',
  //   icon: <Smartphone className="h-5 w-5" />,
  //   description: 'Quick & secure',
  //   processingTime: 'Instant',
  //   fees: '2.9%'
  // }
];

export default function OneClickPayment({
  preselectedPackage = '',
  preselectedAmount = '',
  className = '',
}: OneClickPaymentProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { user } = useAuth();
  const paymentService = createPaymentService();

  // State management
  const [step, setStep] = useState<'select' | 'pay' | 'success'>('select');
  const [selectedPackage, setSelectedPackage] = useState(preselectedPackage);
  const [selectedPaymentMethod, setSelectedPaymentMethod] =
    useState('USDT_TRC20');
  const [amount, setAmount] = useState(preselectedAmount);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState(false);
  const [deposit, setDeposit] = useState<{ address: string; currency: string } | null>(null);

  // Get popular packages (skip free basic)
  const popularPackages = PACKAGES.filter(
    (pkg) => pkg.price > 0 && !pkg.id.startsWith('api_'),
  ).slice(0, 3);

  // Sync selectedPackage with query string on mount and when query changes
  useEffect(() => {
    const pkgFromQuery = searchParams.get('package');
    if (pkgFromQuery && pkgFromQuery !== selectedPackage) {
      setSelectedPackage(pkgFromQuery);
      const pkg = PACKAGES.find((p) => p.id === pkgFromQuery);
      if (pkg) setAmount(pkg.price.toString());
    } else if (!pkgFromQuery && !selectedPackage && popularPackages.length > 0) {
      setSelectedPackage(popularPackages[0].id);
      setAmount(popularPackages[0].price.toString());
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams, popularPackages]);

  // Update query string when user selects a package
  const handleSelectPackage = (pkgId: string, price: number) => {
    setSelectedPackage(pkgId);
    setAmount(price.toString());
    const params = new URLSearchParams(Array.from(searchParams.entries()));
    params.set('package', pkgId);
    router.replace(`?${params.toString()}`);
  };

  const selectedPackageData = PACKAGES.find(
    (pkg) => pkg.id === selectedPackage,
  );
  const selectedMethodData = PAYMENT_METHODS.find(
    (method) => method.id === selectedPaymentMethod,
  );

  const handleQuickPay = async () => {
    if (!selectedPackageData || !selectedMethodData) return;

    if (!user) {
      setError('Please login to continue with payment');
      return;
    }

    // For crypto payments, get deposit address and show custom UI
    if (selectedPaymentMethod.startsWith('USDT_')) {
      setIsProcessing(true);
      setError('');

      try {
        const response = await fetch('/api/v1/payments/crypto/deposit-address', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            currency: selectedMethodData.id,
            userId: user.uid,
            packageId: selectedPackage,
          }),
        });

        const result = await response.json();
        if (!response.ok || !result.deposit) {
          throw new Error(result.error || 'Failed to get deposit address');
        }

        setDeposit({ address: result.deposit.address, currency: result.deposit.currency });
        setStep('pay');
      } catch (err) {
        setError(
          err instanceof Error
            ? err.message
            : 'Failed to get deposit address',
        );
      } finally {
        setIsProcessing(false);
      }
      return;
    }

    // For card payments, process directly (if implemented)
    setIsProcessing(true);
    setError('');

    try {
      // Validate payment
      const validationError = validatePayment(
        Number(amount),
        selectedPaymentMethod as CurrencyType,
      );

      if (validationError) {
        throw new Error(getValidationErrorMessage(validationError));
      }

      // Process payment
      const transactionId = await paymentService.recordPayment(
        Number(amount),
        selectedPaymentMethod,
        `${selectedPackageData.name} purchase`,
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
      <Card
        className={`max-w-md mx-auto ${className} border-0 shadow-2xl bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20 relative overflow-hidden`}
      >
        <div className="absolute inset-0 bg-gradient-to-br from-green-400/10 via-emerald-400/10 to-teal-400/10"></div>
        <CardContent className="pt-8 text-center relative z-10">
          <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full flex items-center justify-center shadow-lg animate-bounce">
            <Check className="h-10 w-10 text-white" />
          </div>
          <h3 className="text-2xl font-bold mb-3 bg-gradient-to-r from-green-600 to-emerald-600 bg-clip-text text-transparent">
            Payment Successful! 🎉
          </h3>
          <p className="text-gray-600 dark:text-gray-300 mb-6 text-lg">
            Your {selectedPackageData?.name} has been activated.
          </p>
          <div className="inline-flex items-center gap-2 text-sm font-medium text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30 px-4 py-2 rounded-full">
            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
            Redirecting to dashboard...
          </div>
        </CardContent>
      </Card>
    );
  }

  // Custom payment details/QR code UI
  if (step === 'pay' && deposit) {
    return (
      <div className={`max-w-2xl mx-auto space-y-6 ${className}`}>
        <Card className="border-0 shadow-2xl bg-gradient-to-br from-white via-blue-50/50 to-cyan-50/50 dark:from-gray-800 dark:via-blue-900/20 dark:to-cyan-900/20 relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-blue-400/5 via-cyan-400/5 to-teal-400/5"></div>
          <CardHeader className="relative z-10">
            <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
              <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-full flex items-center justify-center">
                <Wallet className="h-5 w-5 text-white" />
              </div>
              Send Payment
              <div className="ml-auto text-2xl animate-bounce">💰</div>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-6 relative z-10">
            <div className="flex flex-col items-center gap-4">
              <div className="bg-white p-4 rounded-lg shadow-md">
                <div className="mb-2 font-bold text-gray-700 dark:text-gray-200">Scan QR to Pay</div>
                <div className="flex justify-center">
                  {/* QR code for address+amount+currency */}
                  <div className="bg-white p-2 rounded">
                    {/* @ts-ignore */}
                    <QRCodeCanvas
                      value={JSON.stringify({
                        address: deposit.address,
                        amount,
                        currency: deposit.currency,
                      })}
                      size={192}
                    />
                  </div>
                </div>
              </div>
              <div className="w-full">
                <div className="mb-1 text-gray-700 dark:text-gray-200 font-semibold">Payment Address</div>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={deposit.address}
                    readOnly
                    className="flex-1 p-2 border rounded bg-gray-50 text-xs"
                  />
                  <Button
                    onClick={() => {
                      navigator.clipboard.writeText(deposit.address);
                    }}
                    className="px-3 py-2 bg-gray-200 rounded hover:bg-gray-300 text-sm flex items-center gap-2"
                  >
                    <Copy className="h-4 w-4" />
                    Copy
                  </Button>
                </div>
              </div>
              <div className="w-full flex justify-between mt-2">
                <span className="font-bold text-blue-600">{amount} {deposit.currency}</span>
                <span className="font-bold text-gray-600">Network: {deposit.currency.split('_')[1]}</span>
              </div>
              <div className="w-full mt-4 flex gap-3">
                <Button
                  variant="outline"
                  onClick={() => setStep('select')}
                  className="flex-1 border border-gray-300 rounded hover:bg-gray-50"
                >
                  Back
                </Button>
                <Button
                  onClick={() => setSuccess(true)}
                  className="flex-1 bg-green-500 text-white rounded hover:bg-green-600"
                >
                  Sent Payment
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className={`max-w-4xl mx-auto space-y-6 ${className}`}>
      {step === 'select' && (
        <>
          {/* Quick Package Selection */}
          <Card className="border-0 shadow-2xl bg-gradient-to-br from-white via-pink-50/50 to-purple-50/50 dark:from-gray-800 dark:via-purple-900/20 dark:to-gray-800 relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-pink-400/5 via-purple-400/5 to-orange-400/5"></div>
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
                <div className="w-8 h-8 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-full flex items-center justify-center">
                  <Star className="h-5 w-5 text-white" />
                </div>
                Choose Your Plan
                <div className="ml-auto text-2xl animate-bounce">🎯</div>
              </CardTitle>
            </CardHeader>
            <CardContent className="relative z-10">
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
                {popularPackages.map((pkg) => (
                  <div
                    key={pkg.id}
                    className={`group relative p-6 rounded-2xl border-2 cursor-pointer transition-all duration-300 hover:scale-105 hover:shadow-xl hover:-translate-y-1 ${
                      selectedPackage === pkg.id
                        ? 'border-pink-400 bg-gradient-to-br from-pink-50 to-purple-50 dark:from-pink-900/30 dark:to-purple-900/30 shadow-xl scale-105'
                        : 'border-gray-200 dark:border-gray-600 hover:border-pink-300 dark:hover:border-pink-500 bg-white dark:bg-gray-700 hover:bg-gradient-to-br hover:from-pink-50 hover:to-purple-50 dark:hover:from-pink-900/20 dark:hover:to-purple-900/20'
                    }`}
                    onClick={() => handleSelectPackage(pkg.id, pkg.price)}
                  >
                    <div className="absolute inset-0 bg-gradient-to-br from-pink-400/10 via-purple-400/10 to-orange-400/10 rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>

                    <div className="relative z-10">
                      <div className="flex items-center justify-between mb-4">
                        <h4 className="font-bold text-gray-900 dark:text-white text-lg">
                          {pkg.name}
                        </h4>
                        {pkg.level === 'GOLD' && (
                          <Badge className="bg-gradient-to-r from-yellow-400 to-orange-500 text-white border-0 shadow-lg animate-pulse">
                            🔥 Popular
                          </Badge>
                        )}
                      </div>

                      <div className="mb-4">
                        <div className="flex items-baseline gap-1">
                          <span className="text-3xl font-black bg-gradient-to-r from-pink-600 via-purple-600 to-orange-600 bg-clip-text text-transparent">
                            ${pkg.price}
                          </span>
                          <span className="text-sm font-medium text-gray-500 dark:text-gray-400">
                            /month
                          </span>
                        </div>
                      </div>

                      <div className="space-y-2">
                        {LEVEL_BENEFITS[pkg.level]
                          .slice(0, 3)
                          .map((benefit, idx) => (
                            <div
                              key={idx}
                              className="flex items-center gap-2 text-sm"
                            >
                              <div className="w-4 h-4 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full flex items-center justify-center flex-shrink-0">
                                <Check className="h-2.5 w-2.5 text-white" />
                              </div>
                              <span className="text-gray-700 dark:text-gray-300 font-medium">
                                {benefit}
                              </span>
                            </div>
                          ))}
                      </div>

                      {selectedPackage === pkg.id && (
                        <div className="absolute -top-2 -right-2 w-6 h-6 bg-gradient-to-r from-pink-500 to-purple-500 rounded-full flex items-center justify-center shadow-lg animate-pulse">
                          <Check className="h-3 w-3 text-white" />
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Payment Method Selection */}
          <Card className="border-0 shadow-2xl bg-gradient-to-br from-white via-blue-50/50 to-cyan-50/50 dark:from-gray-800 dark:via-blue-900/20 dark:to-gray-800 relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-blue-400/5 via-cyan-400/5 to-teal-400/5"></div>
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
                <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-full flex items-center justify-center">
                  <Shield className="h-5 w-5 text-white" />
                </div>
                Payment Method
                <div className="ml-auto text-2xl animate-bounce">💳</div>
              </CardTitle>
            </CardHeader>
            <CardContent className="relative z-10">
              <div className="space-y-4">
                {PAYMENT_METHODS.map((method) => (
                  <div
                    key={method.id}
                    className={`group relative p-4 rounded-2xl border-2 cursor-pointer transition-all duration-300 hover:scale-[1.02] hover:shadow-xl ${
                      selectedPaymentMethod === method.id
                        ? 'border-blue-400 bg-gradient-to-br from-blue-50 to-cyan-50 dark:from-blue-900/30 dark:to-cyan-900/30 shadow-xl scale-[1.02]'
                        : 'border-gray-200 dark:border-gray-600 hover:border-blue-300 dark:hover:border-blue-500 bg-white dark:bg-gray-700 hover:bg-gradient-to-br hover:from-blue-50 hover:to-cyan-50 dark:hover:from-blue-900/20 dark:hover:to-cyan-900/20'
                    }`}
                    onClick={() => setSelectedPaymentMethod(method.id)}
                  >
                    <div className="absolute inset-0 bg-gradient-to-br from-blue-400/5 via-cyan-400/5 to-teal-400/5 rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>

                    <div className="relative z-10">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-4">
                          <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-xl flex items-center justify-center text-white shadow-lg">
                            {method.icon}
                          </div>
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2 flex-wrap mb-1">
                              <span className="font-bold text-gray-900 dark:text-white text-lg">
                                {method.name}
                              </span>
                              {method.popular && (
                                <Badge className="bg-gradient-to-r from-orange-400 to-red-500 text-white border-0 shadow-lg animate-pulse">
                                  🔥 Popular
                                </Badge>
                              )}
                            </div>
                            <div className="text-sm text-gray-600 dark:text-gray-300 font-medium">
                              {method.description}
                            </div>
                          </div>
                        </div>
                        <div className="flex items-center gap-4">
                          <div className="text-right text-sm hidden sm:block">
                            <div className="flex items-center gap-1 text-gray-600 dark:text-gray-300 font-medium mb-1">
                              <Clock className="h-4 w-4" />
                              {method.processingTime}
                            </div>
                            <div className="text-xs text-gray-500 dark:text-gray-400">
                              Fee:{' '}
                              <span className="font-semibold text-green-600 dark:text-green-400">
                                {method.fees}
                              </span>
                            </div>
                          </div>
                          {selectedPaymentMethod === method.id && (
                            <div className="w-6 h-6 bg-gradient-to-r from-blue-500 to-cyan-500 rounded-full flex items-center justify-center shadow-lg animate-pulse">
                              <Check className="h-4 w-4 text-white" />
                            </div>
                          )}
                        </div>
                      </div>
                      {/* Mobile-friendly fee display */}
                      <div className="sm:hidden mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
                        <div className="flex items-center justify-between text-sm">
                          <div className="flex items-center gap-2 text-gray-600 dark:text-gray-300">
                            <Clock className="h-4 w-4" />
                            <span className="font-medium">
                              {method.processingTime}
                            </span>
                          </div>
                          <div className="text-gray-500 dark:text-gray-400">
                            Fee:{' '}
                            <span className="font-semibold text-green-600 dark:text-green-400">
                              {method.fees}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Payment Summary & Action */}
          <Card className="border-0 shadow-2xl bg-gradient-to-br from-white via-green-50/50 to-emerald-50/50 dark:from-gray-800 dark:via-green-900/20 dark:to-gray-800 relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-green-400/5 via-emerald-400/5 to-teal-400/5"></div>
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
                <div className="w-8 h-8 bg-gradient-to-br from-green-500 to-emerald-500 rounded-full flex items-center justify-center">
                  <ArrowRight className="h-5 w-5 text-white" />
                </div>
                Payment Summary
                <div className="ml-auto text-2xl animate-bounce">💎</div>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6 relative z-10">
              <div className="space-y-4">
                <div className="flex justify-between items-center p-4 bg-white/60 dark:bg-gray-700/60 rounded-xl border border-gray-200/50 dark:border-gray-600/50">
                  <span className="text-gray-600 dark:text-gray-300 font-medium">
                    Package:
                  </span>
                  <span className="font-bold text-gray-900 dark:text-white text-lg">
                    {selectedPackageData?.name}
                  </span>
                </div>
                <div className="flex justify-between items-center p-4 bg-white/60 dark:bg-gray-700/60 rounded-xl border border-gray-200/50 dark:border-gray-600/50">
                  <span className="text-gray-600 dark:text-gray-300 font-medium">
                    Payment Method:
                  </span>
                  <span className="font-bold text-gray-900 dark:text-white text-lg">
                    {selectedMethodData?.name}
                  </span>
                </div>
                <div className="flex justify-between items-center p-4 bg-white/60 dark:bg-gray-700/60 rounded-xl border border-gray-200/50 dark:border-gray-600/50">
                  <span className="text-gray-600 dark:text-gray-300 font-medium">
                    Processing Time:
                  </span>
                  <span className="font-semibold text-blue-600 dark:text-blue-400">
                    {selectedMethodData?.processingTime}
                  </span>
                </div>
              </div>

              <div className="h-px bg-gradient-to-r from-transparent via-gray-300 dark:via-gray-600 to-transparent"></div>

              <div className="flex justify-between items-center p-6 bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/30 dark:to-emerald-900/30 rounded-2xl border-2 border-green-200 dark:border-green-700">
                <span className="text-gray-900 dark:text-white font-bold text-xl">
                  Total:
                </span>
                <span className="text-3xl font-black bg-gradient-to-r from-green-600 via-emerald-600 to-teal-600 bg-clip-text text-transparent">
                  ${amount}
                </span>
              </div>

              {error && (
                <Alert className="border-red-200 bg-gradient-to-r from-red-50 to-pink-50 dark:from-red-900/20 dark:to-pink-900/20 relative overflow-hidden">
                  <div className="absolute inset-0 bg-gradient-to-r from-red-400/10 to-pink-400/10"></div>
                  <AlertCircle className="h-4 w-4 text-red-600 dark:text-red-400" />
                  <AlertDescription className="text-red-700 dark:text-red-300 font-medium">
                    {error}
                  </AlertDescription>
                </Alert>
              )}

              <Button
                onClick={handleQuickPay}
                disabled={isProcessing || !selectedPackageData || !amount}
                className="w-full h-14 text-lg font-bold bg-gradient-to-r from-pink-500 via-purple-500 to-orange-500 hover:from-pink-600 hover:via-purple-600 hover:to-orange-600 text-white border-0 shadow-2xl transition-all duration-300 hover:scale-[1.02] hover:shadow-3xl disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
              >
                {isProcessing ? (
                  <div className="flex items-center gap-3">
                    <Loader2 className="w-5 h-5 animate-spin" />
                    <span>Processing...</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-3">
                    <span>
                      {selectedPaymentMethod.startsWith('USDT_')
                        ? 'Continue Payment'
                        : `Pay $${amount} Now`}
                    </span>
                    <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform" />
                    <span className="text-xl">🚀</span>
                  </div>
                )}
              </Button>

              <div className="flex items-center justify-center gap-2 text-sm text-gray-600 dark:text-gray-400 bg-white/60 dark:bg-gray-700/60 rounded-xl p-3">
                <Shield className="h-4 w-4 text-green-500" />
                <span className="font-medium">
                  Secure payment powered by blockchain technology
                </span>
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              </div>
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}
