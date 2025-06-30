'use client';

import { Suspense } from 'react';
import { PaymentForm } from '@/components/features/payment/PaymentForm';
import { PaymentStatus } from '@/components/features/payment/PaymentStatus';
import { Card, CardHeader, CardContent } from '@/components/ui/card';

export default function PaymentSettingsPage() {
  const PaymentSkeleton = () => (
    <div className="max-w-2xl mx-auto p-6">
      <div className="flex flex-col gap-6">
        {/* Payment Settings Card Skeleton */}
        <div className="border rounded-lg shadow-sm">
          <div className="p-4 border-b">
            <div className="h-6 bg-gray-200 rounded-md animate-pulse w-1/4 dark:bg-gray-700"></div>
          </div>
          <div className="p-4 space-y-4">
            <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse w-2/3 dark:bg-gray-700"></div>
            <div className="h-10 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-10 bg-gray-200 rounded-md animate-pulse w-1/3 dark:bg-gray-700"></div>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <Suspense fallback={<PaymentSkeleton />}>
      <div className="max-w-4xl mx-auto p-6 md:p-6 sm:p-4">
        {/* Page Header */}
        <div className="mb-10 text-center">
          <h1 className="text-3xl md:text-4xl font-bold tracking-tight text-primary mb-2 flex items-center justify-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-8 w-8 text-primary"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-label="Payment Icon"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            Payment Settings
          </h1>
          <p className="text-muted-foreground text-sm md:text-base max-w-xl mx-auto">
            Manage your payment details and view transaction status all in one place.
          </p>
        </div>
        <div className="flex flex-col gap-10 md:gap-6">
          {/* Payment Details Card */}
          <Card className="transition-shadow hover:shadow-lg border-primary border-2 bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-pink-500/5 dark:bg-gradient-to-r dark:from-blue-900/10 dark:via-purple-900/10 dark:to-pink-900/10">
            <CardHeader>
              <div className="flex items-center gap-2">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-6 w-6 md:h-5 md:w-5 text-primary"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  aria-label="Payment Details Icon"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                <h2 className="text-xl md:text-lg font-bold text-primary">
                  Payment Details
                </h2>
                <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary text-white text-sm font-bold">1</span>
              </div>
              <p className="text-sm text-muted-foreground mt-1">
                Enter your payment information to proceed with transactions.
              </p>
            </CardHeader>
            <CardContent>
              <div className="p-6 sm:p-4 bg-primary/10 dark:bg-primary/20 rounded-lg">
                <PaymentForm />
              </div>
            </CardContent>
          </Card>
          {/* Payment Status Card */}
          <Card className="transition-shadow hover:shadow-lg border-primary border-2 bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-pink-500/5 dark:bg-gradient-to-r dark:from-blue-900/10 dark:via-purple-900/10 dark:to-pink-900/10">
            <CardHeader>
              <div className="flex items-center gap-2">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-6 w-6 md:h-5 md:w-5 text-primary"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  aria-label="Payment Status Icon"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 12h6m-3-3v6m9 0a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                <h2 className="text-xl md:text-lg font-bold text-primary">
                  Payment Status
                </h2>
                <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary text-white text-sm font-bold">2</span>
              </div>
              <p className="text-sm text-muted-foreground mt-1">
                Check the status of your recent transactions.
              </p>
            </CardHeader>
            <CardContent>
              <div className="p-6 sm:p-4 bg-primary/10 dark:bg-primary/20 rounded-lg">
                <PaymentStatus />
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </Suspense>
  );
}
