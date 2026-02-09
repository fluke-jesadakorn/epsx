'use client';

import dynamic from 'next/dynamic';
import { Suspense, useEffect } from 'react';

// Dynamically import payment components to avoid SSR issues with wallet
const OneClickPayment = dynamic(
  () => import('@/components/features/payment/one-click-payment'),
  { ssr: false },
);

const DynamicPaymentWidget = dynamic(
  () => import('@/components/features/payment/dynamic-payment-widget').then(mod => ({ default: mod.DynamicPaymentWidget })),
  { ssr: false },
);

interface PaymentPageClientProps {
  selectedPackageId: string;
  // V2 Dynamic Payment Context
  context?: {
    planId: string | null;
    planId: string | null;
    linkSlug: string | null;
  };
}

export function PaymentPageClient({ selectedPackageId, context }: PaymentPageClientProps) {
  // Scroll to top when component mounts
  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  // Determine if this is a dynamic link payment
  const isDynamicPayment = Boolean(context?.planId ?? context?.planId ?? context?.linkSlug);

  // Loading fallback component
  const LoadingFallback = () => (
    <div className="flex justify-center items-center min-h-[400px]">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600 mx-auto mb-4" />
        <div className="text-gray-600 dark:text-gray-400 text-lg">Loading payment options...</div>
      </div>
    </div>
  );

  // Use DynamicPaymentWidget for V2 context-based payments
  if (isDynamicPayment && context) {
    return (
      <Suspense fallback={<LoadingFallback />}>
        <DynamicPaymentWidget
          context={context}
          className="mb-12"
          onPaymentSuccess={(txHash) => {
             
            // Could redirect or show success state
          }}
          onPaymentError={(error) => {
          }}
        />
      </Suspense>
    );
  }

  // Fall back to OneClickPayment for legacy package selection
  return (
    <Suspense fallback={<LoadingFallback />}>
      <OneClickPayment className="mb-12" preselectedPackage={selectedPackageId} />
    </Suspense>
  );
}