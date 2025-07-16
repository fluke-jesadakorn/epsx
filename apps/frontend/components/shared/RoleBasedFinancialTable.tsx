'use client';

import React from 'react';
import { useRankingAccess } from '@/hooks/useRankingAccess';
import { UpgradePrompt } from '@/components/ui/upgrade-prompt';
import FinancialDataTable from '@/components/home/FinancialDataTable';
import { getLevelColor, getLockedRankings } from '@/app/constants/packages';
import type { StockFinancialData } from '@/types/financialChartData';

interface RoleBasedFinancialTableProps {
  data: StockFinancialData[];
  className?: string;
  style?: React.CSSProperties;
  isPublicPreview?: boolean;
}

/**
 * Enhanced FinancialDataTable with role-based access control
 * Integrates user subscription levels with stock ranking visibility
 */
export default function RoleBasedFinancialTable({
  data,
  className,
  style,
  isPublicPreview = false,
}: RoleBasedFinancialTableProps): React.JSX.Element {
  const { maxRankings, upgradeRequired, userLevel, isLoading } = useRankingAccess();

  // Filter data based on user's ranking access (unless it's public preview)
  const filteredData = React.useMemo(() => {
    if (isLoading) return data;
    if (isPublicPreview) return data; // Don't filter public preview data
    return data.slice(0, maxRankings);
  }, [data, maxRankings, isLoading, isPublicPreview]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Public preview indicator */}
      {isPublicPreview && (
        <div className="bg-blue-50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 text-center">
          <p className="text-blue-700 dark:text-blue-300 font-medium">
            📋 Public Preview: Showing rankings #100-110 as a demo of our ranking system
          </p>
        </div>
      )}
      
      {/* Main financial table with filtered data */}
      <FinancialDataTable 
        data={filteredData}
        className={className}
        style={style}
      />
      
      {/* Upgrade prompt (only show for non-public preview) */}
      {!isPublicPreview && upgradeRequired && (
        <UpgradePrompt 
          currentLevel={userLevel} 
          lockedRankings={getLockedRankings(userLevel)} 
          className="max-w-2xl mx-auto"
        />
      )}
      
      {/* Access level indicator */}
      <div className="text-center py-4">
        <div className="inline-flex items-center gap-2 px-4 py-2 bg-secondary/50 rounded-full text-sm">
          {isPublicPreview ? (
            <>
              <span className="font-medium">Public Preview:</span>
              <span className="font-bold text-blue-600">Rankings #100-110</span>
              <span className="text-muted-foreground">
                • {filteredData.length} stocks shown
              </span>
            </>
          ) : (
            <>
              <span className="font-medium">Current Plan:</span>
              <span className={`font-bold ${getLevelColor(userLevel)}`}>
                {userLevel}
              </span>
              <span className="text-muted-foreground">
                • Showing {filteredData.length} of {data.length} rankings
              </span>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
