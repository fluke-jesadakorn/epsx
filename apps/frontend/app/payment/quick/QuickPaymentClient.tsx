'use client';

import { Suspense } from 'react';
import { Pay } from '@/components/pay';

interface QuickPaymentClientProps {
  pkg: string;
  amt: string;
}

export function QuickPaymentClient({ pkg, amt }: QuickPaymentClientProps) {
  return (
    <Suspense
      fallback={
        <div className="flex justify-center items-center min-h-screen">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      }
    >
      <div className="min-h-screen bg-gray-50 py-12 px-4">
        <Pay pkg={pkg} amt={amt} />
      </div>
    </Suspense>
  );
}