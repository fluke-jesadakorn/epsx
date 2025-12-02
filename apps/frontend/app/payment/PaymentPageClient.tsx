'use client';

import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import { withPaymentAuth } from '@/components/sections/payment/withPaymentAuth';

const OneClickPayment = dynamic(
  () => import('@/components/features/payment/OneClickPayment'),
  { ssr: false },
);

// Wrap OneClickPayment with authentication check
const AuthenticatedOneClickPayment = withPaymentAuth(OneClickPayment);

interface PaymentPageClientProps {
  selectedPackageId: string;
}

export function PaymentPageClient({ selectedPackageId }: PaymentPageClientProps) {
  return (
    <>
      {/* OneClickPayment handles package selection UI */}
      <Suspense
        fallback={
          <div className="flex justify-center items-center min-h-[400px]">
            <div className="text-center">
              <div className="text-pink-500 text-lg">Loading payment options...</div>
            </div>
          </div>
        }
      >
        <AuthenticatedOneClickPayment className="mb-12" preselectedPackage={selectedPackageId} />
      </Suspense>
    </>
  );
}