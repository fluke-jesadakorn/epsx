'use client';

import { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuth } from '@/context/auth-context';
import { createPaymentService } from '@/services/payment.service';

import PackageSelection from './PackageSelection';
import AssetSelection from './AssetSelection';
import PaymentForm from './PaymentForm';

interface PaymentFlowProps {
  className?: string;
}

type PaymentStep =
  | 'package'
  | 'asset'
  | 'details'
  | 'processing'
  | 'confirmation';

export default function PaymentFlow({ className }: PaymentFlowProps) {
  const { user, loading } = useAuth();
  const router = useRouter();
  const searchParams = useSearchParams();
  const paymentService = createPaymentService();

  // Get URL parameters
  const prefilledAmount = searchParams.get('amount');
  const prefilledCurrency = searchParams.get('currency');
  const packageType = searchParams.get('packageType');

  // Payment flow state
  const [currentStep, setCurrentStep] = useState<PaymentStep>(
    packageType ? 'asset' : 'package',
  );
  const [selectedPackage, setSelectedPackage] = useState(packageType || '');
  const [selectedAsset, setSelectedAsset] = useState(
    prefilledCurrency || 'USDT_TRC20',
  );
  const [amount, setAmount] = useState(prefilledAmount || '');
  const [paymentStatus, setPaymentStatus] = useState<
    'pending' | 'processing' | 'completed' | 'failed'
  >('pending');

  useEffect(() => {
    if (!loading && !user) {
      router.push('/login');
    }
  }, [loading, user, router]);

  if (loading) {
    return (
      <div className="flex justify-center items-center min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (!user) {
    return null;
  }

  const handlePackageSelect = (packageType: string, amount: string) => {
    setSelectedPackage(packageType);
    setAmount(amount);
    setCurrentStep('asset');
  };

  const handleAssetSelect = (asset: string) => {
    setSelectedAsset(asset);
    setCurrentStep('details');
  };

  const handlePaymentSubmit = async () => {
    setCurrentStep('processing');
    try {
      const transactionId = await paymentService.recordPayment(
        Number(amount),
        selectedAsset,
        `${selectedPackage} package purchase`,
      );

      if (transactionId) {
        setPaymentStatus('completed');
        setCurrentStep('confirmation');
      } else {
        setPaymentStatus('failed');
      }
    } catch (error) {
      console.error('Payment failed:', error);
      setPaymentStatus('failed');
    }
  };

  return (
    <div className={className}>
      <div className="max-w-3xl mx-auto">
        {/* Step indicators */}
        <div className="mb-8">
          <div className="flex justify-between items-center">
            {['package', 'asset', 'details', 'confirmation'].map((step) => (
              <div
                key={step}
                className={`flex-1 h-2 rounded ${
                  currentStep === step
                    ? 'bg-primary'
                    : ['package', 'asset'].includes(step) &&
                        currentStep === 'details'
                      ? 'bg-primary'
                      : currentStep === 'confirmation'
                        ? 'bg-primary'
                        : 'bg-muted'
                }`}
              />
            ))}
          </div>
          <div className="flex justify-between mt-2 text-sm text-muted-foreground">
            <span>Select Package</span>
            <span>Select Asset</span>
            <span>Payment Details</span>
            <span>Confirmation</span>
          </div>
        </div>

        {/* Step content */}
        <div className="mt-8">
          {currentStep === 'package' && (
            <div className="grid gap-6">
              <h2 className="text-2xl font-bold">Select a Package</h2>
              <PackageSelection
                onSelect={handlePackageSelect}
                selectedPackage={selectedPackage}
              />
            </div>
          )}

          {currentStep === 'asset' && (
            <div className="grid gap-6">
              <h2 className="text-2xl font-bold">Select Payment Asset</h2>
              <AssetSelection
                onSelect={handleAssetSelect}
                selectedAsset={selectedAsset}
              />
            </div>
          )}

          {currentStep === 'details' && (
            <div className="grid gap-6">
              <h2 className="text-2xl font-bold">Payment Details</h2>
              <PaymentForm
                selectedAsset={selectedAsset}
                amount={amount}
                packageType={selectedPackage}
                onSubmit={handlePaymentSubmit}
              />
            </div>
          )}

          {currentStep === 'processing' && (
            <div className="text-center py-12">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
              <p className="text-lg">Processing your payment...</p>
            </div>
          )}

          {currentStep === 'confirmation' && (
            <div className="text-center py-12">
              {paymentStatus === 'completed' ? (
                <>
                  <h2 className="text-2xl font-bold text-green-600 mb-4">
                    Payment Successful!
                  </h2>
                  <p className="text-lg text-muted-foreground mb-8">
                    Thank you for your purchase. You can now access your package
                    features.
                  </p>
                  <button
                    onClick={() => router.push('/dashboard')}
                    className="bg-primary text-primary-foreground px-6 py-2 rounded-md hover:opacity-90"
                  >
                    Go to Dashboard
                  </button>
                </>
              ) : (
                <>
                  <h2 className="text-2xl font-bold text-destructive mb-4">
                    Payment Failed
                  </h2>
                  <p className="text-lg text-muted-foreground mb-8">
                    There was an error processing your payment. Please try
                    again.
                  </p>
                  <button
                    onClick={() => setCurrentStep('details')}
                    className="bg-primary text-primary-foreground px-6 py-2 rounded-md hover:opacity-90"
                  >
                    Try Again
                  </button>
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
