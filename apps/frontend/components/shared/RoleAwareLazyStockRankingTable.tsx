'use client';

import React from 'react';
import LazyStockRankingTable from '@/components/shared/LazyStockRankingTable';
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
 * Clean implementation using new permission system
 * No legacy dependencies - built for fresh start
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
  
  // TODO: Replace with actual permission service when ready
  // const { canRead, canAnalyze, canExport, maxRankings } = useStockAnalyticsPermissions();
  
  // Temporary implementation for clean start
  const maxRankings = 10; // Default limit
  const canAnalyze = true;
  const canExport = false;
  const userTier = 'BRONZE';
  const isLoading = false;

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
        subtitle={`${subtitle} • Showing ${effectiveMaxCards} rankings for ${userTier} plan`}
        showRank={showRank}
        rankShift={rankShift}
        maxCards={effectiveMaxCards}
        useLazyLoading={useLazyLoading}
      />
      
      {/* Access level indicator */}
      <div className="text-center py-4">
        <div className="inline-flex items-center gap-2 px-4 py-2 bg-secondary/50 rounded-full text-sm">
          <span className="font-medium">Current Plan:</span>
          <span className="font-bold text-orange-600">
            {userTier}
          </span>
          <span className="text-muted-foreground">
            • Access to {maxRankings} rankings
          </span>
          {/* Show permission-based access info */}
          {canAnalyze && (
            <span className="text-green-600">• Analysis Available</span>
          )}
          {canExport && (
            <span className="text-blue-600">• Export Available</span>
          )}
        </div>
      </div>
      
      {/* Show upgrade prompt for basic users */}
      {userTier === 'BRONZE' && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mx-auto max-w-md text-center">
          <p className="text-sm text-blue-800">
            🚀 Upgrade to unlock more rankings and advanced features!
          </p>
          <button className="mt-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
            Upgrade Now
          </button>
        </div>
      )}
    </div>
  );
}
