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
            <div className="relative">
              <div className="w-16 h-16 border-4 border-pink-200 dark:border-pink-800 rounded-full animate-spin"></div>
              <div className="absolute top-0 left-0 w-16 h-16 border-4 border-transparent border-t-pink-500 rounded-full animate-spin"></div>
            </div>
          </div>
        }
      >
        <OneClickPayment className="mb-12" preselectedPackage={selectedPackageId} />
      </Suspense>
    </>
  );
}