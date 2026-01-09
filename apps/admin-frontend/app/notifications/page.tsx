'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';

import { NotificationManagement } from '@/components/notifications/NotificationManagement';
import { SendNotificationForm } from '@/components/notifications/SendNotificationForm';
import { useSharedAuth } from '@/shared/components/auth/Provider';

function NotificationsSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-blue-400 to-purple-500 rounded-2xl w-96 mx-auto mb-4 shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto"></div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-12">
          {Array.from({ length: 2 }, (_, i) => `action-card-${i}`).map((cardId) => (
            <div key={cardId} className="bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 rounded-3xl p-8 shadow-2xl">
              <div className="h-12 w-12 bg-white/20 rounded-2xl mb-6"></div>
              <div className="h-8 bg-white/30 rounded-xl mb-4 w-3/4"></div>
              <div className="h-5 bg-white/20 rounded-lg mb-6 w-full"></div>
              <div className="h-12 bg-white/40 rounded-2xl"></div>
            </div>
          ))}
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }, (_, i) => `stats-card-${i}`).map((cardId) => (
            <div key={cardId} className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-6 shadow-xl border border-white/20">
              <div className="h-6 bg-gradient-to-r from-blue-300 to-purple-400 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-gray-200 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

/**
 *
 */
export default function NotificationsPage() {
  const { user, isLoading, isAuthenticated } = useSharedAuth();
  const router = useRouter();
  const [activeTab, setActiveTab] = useState<'overview' | 'send'>('overview');

  if (isLoading) {
    return <NotificationsSkeleton />;
  }

  if (!isAuthenticated || !user) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Authentication Required</h1>
          <p className="text-gray-600 dark:text-gray-400">Please connect your wallet to access this page.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto mb-6">
        <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl p-2">
            <div className="grid grid-cols-2 gap-2">
              <button
                onClick={() => setActiveTab('overview')}
                className={`px-6 py-3 rounded-xl font-semibold text-base min-h-[44px] ${
                  activeTab === 'overview'
                    ? 'bg-gradient-to-r from-blue-400 to-purple-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                📊 Overview
              </button>
              <button
                onClick={() => setActiveTab('send')}
                className={`px-6 py-3 rounded-xl font-semibold text-base min-h-[44px] ${
                  activeTab === 'send'
                    ? 'bg-gradient-to-r from-pink-400 to-orange-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                📨 Send Notification
              </button>
            </div>
          </div>
        </div>
      </div>

      {activeTab === 'overview' ? (
        <NotificationManagement currentUser={user} />
      ) : (
        <div className="max-w-7xl mx-auto">
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-pink-400/20 via-orange-400/20 to-purple-400/20 p-0.5">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl p-8">
              <h2 className="text-2xl font-bold mb-6 bg-gradient-to-r from-pink-600 via-orange-600 to-purple-600 bg-clip-text text-transparent">
                Send Notification
              </h2>
              <div className="max-w-3xl">
                <SendNotificationForm
                  onSuccess={() => {
                    setActiveTab('overview');
                  }}
                  onCancel={() => {
                    setActiveTab('overview');
                  }}
                />
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
