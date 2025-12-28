'use client';

import dynamic from 'next/dynamic';
import { Suspense } from 'react';

const OneClickPayment = dynamic(
  () => import('@/components/features/payment/OneClickPayment'),
  { ssr: false },
);

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
        <OneClickPayment className="mb-12" preselectedPackage={selectedPackageId} />
      </Suspense>
    </>
  );
}