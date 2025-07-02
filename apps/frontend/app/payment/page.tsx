'use client';

import dynamic from 'next/dynamic';
import { Suspense } from 'react';

// Dynamically import PaymentFlow to ensure proper client-side rendering
const PaymentFlow = dynamic(
  () => import('@/components/features/payment/PaymentFlow'),
  { ssr: false }
);

export default function PaymentPage() {
  return (
    <main className="container mx-auto px-4 py-8">
      <div className="mb-8 text-center">
        <h1 className="text-3xl font-bold">Payment</h1>
        <p className="text-muted-foreground mt-2">
          Select your package and complete the payment process
        </p>
      </div>

      <Suspense fallback={
        <div className="flex justify-center items-center min-h-[400px]">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      }>
        <PaymentFlow />
      </Suspense>
    </main>
  );
}
