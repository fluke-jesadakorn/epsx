'use client';

import {
  getCreditStatsAction,
  getUserCreditsAction,
  grantCreditsAction,
  revokeCreditsAction,
} from '@/app/wallet-management/credits/actions';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';
import type { CreditStats, CreditTransaction, GrantCreditsRequest, RevokeCreditsRequest } from '@/shared/types/credits';
import { Calendar, Clock, Coins, Minus, Plus, Search, TrendingDown, TrendingUp, Users } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

interface CreditsManagementProps {
  activeTab: 'overview' | 'grant' | 'history';
}

export function CreditsManagement({ activeTab }: CreditsManagementProps) {
  const { isAuthenticated } = useSharedAuth();
  const [stats, setStats] = useState<CreditStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStats = useCallback(async () => {
    if (!isAuthenticated) {
      setLoading(false);
      setError('Please sign in to view credit statistics');
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const data = await getCreditStatsAction();
      setStats(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
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
      <div className="bg-destructive/10 border border-destructive/30 rounded-xl p-4">
        <p className="text-destructive text-sm">{error}</p>
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
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard
          icon={<Coins className="w-6 h-6" />}
          label="Total Credits Outstanding"
          value={stats !== null ? `$${Number(stats.total_credits_outstanding).toFixed(2)}` : '$0.00'}
          gradient="from-[#1fc7d4] to-[#7645d9]"
        />
        <StatCard
          icon={<TrendingUp className="w-6 h-6" />}
          label="Credits Granted Today"
          value={stats !== null ? `$${Number(stats.total_credits_granted_today).toFixed(2)}` : '$0.00'}
          gradient="from-[#31d0aa] to-[#1fc7d4]"
        />
        <StatCard
          icon={<TrendingDown className="w-6 h-6" />}
          label="Credits Used Today"
          value={stats !== null ? `$${Number(stats.total_credits_used_today).toFixed(2)}` : '$0.00'}
          gradient="from-[#ffb237] to-[#ed4b9e]"
        />
        <StatCard
          icon={<Users className="w-6 h-6" />}
          label="Active Users with Credits"
          value={stats?.active_users_with_credits.toString() ?? '0'}
          gradient="from-[#7645d9] to-[#ed4b9e]"
        />
      </div>

      {/* Actions */}
      <div className="flex items-center gap-3">
        <button
          onClick={handleRefreshClick}
          className="flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white rounded-xl font-semibold text-sm transition-all"
        >
          Refresh Stats
        </button>
        <button
          disabled
          className="flex items-center gap-2 px-4 py-2.5 bg-card border border-border/40 text-muted-foreground rounded-xl font-semibold text-sm opacity-50 cursor-not-allowed"
        >
          Export Report (Coming Soon)
        </button>
      </div>
    </div>
  );
}

function StatCard({ icon, label, value, gradient }: { icon: React.ReactNode; label: string; value: string; gradient: string }) {
  return (
    <div className="rounded-xl border border-border/20 bg-card p-5 overflow-hidden">
      <div className={cn('w-10 h-10 rounded-lg bg-gradient-to-r mb-3 flex items-center justify-center text-white', gradient)}>
        {icon}
      </div>
      <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2">{label}</div>
      <div className="text-2xl font-black text-foreground">{value}</div>
    </div>
  );
}

// ============================================================================
// GRANT CREDITS TAB
// ============================================================================

interface GrantFormProps {
  mode: 'grant' | 'revoke';
  walletAddress: string;
  amount: string;
  reason: string;
  expiresAt: string;
  loading: boolean;
  success: string | null;
  error: string | null;
  setWalletAddress: (v: string) => void;
  setAmount: (v: string) => void;
  setReason: (v: string) => void;
  setExpiresAt: (v: string) => void;
  onSubmit: (e: React.FormEvent) => void;
}

function GrantForm({ mode, walletAddress, amount, reason, expiresAt, loading, success, error, setWalletAddress, setAmount, setReason, setExpiresAt, onSubmit }: GrantFormProps) {
  const inputClass = 'w-full px-4 py-2.5 rounded-lg border border-border/50 bg-muted/50 text-foreground focus:ring-1 focus:ring-[#1fc7d4] focus:border-transparent transition-all text-sm';
  const labelClass = 'block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2';
  return (
    <>
      {success !== null && success.length > 0 && (
        <div className="mb-4 p-4 bg-success/10 border border-success/30 rounded-xl">
          <p className="text-success text-sm">{success}</p>
        </div>
      )}
      {error !== null && error.length > 0 && (
        <div className="mb-4 p-4 bg-destructive/10 border border-destructive/30 rounded-xl">
          <p className="text-destructive text-sm">{error}</p>
        </div>
      )}
      <form onSubmit={onSubmit} className="space-y-6">
        <div>
          <label className={labelClass}>Wallet Address</label>
          <input type="text" value={walletAddress} onChange={(e) => setWalletAddress(e.target.value)} placeholder="0x..." required className={inputClass} />
        </div>
        <div>
          <label className={labelClass}>Amount (USD)</label>
          <input type="number" step="0.01" value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="0.00" required min="0.01" className={inputClass} />
        </div>
        <div>
          <label className={labelClass}>Reason (optional)</label>
          <textarea value={reason} onChange={(e) => setReason(e.target.value)} placeholder="Promotional credit for early adopter..." rows={3} className={inputClass} />
        </div>
        {mode === 'grant' && (
          <div>
            <label className={labelClass}>Expiry Date (optional)</label>
            <input type="datetime-local" value={expiresAt} onChange={(e) => setExpiresAt(e.target.value)} className={inputClass} />
          </div>
        )}
        <button
          type="submit"
          disabled={loading}
          className={cn('w-full py-2.5 rounded-lg font-semibold text-sm text-white transition-all', loading ? 'bg-muted cursor-not-allowed text-muted-foreground' : mode === 'grant' ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90' : 'bg-destructive hover:opacity-90')}
        >
          {loading ? 'Processing...' : mode === 'grant' ? 'Grant Credits' : 'Revoke Credits'}
        </button>
      </form>
    </>
  );
}

function GrantCreditsTab() {
  const { user } = useSharedAuth();
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
      if (mode === 'grant') {
        const request: GrantCreditsRequest = {
          wallet_address: walletAddress,
          amount: parseFloat(amount),
          reason: reason !== '' ? reason : undefined,
          expires_at: expiresAt !== '' ? expiresAt : undefined,
          granted_by: user?.wallet_address ?? 'system',
        };
        const data = await grantCreditsAction(request);
        setSuccess(`Successfully granted $${amount} credits. New balance: $${Number(data.new_balance).toFixed(2)}`);
        setWalletAddress('');
        setAmount('');
        setReason('');
        setExpiresAt('');
      } else {
        const request: RevokeCreditsRequest = {
          wallet_address: walletAddress,
          amount: parseFloat(amount),
          reason: reason !== '' ? reason : undefined,
          granted_by: user?.wallet_address ?? 'system',
        };
        const data = await revokeCreditsAction(request);
        setSuccess(`Successfully revoked $${amount} credits. New balance: $${Number(data.new_balance).toFixed(2)}`);
        setWalletAddress('');
        setAmount('');
        setReason('');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Operation failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
        <div className="h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]" />
        <div className="p-6 sm:p-8">
          <h2 className="text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em] mb-6 flex items-center gap-2">
            {mode === 'grant' ? <Plus className="w-4 h-4" /> : <Minus className="w-4 h-4" />}
            {mode === 'grant' ? 'Grant' : 'Revoke'} Credits
          </h2>
          <div className="flex gap-2 mb-6">
            <button onClick={() => setMode('grant')} className={cn('flex-1 px-4 py-2 rounded-lg font-semibold text-sm transition-colors', mode === 'grant' ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white' : 'bg-muted/50 border border-border/50 text-muted-foreground hover:text-foreground')}>
              <Plus className="w-4 h-4 inline mr-2" />Grant
            </button>
            <button onClick={() => setMode('revoke')} className={cn('flex-1 px-4 py-2 rounded-lg font-semibold text-sm transition-colors', mode === 'revoke' ? 'bg-destructive text-white' : 'bg-muted/50 border border-border/50 text-muted-foreground hover:text-foreground')}>
              <Minus className="w-4 h-4 inline mr-2" />Revoke
            </button>
          </div>
          <GrantForm
            mode={mode}
            walletAddress={walletAddress}
            amount={amount}
            reason={reason}
            expiresAt={expiresAt}
            loading={loading}
            success={success}
            error={error}
            setWalletAddress={setWalletAddress}
            setAmount={setAmount}
            setReason={setReason}
            setExpiresAt={setExpiresAt}
            onSubmit={(e) => void handleSubmit(e)}
          />
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// CREDIT HISTORY TAB
// ============================================================================
 
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
      const data = await getUserCreditsAction(walletAddress, { limit: 50 });
      setTransactions(data.transactions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
      setTransactions([]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-6xl mx-auto">
      <div className="rounded-xl border border-border/20 bg-card p-4 mb-6">
        <form onSubmit={(e) => void handleSearch(e)} className="flex gap-3">
          <div className="flex-1">
            <input
              type="text"
              value={walletAddress}
              onChange={(e) => setWalletAddress(e.target.value)}
              placeholder="Enter wallet address to search..."
              required
              className="w-full px-4 py-2.5 rounded-lg border border-border/50 bg-muted/50 text-foreground focus:ring-1 focus:ring-[#1fc7d4] focus:border-transparent transition-all text-sm"
            />
          </div>
          <button
            type="submit"
            disabled={loading}
            className="px-5 py-2.5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white rounded-lg font-semibold text-sm transition-all flex items-center gap-2 disabled:opacity-50"
          >
            <Search className="w-4 h-4" />
            {loading ? 'Searching...' : 'Search'}
          </button>
        </form>
      </div>

      {error !== null && error.length > 0 && (
        <div className="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg">
          <p className="text-destructive text-sm">{error}</p>
        </div>
      )}

      {transactions.length > 0 && (
        <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
          <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
          <div className="p-4 border-b border-border/20">
            <h3 className="text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]">Credit Transactions ({transactions.length})</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-muted/30">
                <tr>
                  <th className="px-4 py-3 text-left text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]">Type</th>
                  <th className="px-4 py-3 text-left text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]">Amount</th>
                  <th className="px-4 py-3 text-left text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]">Balance After</th>
                  <th className="px-4 py-3 text-left text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]">Reason</th>
                  <th className="px-4 py-3 text-left text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]">Date</th>
                  <th className="px-4 py-3 text-left text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]">Granted By</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/20">
                {transactions.map((tx) => {
                  const txAmount = Number(tx.amount);
                  const isCredit = txAmount > 0;
                  return (
                    <tr key={tx.id} className="hover:bg-muted/20 transition-colors">
                      <td className="px-4 py-3 whitespace-nowrap">
                        <span className={cn(
                          'px-2 py-1 rounded-full text-xs font-semibold',
                          isCredit ? 'bg-[#31d0aa]/10 text-[#31d0aa]' : 'bg-[#ffb237]/10 text-[#ffb237]'
                        )}>
                          {tx.tx_type}
                        </span>
                      </td>
                      <td className={cn(
                        'px-4 py-3 whitespace-nowrap font-mono font-bold text-sm',
                        isCredit ? 'text-[#31d0aa]' : 'text-[#ffb237]'
                      )}>
                        {isCredit ? '+' : ''}{txAmount.toFixed(2)}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-foreground font-mono text-sm">
                        ${Number(tx.balance_after).toFixed(2)}
                      </td>
                      <td className="px-4 py-3 text-sm text-muted-foreground max-w-xs truncate">
                        {tx.reason ?? '-'}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-xs text-muted-foreground flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        {new Date(tx.created_at).toLocaleDateString()}
                        <Clock className="w-3 h-3 ml-2" />
                        {new Date(tx.created_at).toLocaleTimeString()}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-xs font-mono text-muted-foreground">
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
          <Coins className="w-12 h-12 text-muted-foreground/40 mx-auto mb-3" />
          <p className="text-muted-foreground text-sm">Enter a wallet address to view credit history</p>
        </div>
      )}
    </div>
  );
}
