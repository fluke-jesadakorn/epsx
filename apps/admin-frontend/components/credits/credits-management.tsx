'use client';

import { useCallback, useEffect, useState } from 'react';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { createCreditsApi } from '@/shared/api/credits';
import type { CreditStats, CreditTransaction, GrantCreditsRequest, RevokeCreditsRequest } from '@/shared/types/credits';
import { Coins, TrendingUp, TrendingDown, Users, Plus, Minus, Search, Calendar, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';

interface CreditsManagementProps {
  activeTab: 'overview' | 'grant' | 'history';
}

export function CreditsManagement({ activeTab }: CreditsManagementProps) {
  const { isAuthenticated } = useSharedAuth();
  const [stats, setStats] = useState<CreditStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStats = useCallback(async () => {
    // Don't fetch if not authenticated
    if (!isAuthenticated) {
      setLoading(false);
      setError('Please sign in to view credit statistics');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const apiClient = createAdminApiClient();
      const creditsApi = createCreditsApi(apiClient);
      const res = await creditsApi.adminGetStats();

      if (res.success && res.data) {
        setStats(res.data);
      } else {
        throw new Error('Failed to load credit stats');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [isAuthenticated]);

  useEffect(() => {
    void loadStats();
  }, [loadStats]);

  if (loading) {
    const skeletons = Array.from({ length: 4 }, (_, i) => i);
    return (
      <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {skeletons.map((i) => (
            <div key={`skeleton-${i}`} className="bg-card rounded-3xl h-40 border border-border/50" />
          ))}
        </div>
      </div>
    );
  }

  if (error !== null && error.length > 0) {
    return (
      <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p className="text-red-800 dark:text-red-200">{error}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 sm:space-y-8">
      {activeTab === 'overview' && <OverviewTab stats={stats} onRefresh={loadStats} />}
      {activeTab === 'grant' && <GrantCreditsTab />}
      {activeTab === 'history' && <CreditHistoryTab />}
    </div>
  );
}

// ============================================================================
// OVERVIEW TAB
// ============================================================================

function OverviewTab({ stats, onRefresh }: { stats: CreditStats | null; onRefresh: () => Promise<void> }) {
  const handleRefreshClick = () => {
    void onRefresh();
  };

  return (
    <div className="relative max-w-7xl mx-auto">
      {/* Stats Cards */}
      {/* eslint-disable @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard
          icon={<Coins className="w-6 h-6" />}
          label="Total Credits Outstanding"
          value={stats !== null ? `$${stats.total_credits_outstanding.toFixed(2)}` : '$0.00'}
          gradient="from-blue-500 to-cyan-500"
        />
        <StatCard
          icon={<TrendingUp className="w-6 h-6" />}
          label="Credits Granted Today"
          value={stats !== null ? `$${stats.credits_granted_today.toFixed(2)}` : '$0.00'}
          gradient="from-green-500 to-emerald-500"
        />
        <StatCard
          icon={<TrendingDown className="w-6 h-6" />}
          label="Credits Used Today"
          value={stats !== null ? `$${stats.credits_used_today.toFixed(2)}` : '$0.00'}
          gradient="from-orange-500 to-pink-500"
        />
        <StatCard
          icon={<Users className="w-6 h-6" />}
          label="Active Users with Credits"
          value={stats !== null ? stats.active_users_with_credits.toString() : '0'}
          gradient="from-purple-500 to-indigo-500"
        />
      </div>
      {/* eslint-enable @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access */}

      {/* Actions */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <div
          className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5 cursor-pointer hover:scale-105 transition-transform"
          onClick={handleRefreshClick}
        >
          <div className="relative bg-gradient-to-r from-blue-500 to-cyan-500 text-white rounded-2xl sm:rounded-3xl">
            <div className="p-6 sm:p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4">
                <span className="text-2xl">🔄</span>
              </div>
              <h3 className="text-xl sm:text-2xl font-bold mb-3">Refresh Stats</h3>
              <p className="text-white/80 mb-4 text-sm sm:text-base">Reload credit statistics from server</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                Refresh
              </div>
            </div>
          </div>
        </div>

        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-secondary/10 p-0.5 cursor-not-allowed opacity-75">
          <div className="relative bg-gradient-to-r from-purple-500 to-pink-500 text-white rounded-2xl sm:rounded-3xl">
            <div className="p-6 sm:p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4">
                <span className="text-2xl">📊</span>
              </div>
              <h3 className="text-xl sm:text-2xl font-bold mb-3">Export Report</h3>
              <p className="text-white/80 mb-4 text-sm sm:text-base">Download credit analytics report</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                Coming Soon
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function StatCard({ icon, label, value, gradient }: { icon: React.ReactNode; label: string; value: string; gradient: string }) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-xl border border-gray-200 dark:border-gray-700">
      <div className={cn('w-12 h-12 rounded-xl bg-gradient-to-r mb-4 flex items-center justify-center text-white', gradient)}>
        {icon}
      </div>
      <div className="text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">{label}</div>
      <div className="text-3xl font-bold text-gray-900 dark:text-white">{value}</div>
    </div>
  );
}

// ============================================================================
// GRANT CREDITS TAB
// ============================================================================

// eslint-disable-next-line max-lines-per-function, complexity
function GrantCreditsTab() {
  const [walletAddress, setWalletAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [reason, setReason] = useState('');
  const [expiresAt, setExpiresAt] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [mode, setMode] = useState<'grant' | 'revoke'>('grant');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setSuccess(null);
    setError(null);

    try {
      const apiClient = createAdminApiClient();
      const creditsApi = createCreditsApi(apiClient);

      if (mode === 'grant') {
        const request: GrantCreditsRequest = {
          wallet_address: walletAddress,
          amount: parseFloat(amount),
          reason: reason || undefined,
          expires_at: expiresAt || undefined,
        };

        const res = await creditsApi.adminGrantCredits(request);

        if (res.success && res.data) {
          setSuccess(`Successfully granted $${amount} credits. New balance: $${res.data.new_balance.toFixed(2)}`);
          setWalletAddress('');
          setAmount('');
          setReason('');
          setExpiresAt('');
        } else {
          throw new Error('Failed to grant credits');
        }
      } else {
        const request: RevokeCreditsRequest = {
          wallet_address: walletAddress,
          amount: parseFloat(amount),
          reason: reason || undefined,
        };

        const res = await creditsApi.adminRevokeCredits(request);

        if (res.success && res.data) {
          setSuccess(`Successfully revoked $${amount} credits. New balance: $${res.data.new_balance.toFixed(2)}`);
          setWalletAddress('');
          setAmount('');
          setReason('');
        } else {
          throw new Error('Failed to revoke credits');
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Operation failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="bg-white dark:bg-gray-800 rounded-3xl p-8 shadow-xl border border-gray-200 dark:border-gray-700">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
          {mode === 'grant' ? <Plus className="w-6 h-6 text-green-600" /> : <Minus className="w-6 h-6 text-red-600" />}
          {mode === 'grant' ? 'Grant' : 'Revoke'} Credits
        </h2>

        {/* Mode Toggle */}
        <div className="flex gap-2 mb-6">
          <button
            onClick={() => setMode('grant')}
            className={cn(
              'flex-1 px-4 py-2 rounded-lg font-medium transition-colors',
              mode === 'grant'
                ? 'bg-green-600 text-white'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
            )}
          >
            <Plus className="w-4 h-4 inline mr-2" />
            Grant
          </button>
          <button
            onClick={() => setMode('revoke')}
            className={cn(
              'flex-1 px-4 py-2 rounded-lg font-medium transition-colors',
              mode === 'revoke'
                ? 'bg-red-600 text-white'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
            )}
          >
            <Minus className="w-4 h-4 inline mr-2" />
            Revoke
          </button>
        </div>

        {success !== null && success.length > 0 && (
          <div className="mb-4 p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
            <p className="text-green-800 dark:text-green-200">{success}</p>
          </div>
        )}

        {error !== null && error.length > 0 && (
          <div className="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <p className="text-red-800 dark:text-red-200">{error}</p>
          </div>
        )}

        <form onSubmit={(e) => void handleSubmit(e)} className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Wallet Address
            </label>
            <input
              type="text"
              value={walletAddress}
              onChange={(e) => setWalletAddress(e.target.value)}
              placeholder="0x..."
              required
              className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Amount (USD)
            </label>
            <input
              type="number"
              step="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              required
              min="0.01"
              className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Reason (optional)
            </label>
            <textarea
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Promotional credit for early adopter..."
              rows={3}
              className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {mode === 'grant' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Expiry Date (optional)
              </label>
              <input
                type="datetime-local"
                value={expiresAt}
                onChange={(e) => setExpiresAt(e.target.value)}
                className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              />
            </div>
          )}

          <button
            type="submit"
            disabled={loading}
            className={cn(
              'w-full py-3 rounded-lg font-bold text-white transition-all',
              loading ? 'bg-gray-400 cursor-not-allowed' : mode === 'grant' ? 'bg-green-600 hover:bg-green-700' : 'bg-red-600 hover:bg-red-700'
            )}
          >
            {loading ? 'Processing...' : mode === 'grant' ? 'Grant Credits' : 'Revoke Credits'}
          </button>
        </form>
      </div>
    </div>
  );
}

// ============================================================================
// CREDIT HISTORY TAB
// ============================================================================

// eslint-disable-next-line max-lines-per-function
function CreditHistoryTab() {
  const [walletAddress, setWalletAddress] = useState('');
  const [transactions, setTransactions] = useState<CreditTransaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const apiClient = createAdminApiClient();
      const creditsApi = createCreditsApi(apiClient);
      const res = await creditsApi.adminGetUserCredits(walletAddress, { limit: 50 });

      if (res.success && res.data) {
        setTransactions(res.data.transactions);
      } else {
        throw new Error('Failed to load credit history');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
      setTransactions([]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-6xl mx-auto">
      <div className="bg-white dark:bg-gray-800 rounded-3xl p-6 shadow-xl border border-gray-200 dark:border-gray-700 mb-6">
        <form onSubmit={(e) => void handleSearch(e)} className="flex gap-4">
          <div className="flex-1">
            <input
              type="text"
              value={walletAddress}
              onChange={(e) => setWalletAddress(e.target.value)}
              placeholder="Enter wallet address to search..."
              required
              className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <button
            type="submit"
            disabled={loading}
            className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-bold transition-all flex items-center gap-2"
          >
            <Search className="w-4 h-4" />
            {loading ? 'Searching...' : 'Search'}
          </button>
        </form>
      </div>

      {error !== null && error.length > 0 && (
        <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-red-800 dark:text-red-200">{error}</p>
        </div>
      )}

      {transactions.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-3xl shadow-xl border border-gray-200 dark:border-gray-700 overflow-hidden">
          <div className="p-6 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-xl font-bold text-gray-900 dark:text-white">Credit Transactions ({transactions.length})</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Type</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Amount</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Balance After</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Reason</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Date</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Granted By</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {transactions.map((tx) => {
                  const isCredit = tx.amount > 0;
                  return (
                    <tr key={tx.id} className="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span className={cn(
                          'px-2 py-1 rounded-full text-xs font-medium',
                          isCredit ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' : 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'
                        )}>
                          {tx.tx_type}
                        </span>
                      </td>
                      <td className={cn(
                        'px-6 py-4 whitespace-nowrap font-mono font-bold',
                        isCredit ? 'text-green-600 dark:text-green-400' : 'text-orange-600 dark:text-orange-400'
                      )}>
                        {isCredit ? '+' : ''}{tx.amount.toFixed(2)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-gray-900 dark:text-white font-mono">
                        ${tx.balance_after.toFixed(2)}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-600 dark:text-gray-400 max-w-xs truncate">
                        {tx.reason ?? '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-600 dark:text-gray-400 flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        {new Date(tx.created_at).toLocaleDateString()}
                        <Clock className="w-3 h-3 ml-2" />
                        {new Date(tx.created_at).toLocaleTimeString()}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-xs font-mono text-gray-600 dark:text-gray-400">
                        {tx.granted_by !== null && tx.granted_by.length > 0 ? `${tx.granted_by.slice(0, 6)}...${tx.granted_by.slice(-4)}` : 'System'}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {!loading && transactions.length === 0 && (walletAddress === '' || walletAddress.length === 0) && (
        <div className="text-center py-12">
          <Coins className="w-16 h-16 text-gray-400 dark:text-gray-600 mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400">Enter a wallet address to view credit history</p>
        </div>
      )}
    </div>
  );
}
