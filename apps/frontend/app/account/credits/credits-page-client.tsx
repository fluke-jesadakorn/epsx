'use client';

import { cn } from '@/lib/utils';
import { createCreditsApi } from '@/shared/api/credits';
import type { CreditBalance, CreditTransaction, CreditTransactionType } from '@/shared/types/credits';
import { fmtAmt } from '@/shared/utils/formatting/currency';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { ArrowDownRight, ArrowUpRight, Calendar, Clock, Coins, DollarSign, TrendingDown, TrendingUp } from 'lucide-react';
import { useEffect, useState } from 'react';

const TX_TYPE_LABELS: Record<CreditTransactionType, string> = {
  grant: 'Admin Grant',
  revoke: 'Admin Revoke',
  payment_debit: 'Payment',
  proration_credit: 'Proration Credit',
  refund: 'Refund',
  expiry: 'Expired',
  adjustment: 'Adjustment',
};

const TX_TYPE_COLORS: Record<CreditTransactionType, { bg: string; text: string; icon: string }> = {
  grant: { bg: 'bg-green-50 dark:bg-green-900/20', text: 'text-green-700 dark:text-green-400', icon: 'text-green-600 dark:text-green-500' },
  proration_credit: { bg: 'bg-blue-50 dark:bg-blue-900/20', text: 'text-blue-700 dark:text-blue-400', icon: 'text-blue-600 dark:text-blue-500' },
  refund: { bg: 'bg-emerald-50 dark:bg-emerald-900/20', text: 'text-emerald-700 dark:text-emerald-400', icon: 'text-emerald-600 dark:text-emerald-500' },
  payment_debit: { bg: 'bg-orange-50 dark:bg-orange-900/20', text: 'text-orange-700 dark:text-orange-400', icon: 'text-orange-600 dark:text-orange-500' },
  revoke: { bg: 'bg-red-50 dark:bg-red-900/20', text: 'text-red-700 dark:text-red-400', icon: 'text-red-600 dark:text-red-500' },
  expiry: { bg: 'bg-gray-50 dark:bg-gray-800', text: 'text-gray-700 dark:text-gray-400', icon: 'text-gray-600 dark:text-gray-500' },
  adjustment: { bg: 'bg-purple-50 dark:bg-purple-900/20', text: 'text-purple-700 dark:text-purple-400', icon: 'text-purple-600 dark:text-purple-500' },
};

export function CreditsPageClient() {
  const [balance, setBalance] = useState<CreditBalance | null>(null);
  const [transactions, setTransactions] = useState<CreditTransaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<CreditTransactionType | 'all'>('all');

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        setError(null);

        const apiClient = createFrontendApiClient();
        const creditsApi = createCreditsApi(apiClient);

        // Fetch balance
        const balanceRes = await creditsApi.getBalance();
        if (balanceRes.success && balanceRes.data) {
          setBalance(balanceRes.data);
        }

        // Fetch transactions
        const historyRes = await creditsApi.getHistory({
          limit: 50,
          tx_type: filter === 'all' ? undefined : filter,
        });

        if (historyRes.success && historyRes.data) {
          setTransactions(historyRes.data.data);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load credit data');
      } finally {
        setLoading(false);
      }
    };

    void fetchData();
  }, [filter]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 dark:border-blue-400" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="max-w-4xl mx-auto p-6">
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-red-800 dark:text-red-200">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">Credit Balance</h1>
        <p className="text-gray-600 dark:text-gray-400">
          Manage your EPSX credits and view transaction history
        </p>
      </div>

      {/* Balance Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Available Balance */}
        <div className="bg-gradient-to-br from-blue-500 to-blue-600 dark:from-blue-600 dark:to-blue-700 rounded-xl p-6 text-white shadow-lg">
          <div className="flex items-center justify-between mb-4">
            <div className="p-3 bg-white/20 rounded-lg">
              <Coins className="h-6 w-6" />
            </div>
            <DollarSign className="h-8 w-8 opacity-50" />
          </div>
          <div className="space-y-1">
            <p className="text-sm opacity-90">Available Balance</p>
            <p className="text-4xl font-bold">${fmtAmt(Number(balance?.available_balance ?? 0))}</p>
          </div>
        </div>

        {/* Lifetime Earned */}
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl p-6">
          <div className="flex items-center justify-between mb-4">
            <div className="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
              <TrendingUp className="h-6 w-6 text-green-600 dark:text-green-400" />
            </div>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-gray-600 dark:text-gray-400">Lifetime Earned</p>
            <p className="text-3xl font-bold text-gray-900 dark:text-white">
              ${fmtAmt(Number(balance?.lifetime_earned ?? 0))}
            </p>
          </div>
        </div>

        {/* Lifetime Spent */}
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl p-6">
          <div className="flex items-center justify-between mb-4">
            <div className="p-3 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
              <TrendingDown className="h-6 w-6 text-orange-600 dark:text-orange-400" />
            </div>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-gray-600 dark:text-gray-400">Lifetime Spent</p>
            <p className="text-3xl font-bold text-gray-900 dark:text-white">
              ${fmtAmt(Number(balance?.lifetime_spent ?? 0))}
            </p>
          </div>
        </div>
      </div>

      {/* Transaction History */}
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl overflow-hidden">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">Transaction History</h2>

          {/* Filter Tabs */}
          <div className="flex flex-wrap gap-2">
            <button
              onClick={() => setFilter('all')}
              className={cn(
                'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
                filter === 'all'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
              )}
            >
              All
            </button>
            {Object.entries(TX_TYPE_LABELS).map(([type, label]) => (
              <button
                key={type}
                onClick={() => setFilter(type as CreditTransactionType)}
                className={cn(
                  'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
                  filter === type
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                )}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        {/* Transaction List */}
        <div className="divide-y divide-gray-200 dark:divide-gray-700">
          {transactions.length === 0 ? (
            <div className="p-12 text-center">
              <Coins className="h-12 w-12 text-gray-400 dark:text-gray-600 mx-auto mb-3" />
              <p className="text-gray-600 dark:text-gray-400">No transactions found</p>
            </div>
          ) : (
            transactions.map((tx) => {
              const colors = TX_TYPE_COLORS[tx.tx_type];
              const txAmount = Number(tx.amount);
              const isCredit = txAmount > 0;

              return (
                <div key={tx.id} className="p-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                  <div className="flex items-start justify-between">
                    <div className="flex items-start gap-3 flex-1">
                      <div className={cn('p-2 rounded-lg', colors.bg)}>
                        {isCredit ? (
                          <ArrowDownRight className={cn('h-5 w-5', colors.icon)} />
                        ) : (
                          <ArrowUpRight className={cn('h-5 w-5', colors.icon)} />
                        )}
                      </div>

                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="font-medium text-gray-900 dark:text-white">
                            {TX_TYPE_LABELS[tx.tx_type]}
                          </span>
                          <span className={cn('px-2 py-0.5 rounded-full text-xs font-medium', colors.bg, colors.text)}>
                            {tx.tx_type}
                          </span>
                        </div>

                        {tx.reason && (
                          <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">{tx.reason}</p>
                        )}

                        <div className="flex flex-wrap items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
                          <div className="flex items-center gap-1">
                            <Calendar className="h-3 w-3" />
                            {new Date(tx.created_at).toLocaleDateString()}
                          </div>
                          <div className="flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {new Date(tx.created_at).toLocaleTimeString()}
                          </div>
                          {tx.granted_by && (
                            <div className="text-xs">
                              by {tx.granted_by.slice(0, 6)}...{tx.granted_by.slice(-4)}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>

                    <div className="text-right ml-4">
                      <div className={cn('text-lg font-bold', isCredit ? 'text-green-600 dark:text-green-400' : 'text-orange-600 dark:text-orange-400')}>
                        {isCredit ? '+' : ''}{fmtAmt(txAmount)}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        Balance: ${fmtAmt(Number(tx.balance_after))}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
