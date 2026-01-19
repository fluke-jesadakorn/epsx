'use client';

import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { PaymentAnalytics } from '@/components/payments/PaymentAnalytics';
import { PaymentLinksManagement } from '@/components/payments/PaymentLinksManagement';
import { PaymentsManagement } from '@/components/payments/PaymentsManagement';
import { UserAccessManagement } from '@/components/payments/UserAccessManagement';
import { useSharedAuth } from '@/shared/components/auth/Provider';

type TabType = 'payments' | 'user-access' | 'payment-links' | 'analytics';

function PaymentsHubSkeleton() {
  return (
    <div className="min-h-screen bg-background p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-blue-400 to-purple-500 rounded-2xl w-96 mx-auto mb-4 shadow-xl animate-pulse"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto animate-pulse"></div>
        </div>

        {/* Tab navigation skeleton */}
        <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5 mb-8">
          <div className="relative bg-white dark:bg-gray-900 rounded-2xl p-2">
            <div className="grid grid-cols-4 gap-2">
              {Array.from({ length: 4 }, (_, i) => `tab-${i}`).map((tabId) => (
                <div key={tabId} className="h-12 bg-gray-200 dark:bg-gray-700 rounded-xl animate-pulse"></div>
              ))}
            </div>
          </div>
        </div>

        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-12">
          {Array.from({ length: 2 }, (_, i) => `action-card-${i}`).map((cardId) => (
            <div key={cardId} className="bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 rounded-3xl p-8 shadow-2xl animate-pulse">
              <div className="h-12 w-12 bg-white/20 rounded-2xl mb-6"></div>
              <div className="h-8 bg-white/30 rounded-xl mb-4 w-3/4"></div>
              <div className="h-5 bg-white/20 rounded-lg mb-6 w-full"></div>
              <div className="h-12 bg-white/40 rounded-2xl"></div>
            </div>
          ))}
        </div>

        {/* Stats grid skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }, (_, i) => `stats-card-${i}`).map((cardId) => (
            <div key={cardId} className="bg-white dark:bg-gray-800 rounded-3xl p-6 shadow-xl border border-white/20 animate-pulse">
              <div className="h-6 bg-gradient-to-r from-blue-300 to-purple-400 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-gray-200 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>

        {/* Content skeleton */}
        <div className="bg-white dark:bg-gray-800 rounded-3xl shadow-2xl border border-white/30 overflow-hidden animate-pulse">
          <div className="p-8">
            <div className="h-8 bg-gradient-to-r from-blue-400 to-purple-500 rounded-xl mb-6 w-1/3"></div>
            <div className="space-y-4">
              {Array.from({ length: 6 }, (_, i) => `row-${i}`).map((rowId) => (
                <div key={rowId} className="flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="h-10 w-10 bg-gradient-to-br from-blue-400 to-purple-500 rounded-2xl"></div>
                    <div className="space-y-2 flex-1">
                      <div className="h-5 bg-gray-300 rounded-lg w-1/3"></div>
                      <div className="h-4 bg-gray-200 rounded-lg w-1/2"></div>
                    </div>
                  </div>
                  <div className="h-8 w-24 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 *
 */
export default function AdminPaymentsPage() {
  const { isAuthenticated, isLoading } = useSharedAuth();
  const router = useRouter();
  const [activeTab, setActiveTab] = useState<TabType>('payments');

  // Redirect to auth if not authenticated
  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/auth');
    }
  }, [isAuthenticated, isLoading, router]);

  // Show loading state while checking authentication
  if (isLoading) {
    return <PaymentsHubSkeleton />;
  }

  // Show loading state if not authenticated (will redirect)
  if (!isAuthenticated) {
    return <PaymentsHubSkeleton />;
  }

  const tabs = [
    { id: 'payments' as TabType, label: '💳 Payments', gradient: 'from-blue-400 to-purple-500' },
    { id: 'user-access' as TabType, label: '👥 User Access', gradient: 'from-emerald-400 to-teal-500' },
    { id: 'payment-links' as TabType, label: '🔗 Links', gradient: 'from-purple-400 to-pink-500' },
    { id: 'analytics' as TabType, label: '📊 Analytics', gradient: 'from-indigo-400 to-fuchsia-500' },
  ];

  return (
    <div className="min-h-screen bg-background p-6">
      <div className="max-w-7xl mx-auto mb-6">
        {/* Tab Navigation */}
        <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
          <div className="relative bg-white dark:bg-gray-900 rounded-2xl p-2">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all duration-200 ${activeTab === tab.id
                    ? `bg-gradient-to-r ${tab.gradient} text-white shadow-lg`
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                    }`}
                >
                  {tab.label}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {activeTab === 'payments' && <PaymentsManagement />}
      {activeTab === 'user-access' && <UserAccessManagement />}
      {activeTab === 'payment-links' && <PaymentLinksManagement />}
      {activeTab === 'analytics' && <PaymentAnalytics />}
    </div>
  );
}