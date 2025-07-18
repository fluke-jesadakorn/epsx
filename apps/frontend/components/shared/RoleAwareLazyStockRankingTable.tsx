'use client';

import React from 'react';
import { useRankingAccess } from '@/hooks/useRankingAccess';
import { UpgradePrompt } from '@/components/ui/upgrade-prompt';
import LazyStockRankingTable from '@/components/shared/LazyStockRankingTable';
import { getLevelColor, getLockedRankings } from '@/app/constants/packages';
import { formatLevelAsNumber } from '@/utils/level-utils';
import type { StockFinancialData } from '@/types/financialChartData';

interface RoleAwareLazyStockRankingTableProps {
  data?: StockFinancialData[];
  title?: string;
  subtitle?: string;
  showRank?: boolean;
  rankShift?: number;
  maxCards?: number;
  useLazyLoading?: boolean;
}

/**
 * Enhanced LazyStockRankingTable with role-based access control
 * Integrates user subscription levels with data ranking visibility
 */
export default function RoleAwareLazyStockRankingTable({
  data = [],
  title = "🍯 Sweet Performance Rankings 📊",
  subtitle = "Discover the most delicious data insights with our comprehensive analytics",
  showRank = true,
  rankShift = 0,
  maxCards = 20,
  useLazyLoading = false,
}: RoleAwareLazyStockRankingTableProps): React.JSX.Element {
  const { maxRankings, upgradeRequired, userLevel, isLoading } = useRankingAccess();

  // Apply user-based limits to maxCards
  const effectiveMaxCards = Math.min(maxCards, maxRankings);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Enhanced subtitle with access info */}
      <LazyStockRankingTable
        data={data}
        title={title}
        subtitle={`${subtitle} • Showing ${effectiveMaxCards} rankings for ${formatLevelAsNumber(userLevel)} plan`}
        showRank={showRank}
        rankShift={rankShift}
        maxCards={effectiveMaxCards}
        useLazyLoading={useLazyLoading}
      />
      
      {/* Upgrade prompt for users who have reached their limit */}
      {upgradeRequired && (
        <div className="max-w-4xl mx-auto px-4">
          <UpgradePrompt 
            currentLevel={userLevel} 
            lockedRankings={getLockedRankings(userLevel)} 
          />
        </div>
      )}
      
      {/* Access level indicator */}
      <div className="text-center py-4">
        <div className="inline-flex items-center gap-2 px-4 py-2 bg-secondary/50 rounded-full text-sm">
          <span className="font-medium">Current Plan:</span>
          <span className={`font-bold ${getLevelColor(userLevel)}`}>
            {formatLevelAsNumber(userLevel)}
          </span>
          <span className="text-muted-foreground">
            • Access to {maxRankings} rankings
          </span>
        </div>
      </div>
    </div>
  );
}
