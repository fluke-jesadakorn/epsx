'use client';

import { Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import { Pay } from '@/components/pay';

function PaymentContent() {
  const searchParams = useSearchParams();
  const pkg = searchParams.get('package');
  const amt = searchParams.get('amount');

  return (
    <div className="min-h-screen bg-gray-50 py-12 px-4">
      <Pay pkg={pkg || ''} amt={amt || ''} />
    </div>
  );
}

export default function QuickPaymentPage() {
  return (
    <Suspense
      fallback={
        <div className="flex justify-center items-center min-h-screen">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      }
    >
      <PaymentContent />
    </Suspense>
  );
}
