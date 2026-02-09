'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '@/lib/auth';
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
  Wallet,
  Zap,
} from 'lucide-react';
import { useAccount } from 'wagmi';
import MetaMaskPayment from './meta-mask-payment';
import { UpgradeBanner } from './upgrade-banner';
import type { OneClickPaymentProps } from './types';
import { StepIndicator } from './ui/step-indicator';
import { PackageCard } from './ui/package-card';
import { usePaymentState } from './hooks/use-payment-state';
import { usePaymentProcessing } from './hooks/use-payment-processing';
import { PAYMENT_METHODS } from './constants';

export default function OneClickPayment({
  className,
  preselectedPackage,
}: OneClickPaymentProps) {
  const { chain } = useAccount();
  const { user, isLoading: isAuthLoading, refreshUser } = useAuth();
  const isAuthenticated = Boolean(user);

  const {
    packages,
    loading,
    error,
    selectedPackage,
    setSelectedPackage,
    currentStep,
    setCurrentStep,
    selectedPaymentMethod,
    setSelectedPaymentMethod,
    selectedPkg,
  } = usePaymentState(preselectedPackage);

  const {
    isProcessing,
    handlePayment,
    handleMetaMaskSuccess,
    handleMetaMaskError,
  } = usePaymentProcessing();

  if (!isAuthLoading && !isAuthenticated) {
    return <NotAuthenticatedState className={className} />;
  }

  if (loading) {
    return <LoadingState className={className} />;
  }

  if (error && packages.length === 0) {
    return <ErrorState error={error} className={className} />;
  }

  if (currentStep === 'confirmation') {
    return <ConfirmationState selectedPkg={selectedPkg} setCurrentStep={setCurrentStep} className={className} />;
  }

  return (
    <div className={cn('mx-auto max-w-5xl p-4 sm:p-6', className)}>
      <StepIndicator currentStep={currentStep} />

      {currentStep === 'package' && (
        <>
          <PackageSelectionHeader />
          <MobilePackageCards packages={packages} selectedPackage={selectedPackage} setSelectedPackage={setSelectedPackage} />
          <DesktopPackageCards packages={packages} selectedPackage={selectedPackage} setSelectedPackage={setSelectedPackage} />
          <ContinueButton selectedPkg={selectedPkg} setCurrentStep={setCurrentStep} />
        </>
      )}

      {currentStep === 'payment' && selectedPkg && (
        <PaymentStep
          selectedPkg={selectedPkg}
          selectedPaymentMethod={selectedPaymentMethod}
          setSelectedPaymentMethod={setSelectedPaymentMethod}
          setCurrentStep={setCurrentStep}
          isProcessing={isProcessing}
          handlePayment={handlePayment}
          handleMetaMaskSuccess={handleMetaMaskSuccess}
          handleMetaMaskError={handleMetaMaskError}
        />
      )}
    </div>
  );
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function NotAuthenticatedState({ className }: { className?: string }) {
  return (
    <div className={cn('mx-auto max-w-2xl p-4 sm:p-6', className)}>
      <div className="space-y-6 text-center">
        <div className="mx-auto flex h-20 w-20 items-center justify-center rounded-full bg-amber-100 sm:h-24 sm:w-24 dark:bg-amber-900/30">
          <Wallet className="h-10 w-10 text-amber-600 sm:h-12 sm:w-12 dark:text-amber-400" />
        </div>

        <div>
          <h2 className="mb-2 text-2xl font-bold sm:text-3xl">
            🔐 Sign In Required
          </h2>
          <p className="text-muted-foreground max-w-md mx-auto">
            To complete your purchase, please connect and sign in with your wallet. This verifies your identity and ensures your subscription is properly activated.
          </p>
        </div>

        <Card className="text-left max-w-md mx-auto">
          <CardContent className="p-6">
            <div className="space-y-4">
              <div className="flex items-start gap-3">
                <Shield className="h-5 w-5 text-green-500 mt-0.5" />
                <div>
                  <p className="font-medium">Secure Authentication</p>
                  <p className="text-sm text-muted-foreground">Sign-In with Ethereum (SIWE) ensures only you can access your account</p>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <Lock className="h-5 w-5 text-blue-500 mt-0.5" />
                <div>
                  <p className="font-medium">One-Time Sign In</p>
                  <p className="text-sm text-muted-foreground">You'll stay signed in until you disconnect your wallet</p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="flex justify-center">
          <Button
            className="px-8 py-3 text-lg"
            onClick={() => {
              window.scrollTo({ top: 0, behavior: 'smooth' });
              const connectBtn = document.querySelector('[data-testid="rk-connect-button"]') as HTMLButtonElement;
              if (connectBtn) {
                connectBtn.click();
              } else {
                const walletBtn = document.querySelector('button[aria-label*="wallet"], button[aria-label*="connect"]') as HTMLButtonElement;
                if (walletBtn) {walletBtn.click();}
              }
            }}
          >
            <Wallet className="mr-2 h-5 w-5" />
            Connect & Sign In
          </Button>
        </div>

        <p className="text-sm text-muted-foreground">
          Don't have a wallet? <a href="https://metamask.io" target="_blank" rel="noopener noreferrer" className="underline hover:text-primary">Get MetaMask</a>
        </p>
      </div>
    </div>
  );
}

function LoadingState({ className }: { className?: string }) {
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

function ErrorState({ error, className }: { error: string; className?: string }) {
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

interface ConfirmationStateProps {
  selectedPkg: any;
  setCurrentStep: (step: 'package') => void;
  className?: string;
}

function ConfirmationState({ selectedPkg, setCurrentStep, className }: ConfirmationStateProps) {
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
                {selectedPkg?.features.slice(0, 4).map((feature: string, index: number) => (
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

function PackageSelectionHeader() {
  return (
    <div className="mb-6 text-center sm:mb-8">
      <h2 className="mb-2 text-2xl font-bold sm:mb-4 sm:text-3xl lg:text-4xl">
        💎 Choose for Unlock Your Plan
      </h2>
      <p className="text-muted-foreground mx-auto max-w-2xl text-sm sm:text-base">
        Unlock powerful analytics to the next level
      </p>
    </div>
  );
}

function MobilePackageCards({ packages, selectedPackage, setSelectedPackage }: any) {
  return (
    <div className="mb-6 block md:hidden">
      <div className="overflow-x-auto pb-4">
        <div className="flex gap-4 px-2">
          {packages.map((pkg: any) => (
            <PackageCard
              key={`${pkg.id} - ${pkg.name}`}
              pkg={pkg}
              isSelected={selectedPackage === pkg.id}
              onSelect={() => setSelectedPackage(pkg.id)}
              isMobile
            />
          ))}
        </div>
        <div className="mt-4 flex justify-center">
          <p className="text-muted-foreground text-xs">
            👆 Swipe to explore all plans
          </p>
        </div>
      </div>
    </div>
  );
}

function DesktopPackageCards({ packages, selectedPackage, setSelectedPackage }: any) {
  return (
    <div className="mb-8 hidden gap-6 md:grid md:grid-cols-3">
      {packages.map((pkg: any) => (
        <PackageCard
          key={`${pkg.id} - ${pkg.name}`}
          pkg={pkg}
          isSelected={selectedPackage === pkg.id}
          onSelect={() => setSelectedPackage(pkg.id)}
        />
      ))}
    </div>
  );
}

function ContinueButton({ selectedPkg, setCurrentStep }: { selectedPkg: any; setCurrentStep: (step: 'payment') => void }) {
  return (
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
  );
}

interface PaymentStepProps {
  selectedPkg: any;
  selectedPaymentMethod: string;
  setSelectedPaymentMethod: (method: string) => void;
  setCurrentStep: (step: 'package') => void;
  isProcessing: boolean;
  handlePayment: () => void;
  handleMetaMaskSuccess: (txHash: string) => void;
  handleMetaMaskError: (error: string) => void;
}

function PaymentStep({
  selectedPkg,
  selectedPaymentMethod,
  setSelectedPaymentMethod,
  setCurrentStep,
  isProcessing,
  handlePayment,
  handleMetaMaskSuccess,
  handleMetaMaskError,
}: PaymentStepProps) {
  return (
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
        <div className="space-y-6">
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
              {PAYMENT_METHODS.map((method) => (
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

          {selectedPaymentMethod === 'metamask' && selectedPkg && (
            <MetaMaskPayment
              planId={selectedPkg.original_plan_id}
              planName={selectedPkg.name}
              amount={selectedPkg.current_price}
              currency={selectedPkg.currency}
              onSuccess={(txHash) => handleMetaMaskSuccess(txHash)}
              onError={handleMetaMaskError}
            />
          )}

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

        <OrderSummary selectedPkg={selectedPkg} selectedPaymentMethod={selectedPaymentMethod} isProcessing={isProcessing} handlePayment={handlePayment} />
      </div>
    </>
  );
}

interface OrderSummaryProps {
  selectedPkg: any;
  selectedPaymentMethod: string;
  isProcessing: boolean;
  handlePayment: () => void;
}

function OrderSummary({ selectedPkg, selectedPaymentMethod, isProcessing, handlePayment }: OrderSummaryProps) {
  return (
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
            disabled={isProcessing || selectedPaymentMethod === 'metamask'}
            className="w-full"
            size="lg"
            onClick={handlePayment}
          >
            {isProcessing ? (
              <>
                <div className="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
                Processing...
              </>
            ) : selectedPaymentMethod === 'metamask' ? (
              <>
                <Wallet className="mr-2 h-4 w-4" />
                Use MetaMask Button Below
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
  );
}
