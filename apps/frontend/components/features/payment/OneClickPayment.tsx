'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
// Plans are fetched dynamically from API - no hardcoded fallback
import { cn } from '@/lib/utils';
import {
  AlertCircle,
  ArrowLeft,
  Check,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Lock,
  Shield,
  Smartphone,
  Sparkles,
  Zap,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { API_ROUTES } from '../../../../../shared/config/route-constants';
import { env } from '../../../../../shared/env/schema';
import MetaMaskPayment from './MetaMaskPayment';
import { UpgradeBanner } from './UpgradeBanner';

interface OneClickPaymentProps {
  className?: string;
  preselectedPackage?: string;
}

// Raw API response interface (backend data)
interface ApiPaymentPlan {
  id: number | string;
  name: string;
  plan_type: string;
  base_price?: number;
  current_price: number | string; // Backend returns string
  effective_price?: number; // Calculated with promotions
  currency: string;
  features: string[] | string; // Can be JSON string from API
  affiliate_commission_rate?: number;
  display_order?: number;
  is_active: boolean;
  is_highlighted?: boolean;
  created_at: string;
  updated_at: string;
  // Promotion fields
  promotional_badge?: string;
  promotional_message?: string;
  discount_type?: string;
  discount_value?: number;
  max_discount_amount?: number;
  promotion_active?: boolean;
  promotion_status?: string;
  promotion_discount?: number;
  promotion_ends_at?: string;
}

// UI-enhanced payment package interface
interface PaymentPackage
  extends Omit<ApiPaymentPlan, 'features' | 'current_price' | 'id'> {
  id: number | string; // Allow string for UUIDs
  original_plan_id: number | string; // Original plan ID from backend (for contract calls)
  features: string[]; // Always array in UI
  current_price: number; // Always number in UI (parsed from backend string)
  base_price: number; // Always number in UI
  // UI fields (derived)
  icon?: string;
  description?: string;
  popular?: boolean;
}

type PaymentStep = 'package' | 'payment' | 'confirmation';

// API helper function to fetch plans
const fetchPlans = async (): Promise<PaymentPackage[]> => {
  try {
    const baseUrl = env.BACKEND_URL;
    const apiUrl = `${baseUrl}${API_ROUTES.PUBLIC.PLANS}`;


    const response = await fetch(apiUrl, {
      method: 'GET',
      headers: {
        Accept: 'application/json',
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error('[OneClickPayment] Failed to fetch plans:', {
        status: response.status,
        statusText: response.statusText,
        error: errorText,
      });
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();

    // Backend returns wrapped response: { success, data, message }
    if (!result.success || !result.data || !Array.isArray(result.data)) {
      console.error('[OneClickPayment] Invalid response format');
      throw new Error('Invalid API response format');
    }

    const plans: ApiPaymentPlan[] = result.data;

    // Transform API response to include UI-specific fields
    return plans.map((plan: ApiPaymentPlan, index: number): PaymentPackage => {
      // Parse prices (backend returns strings) with comprehensive validation
      let currentPrice: number;
      if (typeof plan.current_price === 'string') {
        currentPrice = parseFloat(plan.current_price);
      } else if (typeof plan.current_price === 'number') {
        currentPrice = plan.current_price;
      } else {
        currentPrice = 0;
      }

      // Validate parsed price - if invalid (NaN or negative), try effective_price as fallback
      // Note: Price of 0 is valid for free plans
      if (isNaN(currentPrice) || currentPrice < 0) {
        console.warn('⚠️ Invalid current_price, trying effective_price as fallback:', {
          plan_id: plan.id,
          plan_name: plan.name,
          current_price: plan.current_price,
          effective_price: plan.effective_price
        });

        // Try effective_price as fallback
        if (typeof plan.effective_price === 'number' && plan.effective_price >= 0) {
          currentPrice = plan.effective_price;
        } else if (typeof plan.effective_price === 'string') {
          const parsed = parseFloat(plan.effective_price);
          if (!isNaN(parsed) && parsed >= 0) {
            currentPrice = parsed;
          }
        }
      }

      // Final validation - if still invalid, log error
      if (isNaN(currentPrice) || currentPrice < 0) {
        console.error('❌ CRITICAL: Plan has invalid price after all fallbacks:', {
          plan_id: plan.id,
          plan_name: plan.name,
          plan_type: plan.plan_type,
          current_price: plan.current_price,
          effective_price: plan.effective_price,
          parsed_current: currentPrice
        });
        currentPrice = 0; // Default to 0 to prevent NaN propagation
      }

      const basePrice = plan.base_price
        ? typeof plan.base_price === 'string'
          ? parseFloat(plan.base_price)
          : plan.base_price
        : currentPrice;

      const effectivePrice = plan.effective_price ?? currentPrice;

      // Parse original plan ID from backend
      const originalPlanId = plan.id;

      return {
        ...plan,
        id: originalPlanId,
        original_plan_id: originalPlanId,
        current_price: currentPrice,
        base_price: basePrice,
        effective_price: effectivePrice,
        icon: getIconForPlan(plan.plan_type),
        description: getDescriptionForPlan(plan.plan_type),
        popular: plan.is_highlighted || plan.plan_type === 'professional',
        features: Array.isArray(plan.features)
          ? plan.features
          : typeof plan.features === 'string'
            ? JSON.parse(plan.features)
            : getDefaultFeaturesForPlan(plan.plan_type),
      };
    });
  } catch (error) {
    console.error('[OneClickPayment] Error fetching plans:', error);
    throw error; // Let caller handle the error
  }
};

// Helper functions for UI data
const getIconForPlan = (planType: string): string => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic':
      return '🚀';
    case 'professional':
    case 'pro':
      return '⭐';
    case 'enterprise':
    case 'premium':
      return '👑';
    default:
      return '📊';
  }
};

const getDescriptionForPlan = (planType: string): string => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic':
      return 'Perfect for beginners';
    case 'professional':
    case 'pro':
      return 'Most popular choice';
    case 'enterprise':
    case 'premium':
      return 'For serious traders';
    default:
      return 'Trading plan';
  }
};

const getDefaultFeaturesForPlan = (planType: string): string[] => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic':
      return [
        '5 API calls per day',
        'Basic analytics dashboard',
        'Email support',
        'Mobile app access',
        'Basic stock alerts',
      ];
    case 'professional':
    case 'pro':
      return [
        'Everything in Starter',
        '50 API calls per day',
        'Advanced analytics & charts',
        'Priority email support',
        'Real-time data streaming',
        'Portfolio tracking',
        'Custom alerts & notifications',
      ];
    case 'enterprise':
    case 'premium':
      return [
        'Everything in Professional',
        'Unlimited API calls',
        'Premium analytics suite',
        '24/7 phone & chat support',
        'AI-powered insights',
        'Advanced portfolio management',
        'Custom integrations',
        'Dedicated account manager',
      ];
    default:
      return ['Standard features included'];
  }
};

// Plans are now fully dynamic from API - no hardcoded fallback

const PAYMENT_METHODS = [
  {
    id: 'metamask',
    name: 'MetaMask (Instant)',
    icon: Zap,
    description: 'Pay directly with USDT/USDC via MetaMask',
  },
];

export default function OneClickPayment({
  className,
  preselectedPackage,
}: OneClickPaymentProps) {
  const [packages, setPackages] = useState<PaymentPackage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedPackage, setSelectedPackage] = useState<number | string | null>(null);
  const [currentStep, setCurrentStep] = useState<PaymentStep>('package');
  const [selectedPaymentMethod, setSelectedPaymentMethod] =
    useState('metamask');
  const [isProcessing, setIsProcessing] = useState(false);
  const [transactionHash, setTransactionHash] = useState<string | null>(null);

  // Load plans from API
  useEffect(() => {
    const loadPlans = async () => {
      try {
        setLoading(true);
        setError(null);
        const plans = await fetchPlans();
        setPackages(plans);

        // Set default selected package
        if (preselectedPackage) {
          const selectedPlan = plans.find(
            p =>
              p.id === preselectedPackage ||
              p.plan_type.toLowerCase() === preselectedPackage.toLowerCase() ||
              p.name.toLowerCase() === preselectedPackage.toLowerCase()
          );
          setSelectedPackage(selectedPlan?.id || plans[0]?.id || null);
        } else {
          // Default to first plan (Starter at $14.99)
          const defaultPlan = plans[0];
          setSelectedPackage(defaultPlan?.id || null);
        }
      } catch (err) {
        console.error('Error loading plans:', err);
        setError('Failed to load plans. Please try again.');
        // No fallback - show error state instead
        setPackages([]);
        setSelectedPackage(null);
      } finally {
        setLoading(false);
      }
    };

    loadPlans();
  }, [preselectedPackage]);

  const handlePayment = async () => {
    if (selectedPaymentMethod === 'metamask') {
      // MetaMask payment is handled by the MetaMaskPayment component
      return;
    }

    setIsProcessing(true);

    try {

      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000));

      setCurrentStep('confirmation');
    } catch (error) {
      console.error('Payment failed:', error);
      alert('Payment failed. Please try again.');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleMetaMaskSuccess = async (txHash: string) => {
    setTransactionHash(txHash);

    // Call backend to confirm payment and activate subscription
    if (selectedPkg) {
      try {
        const response = await fetch('/api/payments/confirm', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          credentials: 'include',
          body: JSON.stringify({
            plan_id: selectedPkg.original_plan_id, // Use original plan ID for backend
            transaction_hash: txHash,
            amount: selectedPkg.current_price,
            currency: selectedPkg.currency,
            network: 'localhost', // Use localhost for Anvil local development
          }),
        });

        const result = await response.json();

        if (result.success) {
          setCurrentStep('confirmation');
        } else {
          console.error('Failed to activate subscription:', result.message);
          alert(
            `Payment confirmed but subscription activation failed: ${result.message}`
          );
          setCurrentStep('confirmation'); // Still show confirmation since payment succeeded
        }
      } catch (error) {
        console.error('Error confirming payment:', error);
        alert(
          'Payment succeeded but there was an error activating your subscription. Please contact support.'
        );
        setCurrentStep('confirmation'); // Still show confirmation since payment succeeded
      }
    }
  };

  const handleMetaMaskError = (error: string) => {
    console.error('MetaMask payment error:', error);
  };

  const selectedPkg = packages.find(pkg => pkg.id === selectedPackage);

  // Validate selected package has valid price
  // Price of 0 is valid for free plans
  if (selectedPkg && (selectedPkg.current_price === undefined || isNaN(selectedPkg.current_price) || selectedPkg.current_price < 0)) {
    console.error('❌ CRITICAL ERROR: Selected package has invalid price!', {
      selectedPkg,
      price: selectedPkg.current_price
    });
  }

  // Loading state
  if (loading) {
    return (
      <div className={cn('mx-auto max-w-5xl p-4 sm:p-6', className)}>
        <div className="space-y-6 text-center">
          <div className="border-primary mx-auto h-16 w-16 animate-spin rounded-full border-4 border-t-transparent" />
          <h2 className="text-xl font-semibold">Loading plans...</h2>
          <p className="text-muted-foreground">
            Please wait while we fetch the latest pricing
          </p>
        </div>
      </div>
    );
  }

  // Error state
  if (error && packages.length === 0) {
    return (
      <div className={cn('mx-auto max-w-5xl p-4 sm:p-6', className)}>
        <div className="space-y-6 text-center">
          <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-100">
            <AlertCircle className="h-8 w-8 text-red-600" />
          </div>
          <div>
            <h2 className="text-xl font-semibold text-red-600">
              Failed to Load Plans
            </h2>
            <p className="text-muted-foreground mt-2">{error}</p>
          </div>
          <Button onClick={() => window.location.reload()}>Try Again</Button>
        </div>
      </div>
    );
  }

  const StepIndicator = () => (
    <div className="mb-6 flex items-center justify-center sm:mb-8">
      <div className="flex items-center space-x-2 sm:space-x-4">
        {(['package', 'payment', 'confirmation'] as PaymentStep[]).map(
          (step, index) => (
            <div key={step} className="flex items-center">
              <div
                className={cn(
                  'flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold transition-all sm:h-10 sm:w-10 sm:text-sm',
                  currentStep === step
                    ? 'bg-primary text-primary-foreground scale-110'
                    : index <
                      ['package', 'payment', 'confirmation'].indexOf(
                        currentStep
                      )
                      ? 'bg-green-500 text-white'
                      : 'bg-muted text-muted-foreground'
                )}
              >
                {index <
                  ['package', 'payment', 'confirmation'].indexOf(currentStep) ? (
                  <Check className="h-4 w-4" />
                ) : (
                  index + 1
                )}
              </div>
              {index < 2 && (
                <div
                  className={cn(
                    'h-0.5 w-6 transition-colors sm:w-12',
                    index <
                      ['package', 'payment', 'confirmation'].indexOf(
                        currentStep
                      )
                      ? 'bg-green-500'
                      : 'bg-muted'
                  )}
                />
              )}
            </div>
          )
        )}
      </div>
    </div>
  );

  if (currentStep === 'confirmation') {
    return (
      <div className={cn('mx-auto max-w-2xl p-4 sm:p-6', className)}>
        <div className="space-y-6 text-center">
          <div className="mx-auto flex h-20 w-20 items-center justify-center rounded-full bg-green-100 sm:h-24 sm:w-24 dark:bg-green-900">
            <CheckCircle2 className="h-10 w-10 text-green-600 sm:h-12 sm:w-12 dark:text-green-400" />
          </div>

          <div>
            <h2 className="mb-2 text-2xl font-bold sm:text-3xl">
              🎉 Payment Successful!
            </h2>
            <p className="text-muted-foreground">
              Welcome to the {selectedPkg?.name} plan. Your account has been
              upgraded!
            </p>
          </div>

          <Card className="text-left">
            <CardContent className="p-4 sm:p-6">
              <div className="mb-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className="text-2xl">{selectedPkg?.icon}</span>
                  <div>
                    <h3 className="font-semibold">{selectedPkg?.name} Plan</h3>
                    <p className="text-muted-foreground text-sm">
                      Monthly subscription
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-xl font-bold">
                    ${selectedPkg?.current_price}
                  </p>
                  <p className="text-muted-foreground text-xs">per month</p>
                </div>
              </div>

              <div className="border-t pt-4">
                <p className="text-muted-foreground mb-2 text-sm">
                  What's included:
                </p>
                <div className="grid grid-cols-1 gap-1 sm:grid-cols-2">
                  {selectedPkg?.features.slice(0, 4).map((feature, index) => (
                    <div key={index} className="flex items-center text-sm">
                      <Check className="mr-2 h-3 w-3 flex-shrink-0 text-green-500" />
                      {feature}
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="flex flex-col gap-3 sm:flex-row">
            <Button
              className="flex-1"
              onClick={() => (window.location.href = '/dashboard')}
            >
              <Smartphone className="mr-2 h-4 w-4" />
              Go to Dashboard
            </Button>
            <Button
              variant="outline"
              className="flex-1"
              onClick={() => setCurrentStep('package')}
            >
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to Plans
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={cn('mx-auto max-w-5xl p-4 sm:p-6', className)}>
      <StepIndicator />

      {currentStep === 'package' && (
        <>
          <div className="mb-6 text-center sm:mb-8">
            <h2 className="mb-2 text-2xl font-bold sm:mb-4 sm:text-3xl lg:text-4xl">
              💎 Choose for Unlock Your Plan
            </h2>
            <p className="text-muted-foreground mx-auto max-w-2xl text-sm sm:text-base">
              Unlock powerful analytics to the next level
            </p>
          </div>

          {/* Mobile: Swipeable Cards */}
          <div className="mb-6 block md:hidden">
            <div className="overflow-x-auto pb-4">
              <div className="flex gap-4 px-2">
                {packages.map(pkg => (
                  <div key={`${pkg.id}-${pkg.name}`} className="relative w-80 flex-shrink-0">
                    {/* Main Card */}
                    <div
                      className={cn(
                        'card-insight group relative flex h-full cursor-pointer flex-col overflow-visible',
                        selectedPackage === pkg.id
                          ? pkg.popular
                            ? 'border-orange-200/50 shadow-2xl ring-2 shadow-orange-500/25 ring-orange-200/60 dark:border-orange-400/30'
                            : 'border-blue-200/50 shadow-xl ring-2 shadow-blue-500/20 ring-blue-200/60 dark:border-blue-400/30'
                          : pkg.popular
                            ? 'border-orange-200/50 shadow-xl ring-2 shadow-orange-500/20 ring-orange-200/60 dark:border-orange-400/30'
                            : 'border-blue-200/50 shadow-lg ring-2 shadow-blue-500/15 ring-blue-200/60 dark:border-blue-400/30'
                      )}
                      onClick={() => setSelectedPackage(pkg.id)}
                    >
                      {/* Card Content */}
                      <div className="relative flex h-full flex-col px-6 pt-6 pb-6">
                        {/* Title Section */}
                        <div className="mb-4 flex h-[160px] flex-col items-center text-center">
                          <div
                            className={cn(
                              pkg.popular ? 'h-[80px]' : 'h-[40px]',
                              'mb-2 flex flex-col items-center justify-start'
                            )}
                          >
                            <h3
                              className={cn(
                                'text-xl leading-tight font-bold whitespace-nowrap uppercase',
                                pkg.popular
                                  ? 'bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent'
                                  : 'text-foreground'
                              )}
                            >
                              {pkg.name}
                            </h3>
                            {pkg.popular && (
                              <div className="mt-2">
                                <div className="rounded-full border-2 border-orange-300/50 bg-gradient-to-r from-orange-500 to-yellow-500 px-3 py-1 text-xs font-bold tracking-wide text-white shadow-lg shadow-orange-500/30">
                                  ⭐ MOST POPULAR ⭐
                                </div>
                              </div>
                            )}
                          </div>

                          {/* Price Display */}
                          <div
                            className={cn(
                              pkg.popular ? 'h-[58px]' : 'h-[78px]',
                              'flex flex-col items-center justify-center'
                            )}
                          >
                            <div className="flex flex-wrap items-baseline justify-center gap-3">
                              <span
                                className={cn(
                                  'text-4xl leading-none font-bold whitespace-nowrap',
                                  pkg.popular
                                    ? 'bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent'
                                    : 'insight-gradient-text'
                                )}
                              >
                                ${pkg.current_price}
                              </span>
                              {pkg.base_price > pkg.current_price && (
                                <span className="text-lg whitespace-nowrap text-gray-400 line-through decoration-2">
                                  ${pkg.base_price}
                                </span>
                              )}
                            </div>
                          </div>
                        </div>

                        {/* Features List */}
                        <div className="mb-8 flex min-h-[200px] flex-grow flex-col space-y-4">
                          {pkg.features.map((feature, idx) => (
                            <div
                              key={idx}
                              className="group/feature flex items-start"
                            >
                              <div
                                className={cn(
                                  'flex-shrink-0 rounded-full p-1.5',
                                  pkg.popular
                                    ? 'bg-orange-100 dark:bg-orange-900/30'
                                    : 'bg-insight-primary/20'
                                )}
                              >
                                <Check
                                  className={cn(
                                    'h-4 w-4',
                                    pkg.popular
                                      ? 'text-orange-600 dark:text-orange-400'
                                      : 'text-insight-primary'
                                  )}
                                />
                              </div>
                              <span className="text-muted-foreground ml-3 text-sm font-medium">
                                {feature}
                              </span>
                            </div>
                          ))}
                        </div>

                        {/* Action Button */}
                        <div className="mt-auto">
                          <button
                            className={cn(
                              'group relative w-full overflow-hidden rounded-xl py-4 text-base font-semibold',
                              pkg.popular
                                ? 'border-0 bg-gradient-to-r from-orange-400 via-amber-400 via-amber-500 via-yellow-400 to-orange-500 text-white shadow-xl shadow-orange-500/40'
                                : 'border-0 bg-gradient-to-r from-blue-400 via-blue-300 via-cyan-400 to-blue-400 text-white shadow-lg shadow-blue-400/30'
                            )}
                          >
                            <span className="relative flex items-center justify-center gap-2">
                              {selectedPackage === pkg.id
                                ? 'Selected'
                                : 'Select Plan'}
                              {pkg.popular && <Sparkles className="h-4 w-4" />}
                            </span>
                          </button>
                        </div>
                      </div>

                      {/* Decorative Elements */}
                      <div className="absolute -right-2 -bottom-2 h-12 w-12 rounded-full bg-gradient-to-br from-transparent via-transparent to-gray-100/30 blur-xl dark:to-gray-800/30" />
                      <div className="absolute -top-2 -left-2 h-8 w-8 rounded-full bg-gradient-to-br from-transparent via-transparent to-blue-100/20 blur-lg dark:to-blue-800/20" />
                    </div>
                  </div>
                ))}
              </div>
              <div className="mt-4 flex justify-center">
                <p className="text-muted-foreground text-xs">
                  👆 Swipe to explore all plans
                </p>
              </div>
            </div>
          </div>

          {/* Desktop: Grid */}
          <div className="mb-8 hidden gap-6 md:grid md:grid-cols-3">
            {packages.map(pkg => (
              <div key={`${pkg.id}-${pkg.name}`} className="relative">
                {/* Main Card */}
                <div
                  className={cn(
                    'card-insight group relative flex h-full cursor-pointer flex-col overflow-visible',
                    selectedPackage === pkg.id
                      ? pkg.popular
                        ? 'border-orange-200/50 shadow-2xl ring-2 shadow-orange-500/25 ring-orange-200/60 dark:border-orange-400/30'
                        : 'border-blue-200/50 shadow-xl ring-2 shadow-blue-500/20 ring-blue-200/60 dark:border-blue-400/30'
                      : pkg.popular
                        ? 'border-orange-200/50 shadow-xl ring-2 shadow-orange-500/20 ring-orange-200/60 dark:border-orange-400/30'
                        : 'border-blue-200/50 shadow-lg ring-2 shadow-blue-500/15 ring-blue-200/60 dark:border-blue-400/30'
                  )}
                  onClick={() => setSelectedPackage(pkg.id)}
                >
                  {/* Card Content */}
                  <div className="relative flex h-full flex-col px-6 pt-6 pb-6 sm:px-8 sm:pt-8 sm:pb-8">
                    {/* Title Section */}
                    <div className="mb-4 flex h-[160px] flex-col items-center text-center">
                      <div
                        className={cn(
                          pkg.popular ? 'h-[80px]' : 'h-[40px]',
                          'mb-2 flex flex-col items-center justify-start'
                        )}
                      >
                        <h3
                          className={cn(
                            'text-xl leading-tight font-bold whitespace-nowrap uppercase sm:text-2xl',
                            pkg.popular
                              ? 'bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent'
                              : 'text-foreground'
                          )}
                        >
                          {pkg.name}
                        </h3>
                        {pkg.popular && (
                          <div className="mt-2">
                            <div className="rounded-full border-2 border-orange-300/50 bg-gradient-to-r from-orange-500 to-yellow-500 px-3 py-1 text-xs font-bold tracking-wide text-white shadow-lg shadow-orange-500/30">
                              ⭐ MOST POPULAR ⭐
                            </div>
                          </div>
                        )}
                      </div>

                      {/* Price Display */}
                      <div
                        className={cn(
                          pkg.popular ? 'h-[58px]' : 'h-[78px]',
                          'flex flex-col items-center justify-center'
                        )}
                      >
                        <div className="flex flex-wrap items-baseline justify-center gap-3">
                          <span
                            className={cn(
                              'text-4xl leading-none font-bold whitespace-nowrap sm:text-5xl',
                              pkg.popular
                                ? 'bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent'
                                : 'insight-gradient-text'
                            )}
                          >
                            ${pkg.current_price}
                          </span>
                          {pkg.base_price > pkg.current_price && (
                            <span className="text-lg whitespace-nowrap text-gray-400 line-through decoration-2">
                              ${pkg.base_price}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Features List */}
                    <div className="mb-8 flex min-h-[200px] flex-grow flex-col space-y-4">
                      {pkg.features.map((feature, idx) => (
                        <div
                          key={idx}
                          className="group/feature flex items-start"
                        >
                          <div
                            className={cn(
                              'flex-shrink-0 rounded-full p-1.5',
                              pkg.popular
                                ? 'bg-orange-100 dark:bg-orange-900/30'
                                : 'bg-insight-primary/20'
                            )}
                          >
                            <Check
                              className={cn(
                                'h-4 w-4',
                                pkg.popular
                                  ? 'text-orange-600 dark:text-orange-400'
                                  : 'text-insight-primary'
                              )}
                            />
                          </div>
                          <span className="text-muted-foreground ml-3 text-sm font-medium sm:text-base">
                            {feature}
                          </span>
                        </div>
                      ))}
                    </div>

                    {/* Action Button */}
                    <div className="mt-auto">
                      <button
                        className={cn(
                          'group relative w-full overflow-hidden rounded-xl py-4 text-base font-semibold',
                          pkg.popular
                            ? 'border-0 bg-gradient-to-r from-orange-400 via-amber-400 via-amber-500 via-yellow-400 to-orange-500 text-white shadow-xl shadow-orange-500/40'
                            : 'border-0 bg-gradient-to-r from-blue-400 via-blue-300 via-cyan-400 to-blue-400 text-white shadow-lg shadow-blue-400/30'
                        )}
                      >
                        <span className="relative flex items-center justify-center gap-2">
                          {selectedPackage === pkg.id
                            ? 'Selected'
                            : 'Select Plan'}
                          {pkg.popular && <Sparkles className="h-4 w-4" />}
                        </span>
                      </button>
                    </div>
                  </div>

                  {/* Decorative Elements */}
                  <div className="absolute -right-2 -bottom-2 h-12 w-12 rounded-full bg-gradient-to-br from-transparent via-transparent to-gray-100/30 blur-xl dark:to-gray-800/30" />
                  <div className="absolute -top-2 -left-2 h-8 w-8 rounded-full bg-gradient-to-br from-transparent via-transparent to-blue-100/20 blur-lg dark:to-blue-800/20" />
                </div>
              </div>
            ))}
          </div>

          <div className="text-center">
            <Button
              onClick={() => setCurrentStep('payment')}
              size="lg"
              className="w-full px-8 py-3 text-lg font-semibold sm:w-auto"
            >
              Continue with {selectedPkg?.name} Plan
              <ChevronRight className="ml-2 h-5 w-5" />
            </Button>
          </div>
        </>
      )}

      {currentStep === 'payment' && selectedPkg && (
        <>
          <div className="mb-6 flex items-center gap-4">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setCurrentStep('package')}
              className="p-2"
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <h2 className="text-xl font-bold sm:text-2xl">Complete Payment</h2>
          </div>

          <div className="grid gap-6 sm:gap-8 lg:grid-cols-2">
            {/* Payment Form */}
            <div className="space-y-6">
              {/* Upgrade Credit Banner */}
              <UpgradeBanner
                newPlanId={typeof selectedPkg.original_plan_id === 'number'
                  ? selectedPkg.original_plan_id
                  : parseInt(String(selectedPkg.original_plan_id), 10)}
                className="mb-4"
              />

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Zap className="h-5 w-5" />
                    Payment Method
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {PAYMENT_METHODS.map(method => (
                    <div
                      key={method.id}
                      className={cn(
                        'cursor-pointer rounded-lg border p-4 transition-all',
                        selectedPaymentMethod === method.id
                          ? 'border-primary bg-primary/5'
                          : 'border-muted hover:border-primary/50'
                      )}
                      onClick={() => setSelectedPaymentMethod(method.id)}
                    >
                      <div className="flex items-center gap-3">
                        <method.icon className="h-5 w-5" />
                        <div className="flex-1">
                          <p className="font-medium">{method.name}</p>
                          <p className="text-muted-foreground text-sm">
                            {method.description}
                          </p>
                        </div>
                        {selectedPaymentMethod === method.id && (
                          <Check className="text-primary h-5 w-5" />
                        )}
                      </div>
                    </div>
                  ))}
                </CardContent>
              </Card>

              {/* MetaMask Payment Component */}
              {selectedPaymentMethod === 'metamask' && selectedPkg && (
                <MetaMaskPayment
                  planId={selectedPkg.original_plan_id}
                  planName={selectedPkg.name}
                  amount={selectedPkg.current_price}
                  currency={selectedPkg.currency}
                  onSuccess={handleMetaMaskSuccess}
                  onError={handleMetaMaskError}
                />
              )}

              {/* Security Info */}
              <Card className="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
                <CardContent className="p-4">
                  <div className="flex items-center gap-2 text-green-700 dark:text-green-300">
                    <Shield className="h-5 w-5" />
                    <span className="font-medium">Secure Payment</span>
                  </div>
                  <p className="mt-1 text-sm text-green-600 dark:text-green-400">
                    Your MetaMask transaction is processed directly on the
                    blockchain. We never have access to your private keys.
                  </p>
                </CardContent>
              </Card>
            </div>

            {/* Order Summary */}
            <div className="space-y-6">
              <Card className="sticky top-4">
                <CardHeader>
                  <CardTitle>Order Summary</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center gap-3">
                    <span className="text-2xl">{selectedPkg.icon}</span>
                    <div className="flex-1">
                      <h3 className="font-semibold">{selectedPkg.name} Plan</h3>
                      <p className="text-muted-foreground text-sm">
                        Monthly subscription
                      </p>
                    </div>
                  </div>

                  <div className="space-y-2 border-t pt-4">
                    <div className="flex justify-between">
                      <span>Subtotal</span>
                      <span>${selectedPkg.current_price}</span>
                    </div>
                    {selectedPkg.base_price > selectedPkg.current_price && (
                      <div className="flex justify-between text-green-600">
                        <span>Discount</span>
                        <span>
                          -${selectedPkg.base_price - selectedPkg.current_price}
                        </span>
                      </div>
                    )}
                    <div className="flex justify-between border-t pt-2 text-lg font-bold">
                      <span>Total</span>
                      <span>${selectedPkg.current_price}/month</span>
                    </div>
                  </div>

                  <Button
                    disabled={isProcessing}
                    className="w-full"
                    size="lg"
                    onClick={handlePayment}
                  >
                    {isProcessing ? (
                      <>
                        <div className="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
                        Processing...
                      </>
                    ) : (
                      <>
                        <Lock className="mr-2 h-4 w-4" />
                        Pay ${selectedPkg.current_price} Now
                      </>
                    )}
                  </Button>

                  <p className="text-muted-foreground text-center text-xs">
                    By continuing, you agree to our Terms of Service and Privacy
                    Policy
                  </p>
                </CardContent>
              </Card>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
