'use client';

import React from 'react';
import { useRankingAccess } from '@/hooks/useRankingAccess';
import { UpgradePrompt } from '@/components/ui/upgrade-prompt';
import FinancialDataTable from '@/components/home/FinancialDataTable';
import type { StockFinancialData } from '@/types/financialChartData';

interface RoleBasedFinancialTableProps {
  data: StockFinancialData[];
  className?: string;
  style?: React.CSSProperties;
}

/**
 * Enhanced FinancialDataTable with role-based access control
 * Integrates user subscription levels with stock ranking visibility
 */
export default function RoleBasedFinancialTable({
  data,
  className,
  style,
}: RoleBasedFinancialTableProps): React.JSX.Element {
  const { maxRankings, upgradeRequired, userLevel, isLoading } = useRankingAccess();

  // Filter data based on user's ranking access
  const filteredData = React.useMemo(() => {
    if (isLoading) return data;
    return data.slice(0, maxRankings);
  }, [data, maxRankings, isLoading]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Main financial table with filtered data */}
      <FinancialDataTable 
        data={filteredData}
        className={className}
        style={style}
      />
      
      {/* Upgrade prompt for users who have reached their limit */}
      {upgradeRequired && (
        <UpgradePrompt 
          currentLevel={userLevel} 
          lockedRankings={userLevel === 'BASIC' ? 20 : 50} 
          className="max-w-2xl mx-auto"
        />
      )}
      
      {/* Access level indicator */}
      <div className="text-center py-4">
        <div className="inline-flex items-center gap-2 px-4 py-2 bg-secondary/50 rounded-full text-sm">
          <span className="font-medium">Current Plan:</span>
          <span className={`font-bold ${
            userLevel === 'BASIC' ? 'text-gray-600' : 
            userLevel === 'SILVER' ? 'text-blue-600' : 
            userLevel === 'GOLD' ? 'text-yellow-600' : 
            'text-purple-600'
          }`}>
            {userLevel}
          </span>
          <span className="text-muted-foreground">
            • Showing {filteredData.length} of {data.length} rankings
          </span>
        </div>
      </div>
    </div>
  );
}
