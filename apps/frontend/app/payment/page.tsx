'use client';

import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import { PaymentStatusSection } from '@/components/sections/payment/PaymentStatusSection';
import { PACKAGES } from '@/app/constants/packages';
import { useSearchParams } from 'next/navigation';

const OneClickPayment = dynamic(
  () => import('@/components/features/payment/OneClickPayment'),
  { ssr: false },
);

export default function PaymentPage() {
  const searchParams = useSearchParams();
  const selectedPackageId = searchParams.get('package') || '';

  return (
    <main className="min-h-screen bg-gradient-to-br from-pink-50 via-purple-50 to-orange-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 py-12 px-4 relative overflow-hidden">
      {/* Decorative background elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl animate-pulse-slow"></div>
        <div
          className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-orange-400/30 to-yellow-500/30 rounded-full blur-xl animate-pulse-slow"
          style={{ animationDelay: '2s' }}
        ></div>
        <div
          className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl animate-pulse-slow"
          style={{ animationDelay: '4s' }}
        ></div>
      </div>

      <div className="max-w-4xl mx-auto relative z-10">
        <div className="text-center mb-12">
          <div className="inline-block mb-6">
            <div className="bg-gradient-to-r from-pink-500 via-purple-500 to-orange-500 bg-clip-text text-transparent text-6xl font-black mb-2">
              💰
            </div>
          </div>
          <h1 className="text-4xl lg:text-5xl font-black bg-gradient-to-r from-pink-600 via-purple-600 to-orange-600 bg-clip-text text-transparent mb-6">
            Quick & Secure Payment
          </h1>
          <p className="text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto leading-relaxed">
            Choose your plan and pay in seconds. All payments are secured with
            blockchain technology.
            <span className="inline-block ml-2 animate-bounce">🚀</span>
          </p>
        </div>

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

        <div className="mb-12">
          <PaymentStatusSection className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-pink-200/50 dark:border-pink-700/50" />
        </div>
      </div>
    </main>
  );
}
