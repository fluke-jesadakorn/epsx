import { GlobalAuthGuard } from '@/components/auth/GlobalAuthGuard';
import { PaymentStatusServer } from '@/components/sections/payment/PaymentStatusServer';
import { getCurrentUser } from '@/lib/server-actions';
import { getDebugSessionInfo } from '@/lib/server-actions-user';
import { Suspense } from 'react';
import { PaymentPageClient } from './PaymentPageClient';

export const dynamic = 'force-dynamic';

/**
 * Payment Page Props
 * Supports multiple URL patterns for V2 dynamic payments:
 * - /payment?plan=uuid       → Plan payment (subscription)
 * - /payment?group=uuid      → Group payment (permission access)
 * - /payment?link=slug       → Dynamic payment link
 * - /payment?package=id      → Legacy package selection
 */
interface PaymentPageProps {
  searchParams: Promise<{
    package?: string;
    plan?: string;
    group?: string;
    link?: string;
  }>;
}

export default async function PaymentPage({ searchParams }: PaymentPageProps) {
  const user = await getCurrentUser();
  const debugInfo = !user ? await getDebugSessionInfo() : null;

  // Extract payment context from search params
  const resolvedSearchParams = await searchParams;
  const selectedPackageId = resolvedSearchParams.package || resolvedSearchParams.plan || '';

  // V2 Dynamic Payment Context
  const paymentContext = {
    planId: resolvedSearchParams.plan || null,
    groupId: resolvedSearchParams.group || null,
    linkSlug: resolvedSearchParams.link || null,
  };

  // Determine if this is a dynamic link payment
  const isDynamicPayment = !!(paymentContext.planId || paymentContext.groupId || paymentContext.linkSlug);

  // Show GlobalAuthGuard modal for unauthenticated users (same pattern as Developer page)
  if (!user) {
    return (
      <main className="min-h-screen bg-gradient-to-br from-pink-50 via-purple-50 to-orange-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 flex items-center justify-center relative overflow-hidden">
        {/* Decorative background elements */}
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl"></div>
          <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-orange-400/30 to-yellow-500/30 rounded-full blur-xl"></div>
          <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl"></div>
        </div>
        <div className="container mx-auto p-6 relative z-10">
          <GlobalAuthGuard title="Payment Portal" debugInfo={debugInfo} />
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-gradient-to-br from-pink-50 via-purple-50 to-orange-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 py-12 px-4 relative overflow-hidden">
      {/* Decorative background elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-orange-400/30 to-yellow-500/30 rounded-full blur-xl"></div>
        <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl"></div>
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
            <span className="inline-block ml-2">🚀</span>
          </p>
        </div>

        {/* Payment options - now guaranteed to have authenticated user */}
        <PaymentPageClient
          selectedPackageId={selectedPackageId}
          context={isDynamicPayment ? paymentContext : undefined}
        />

        {/* Payment status - backend will handle permissions */}
        <div className="mb-12">
          <Suspense
            fallback={
              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-pink-200/50 dark:border-pink-700/50">
                <div>
                  <div className="h-6 bg-gray-200 dark:bg-gray-700 rounded w-1/3 mb-4"></div>
                  <div className="space-y-3">
                    <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded"></div>
                    <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-5/6"></div>
                  </div>
                </div>
              </div>
            }
          >
            <PaymentStatusServer className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-pink-200/50 dark:border-pink-700/50" />
          </Suspense>
        </div>
      </div>
    </main>
  );
}

