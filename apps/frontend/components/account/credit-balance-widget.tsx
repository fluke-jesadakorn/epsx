'use client';

import { Badge } from '@/components/ui';
import { createCreditsApi } from '@/shared/api/credits';
import { fmtAmt } from '@/shared/utils/formatting/currency';
import type { CreditBalance } from '@/shared/types/credits';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { ArrowRight, Coins } from 'lucide-react';
import Link from 'next/link';
import { useEffect, useState } from 'react';

export function CreditBalanceWidget() {
  const [balance, setBalance] = useState<CreditBalance | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchBalance = async () => {
      try {
        const apiClient = createFrontendApiClient();
        const creditsApi = createCreditsApi(apiClient);
        const res = await creditsApi.getBalance();

        if (res.success && res.data) {
          setBalance(res.data);
        }
      } catch (_err) {
        // Silent failure - widget shows "0.00" on error
      } finally {
        setLoading(false);
      }
    };

    void fetchBalance();
  }, []);

  if (loading) {
    return (
      <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-5 sm:p-6 shadow-xl border-2 border-emerald-300/50 dark:border-emerald-700/50 hover:shadow-2xl transition-all duration-300 group">
        <div className="flex items-center justify-between mb-4 text-2xl sm:text-3xl">
          <span className="group-hover:scale-110 transition-transform">💰</span>
          <Badge variant="outline" className="text-xs font-semibold bg-emerald-50/50 dark:bg-emerald-900/20 text-emerald-600 border-emerald-200">
            Credits
          </Badge>
        </div>
        <div className="space-y-1">
          <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Available Balance</div>
          <div className="h-8 w-24 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
        </div>
      </div>
    );
  }

  return (
    <Link
      href="/account/credits"
      className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-5 sm:p-6 shadow-xl border-2 border-emerald-300/50 dark:border-emerald-700/50 hover:shadow-2xl transition-all duration-300 group block"
    >
      <div className="flex items-center justify-between mb-4 text-2xl sm:text-3xl">
        <span className="group-hover:scale-110 transition-transform">💰</span>
        <Badge variant="outline" className="text-xs font-semibold bg-emerald-50/50 dark:bg-emerald-900/20 text-emerald-600 border-emerald-200">
          Credits
        </Badge>
      </div>
      <div className="space-y-1">
        <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Available Balance</div>
        <div className="flex items-center justify-between">
          <div className="text-lg font-bold text-gray-900 dark:text-gray-100">
            ${fmtAmt(Number(balance?.available_balance ?? 0))}
          </div>
          <ArrowRight className="w-4 h-4 text-emerald-600 dark:text-emerald-400 opacity-0 group-hover:opacity-100 group-hover:translate-x-1 transition-all" />
        </div>
        {balance && Number(balance.lifetime_earned) > 0 && (
          <div className="flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400 mt-2">
            <Coins className="w-3 h-3" />
            <span>${fmtAmt(Number(balance.lifetime_earned))} lifetime earned</span>
          </div>
        )}
      </div>
    </Link>
  );
}
