'use client';

import { StockDataCard } from '@/shared/components';
import type { TableDataMetrics } from '@/types/stockFetchData';
import type { CSSProperties } from 'react';
import { useEffect, useState } from 'react';

interface Props {
  style?: CSSProperties;
  className?: string;
  initialData: TableDataMetrics[];
}

export default function ClientEpsCardSection({
  style,
  className,
  initialData,
}: Props) {
  const [data, setData] = useState<TableDataMetrics[]>(initialData);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function fetchData() {
      try {
        const response = await fetch('/api/public/rankings?type=cards&limit=3');
        if (response.ok) {
          const result = await response.json();
          setData(result);
        }
      } catch (error) {
        console.error('Failed to fetch top performing companies:', error);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, []);

  // Ensure data is always an array to prevent runtime errors
  const safeData = Array.isArray(data) ? data : [];

  return (
    <div
      className={`flex w-full flex-col gap-8 ${className || ''}`}
      style={style}
    >
      {/* Enhanced Section Header */}
      <div className="mb-6 space-y-4 text-center">
        <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
          Top Performing Companies
        </h2>
        <p className="text-muted-foreground mx-auto max-w-2xl">
          Discover the data leaders with exceptional growth and performance
          metrics
        </p>
        <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
      </div>

      {/* Analytics-style card grid */}
      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:gap-6 sm:px-0">
        {isLoading ? (
          // Loading skeletons
          Array.from({ length: 3 }).map((_, index) => (
            <div
              key={index}
              className="relative w-full max-w-[350px] min-w-[240px] flex-shrink-0 overflow-visible rounded-3xl sm:min-w-[300px] from-gray-50 via-gray-100 to-gray-200 dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 animate-pulse"
            >
              <div className="p-8 pt-16">
                <div className="mb-6 text-center">
                  <div className="mb-3">
                    <div className="mb-1 h-3 w-32 bg-gray-300 dark:bg-gray-600 rounded mx-auto"></div>
                    <div className="mb-2 h-8 w-24 bg-gray-300 dark:bg-gray-600 rounded mx-auto"></div>
                    <div className="mx-auto h-px w-16 bg-gray-300 dark:bg-gray-600"></div>
                  </div>
                  <div className="flex justify-center">
                    <div className="h-12 w-32 bg-gray-300 dark:bg-gray-600 rounded-2xl"></div>
                  </div>
                </div>
              </div>
            </div>
          ))
        ) : (
          safeData
            .filter(
              item => item && typeof item === 'object' && item.symbol && item.name
            )
            .slice(0, 3)
            .map((item, index) => (
              <StockDataCard
                key={item.symbol || index}
                symbol={item.symbol || 'N/A'}
                rank={index + 1}
                epsGrowth={parseFloat(String(item.epsGrowth || '0').replace('%', '')) || 0}
                price={parseFloat(String(item.valueIndex || '0').replace(/[^0-9.-]/g, '')) || 0}
                currency={item.currency || 'USD'}
                variant="premium"
              />
            ))
        )}
      </div>
    </div>
  );
}
