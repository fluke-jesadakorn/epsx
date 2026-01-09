import { GlobalAuthGuard } from '@/components/auth/GlobalAuthGuard';
import { getCurrentUser } from '@/lib/server-actions';
import { getDebugSessionInfo } from '@/lib/server-actions-user';
import { redirect } from 'next/navigation';
import { PaymentClient } from './PaymentClient';

export const dynamic = 'force-dynamic';

/**
 * Payment Page
 * 
 * Unified payment page supporting multiple URL patterns:
 * - /payment              → Show all plans
 * - /payment?plan=uuid    → Redirect to /payment/plan/[id]
 * - /payment?group=uuid   → Redirect to /payment/group/[id]
 * - /payment?link=slug    → Redirect to /payment/link/[slug]
 */
interface PaymentPageProps {
  searchParams: Promise<{
    plan?: string;
    group?: string;
    link?: string;
    permission?: string;
  }>;
}

export default async function PaymentPage({ searchParams }: PaymentPageProps) {
  const user = await getCurrentUser();
  const debugInfo = !user ? await getDebugSessionInfo() : null;
  const resolvedSearchParams = await searchParams;

  // Redirect query string patterns to new dynamic routes
  if (resolvedSearchParams.plan) {
    redirect(`/payment/plan/${resolvedSearchParams.plan}`);
  }
  if (resolvedSearchParams.group) {
    redirect(`/payment/group/${resolvedSearchParams.group}`);
  }
  if (resolvedSearchParams.permission) {
    redirect(`/payment/permission/${resolvedSearchParams.permission}`);
  }
  if (resolvedSearchParams.link) {
    redirect(`/payment/link/${resolvedSearchParams.link}`);
  }

  // Show auth guard for unauthenticated users
  if (!user) {
    return (
      <main className="min-h-screen bg-gradient-to-br from-purple-50 via-indigo-50 to-blue-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 flex items-center justify-center relative overflow-hidden">
        {/* Decorative background */}
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-purple-400/30 to-indigo-500/30 rounded-full blur-xl" />
          <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl" />
          <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl" />
        </div>
        <div className="container mx-auto p-6 relative z-10">
          <GlobalAuthGuard title="Payment Portal" debugInfo={debugInfo} />
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-gradient-to-br from-purple-50 via-indigo-50 to-blue-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 py-12 px-4 relative overflow-hidden">
      {/* Decorative background */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-purple-400/30 to-indigo-500/30 rounded-full blur-xl" />
        <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl" />
        <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl" />
        <div className="absolute bottom-40 right-10 w-28 h-28 bg-gradient-to-br from-emerald-400/20 to-teal-500/20 rounded-full blur-xl" />
      </div>

      <div className="max-w-6xl mx-auto relative z-10">
        {/* Hero Header */}
        <div className="text-center mb-12">
          <div className="inline-block mb-6">
            <span className="text-6xl">💎</span>
          </div>
          <h1 className="text-4xl lg:text-5xl font-black bg-gradient-to-r from-purple-600 via-indigo-600 to-blue-600 bg-clip-text text-transparent mb-4">
            Choose Your Plan
          </h1>
          <p className="text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto">
            Unlock powerful analytics, API access, and premium features with blockchain-secured payments
          </p>
        </div>

        {/* Payment Flow */}
        <PaymentClient
          paymentType="plan"
          title="Choose Your Plan"
          description="Select a plan to unlock premium features and analytics. Your subscription is secured by blockchain technology."
        />

        {/* Security Footer */}
        <div className="mt-16 text-center">
          <div className="inline-flex items-center gap-4 px-6 py-3 bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-xl border border-gray-200/50 dark:border-gray-700/50 shadow-lg">
            <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
              <span className="text-green-500">🔒</span>
              <span>Blockchain Secured</span>
            </div>
            <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />
            <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
              <span className="text-blue-500">⚡</span>
              <span>Instant Activation</span>
            </div>
            <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />
            <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
              <span className="text-purple-500">💳</span>
              <span>USDT/USDC</span>
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
