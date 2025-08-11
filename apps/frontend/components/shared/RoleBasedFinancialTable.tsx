'use client';

import React from 'react';
import FinancialDataTable from '@/components/home/FinancialDataTable';
import { useSession } from 'next-auth/react';
import { Card, CardContent } from '@/components/ui/card';
import { Lock } from 'lucide-react';
import type { StockFinancialData } from '@/types/financialChartData';

interface RoleBasedFinancialTableProps {
  data: StockFinancialData[];
  title?: string;
  subtitle?: string;
  showRank?: boolean;
  rankShift?: number;
  className?: string;
}

/**
 * Role-based Financial Table component that shows data based on user permissions
 * Falls back to basic table if user doesn't have premium access
 */
export default function RoleBasedFinancialTable({
  data,
  title = "🍯 Premium Financial Rankings 📊",
  subtitle = "Access exclusive financial insights with role-based data visibility",
  showRank = true,
  rankShift = 0,
  className = "",
}: RoleBasedFinancialTableProps): React.JSX.Element {
  const { data: session } = useSession();
  const user = session?.user;

  // Apply rank shift if needed - moved before conditional returns
  const processedData = React.useMemo(() => {
    if (rankShift === 0) return data;
    
    return data.map((item, index) => ({
      ...item,
      displayRank: index + 1 + rankShift,
    }));
  }, [data, rankShift]);

  // Basic role checking - in a real implementation, this would use proper permission system
  const hasAccess = user !== null;

  if (!hasAccess) {
    return (
      <Card className={className}>
        <CardContent className="p-12 text-center">
          <Lock className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">Premium Access Required</h3>
          <p className="text-gray-600 dark:text-gray-400">
            Please log in to access financial data
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`w-full ${className}`}>
      {/* Header section */}
      <div className="text-center py-8">
        <h2 className="text-3xl sm:text-4xl font-bold mb-4">
          <span className="bg-gradient-to-r from-orange-500 via-yellow-400 to-orange-600 bg-clip-text text-transparent">
            {title}
          </span>
        </h2>
        <p className="text-lg text-slate-600 dark:text-slate-300 max-w-2xl mx-auto">
          {subtitle}
        </p>
        {showRank && (
          <div className="mt-2 text-sm text-slate-500 dark:text-slate-400">
            {rankShift !== 0 && `Ranking shifted by ${rankShift} positions`}
          </div>
        )}
      </div>
      
      <FinancialDataTable 
        data={processedData}
        className="min-h-screen"
      />
    </div>
  );
}