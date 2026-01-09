/**
 * Admin Wallet Management Page
 * Complete unified hub for managing wallet users and permissions
 */
'use client';

import { WalletHub } from '@/components/wallet/WalletHub';

/**
 * Wallet Management Hub Page
 * 
 * Features:
 * - Platform filter (Analytics, Pay, Token, Markets)
 * - Quick stats dashboard
 * - Wallet search and filtering
 * - Detail panel with permissions
 * - Permission assignment (Manual)
 * - Temporary disable/re-enable
 * - Bulk actions
 */
export default function WalletManagementPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-cyan-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-green-400/15 to-emerald-500/15 rounded-full blur-xl"></div>
      </div>

      <div className="relative container mx-auto px-4 py-8 max-w-7xl">
        {/* Header */}
        <div className="mb-8 text-center sm:text-left">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 to-pink-600 bg-clip-text text-transparent mb-4">
              👛 Wallet Management Hub
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto sm:mx-0">
            Unified management for EPSX ecosystem wallets, permissions, and subscriptions
          </p>
        </div>

        {/* Wallet Hub */}
        <WalletHub />
      </div>
    </div>
  );
}