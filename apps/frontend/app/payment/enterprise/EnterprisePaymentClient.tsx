'use client';

import dynamic from 'next/dynamic';
import { Suspense } from 'react';
import { PaymentStatusSection } from '@/components/sections/payment/PaymentStatusSection';
import { SelectPackageSection } from '@/components/sections/payment/SelectPackageSection';
import { Card, CardHeader, CardContent } from '@/components/ui';
import { withPaymentAuth } from '@/components/sections/payment/withPaymentAuth';

// Dynamically import a payment component for enterprise (placeholder for now)
const EnterprisePayment = dynamic(
  () => import('@/components/features/payment/OneClickPayment'),
  { ssr: false }
);

// Wrap EnterprisePayment with authentication check
const AuthenticatedEnterprisePayment = withPaymentAuth(EnterprisePayment);

export function EnterprisePaymentClient() {
  return (
    <main className="min-h-screen bg-gradient-to-br from-pink-50 via-purple-50 to-orange-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 py-12 px-4 relative overflow-hidden">
      {/* Decorative background elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl animate-pulse-slow"></div>
        <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-orange-400/30 to-yellow-500/30 rounded-full blur-xl animate-pulse-slow" style={{animationDelay: '2s'}}></div>
        <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl animate-pulse-slow" style={{animationDelay: '4s'}}></div>
      </div>

      <div className="max-w-4xl mx-auto relative z-10">
        <div className="text-center mb-12">
          <div className="inline-block mb-6">
            <div className="bg-gradient-to-r from-pink-500 via-purple-500 to-orange-500 bg-clip-text text-transparent text-6xl font-black mb-2">
              💰
            </div>
          </div>
          <h1 className="text-4xl lg:text-5xl font-black bg-gradient-to-r from-pink-600 via-purple-600 to-orange-600 bg-clip-text text-transparent mb-6">
            Enterprise Payment Portal
          </h1>
          <p className="text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto leading-relaxed">
            Exclusive payment options for enterprise clients. All payments are secured with blockchain technology.
            <span className="inline-block ml-2 animate-bounce">🚀</span>
          </p>
        </div>

        <Suspense fallback={
          <div className="flex justify-center items-center min-h-[400px]">
            <div className="relative">
              <div className="w-16 h-16 border-4 border-pink-200 dark:border-pink-800 rounded-full animate-spin"></div>
              <div className="absolute top-0 left-0 w-16 h-16 border-4 border-transparent border-t-pink-500 rounded-full animate-spin"></div>
            </div>
          </div>
        }>
          <AuthenticatedEnterprisePayment className="mb-12" />
        </Suspense>

        {/* Payment Status Section */}
        <div className="mb-12">
          <PaymentStatusSection className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-pink-200/50 dark:border-pink-700/50" />
        </div>

        {/* Package Selection */}
        <Card className="transition-shadow hover:shadow-lg border bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-pink-200/50 dark:border-pink-700/50 mb-12">
          <CardHeader className="flex flex-row items-center gap-2 pb-2">
            <span className="bg-gradient-to-r from-pink-500/10 to-purple-500/10 rounded-full p-2">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-6 w-6 text-pink-600 dark:text-pink-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
                />
              </svg>
            </span>
            <h2 className="text-xl font-semibold bg-gradient-to-r from-pink-600 via-purple-600 to-orange-600 bg-clip-text text-transparent">Select Enterprise Package</h2>
          </CardHeader>
          <CardContent className="p-0">
            <SelectPackageSection showTitle={false} />
          </CardContent>
        </Card>
      </div>
    </main>
  );
}