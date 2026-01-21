/**
 * Admin Wallet Management Page
 * Complete unified hub for managing wallet users and permissions
 */
'use client';

import { Wallet } from 'lucide-react';

import { PageHeader, PageLayout } from '@/components/shared';
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
    <PageLayout>
      <PageHeader
        title="Wallet Management Hub"
        subtitle="Unified management for EPSX ecosystem wallets, permissions, and subscriptions"
        icon="Wallet"
        gradient="info"
      />

      <WalletHub />
    </PageLayout>
  );
}
