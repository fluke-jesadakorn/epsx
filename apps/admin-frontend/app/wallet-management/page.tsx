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
    <div className="p-3 sm:p-6 space-y-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 to-pink-600 bg-clip-text text-transparent flex items-center gap-2">
          <span>👛</span> Wallet Management Hub
        </h1>
        <p className="text-muted-foreground mt-2">
          Unified management for EPSX ecosystem wallets, permissions, and subscriptions
        </p>
      </div>

      {/* Wallet Hub */}
      <WalletHub />
    </div>
  );
}