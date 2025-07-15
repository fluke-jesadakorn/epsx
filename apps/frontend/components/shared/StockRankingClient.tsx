'use client';

import React from 'react';
import StockRankingTable from '@/components/shared/StockRankingTable';
import type { StockFinancialData } from '@/types/financialChartData';

interface StockRankingClientProps {
  initialData: StockFinancialData[];
  title?: string;
  subtitle?: string;
  rankShift?: number;
  showRank?: boolean;
}

/**
 * Client component for Stock Ranking Table
 * Reuses the same table structure as /analytics page
 * Supports different configurations for different zones
 */
export default function StockRankingClient({
  initialData,
  title,
  subtitle,
  rankShift = 0,
  showRank = true,
}: StockRankingClientProps): React.JSX.Element {
  return (
    <StockRankingTable
      data={initialData}
      title={title}
      subtitle={subtitle}
      rankShift={rankShift}
      showRank={showRank}
    />
  );
}
