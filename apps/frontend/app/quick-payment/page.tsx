'use client';

import { Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import dynamic from 'next/dynamic';

// Dynamically import the payment component
const OneClickPayment = dynamic(
  () => import('@/components/features/payment/OneClickPayment'),
  { 
    ssr: false,
    loading: () => (
      <div className="flex justify-center items-center min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    )
  }
);

function PaymentContent() {
  const searchParams = useSearchParams();
  const packageType = searchParams.get('package');
  const amount = searchParams.get('amount');

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-gray-900 mb-4">
            Quick & Secure Payment
          </h1>
          <p className="text-lg text-gray-600 max-w-2xl mx-auto">
            Choose your plan and pay in seconds. All payments are secured with blockchain technology.
          </p>
        </div>

        <OneClickPayment 
          preselectedPackage={packageType || ''}
          preselectedAmount={amount || ''}
          className="mb-12"
        />

        {/* Trust indicators */}
        <div className="max-w-2xl mx-auto">
          <div className="bg-white rounded-lg p-6 shadow-sm">
            <div className="flex items-center justify-center space-x-8 text-sm text-gray-600">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span>SSL Encrypted</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span>Blockchain Secured</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span>Instant Activation</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function QuickPaymentPage() {
  return (
    <Suspense fallback={
      <div className="flex justify-center items-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      </div>
    }>
      <PaymentContent />
    </Suspense>
  );
}
