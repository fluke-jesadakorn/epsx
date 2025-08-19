'use client';

import { Card, CardContent, CardHeader } from '@/components/ui';

import type { TableDataMetrics } from '@/types/stockFetchData';
import type { CSSProperties } from 'react';

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
  // Ensure initialData is always an array to prevent runtime errors
  const safeData = Array.isArray(initialData) ? initialData : [];
  const getMarketColor = (marketCode: string | undefined) => {
    switch (marketCode) {
      case 'TYO':
        return 'text-blue-500'; // Blue
      case 'BOM':
        return 'text-green-500'; // Green
      case 'OTC':
        return 'text-purple-600'; // Purple
      default:
        return 'text-gray-400'; // Grey
    }
  };

  return (
    <div
      className={`flex w-full flex-col gap-8 ${className || ''}`}
      style={style}
    >
      {/* Enhanced Section Header */}
      <div className="animate-slide-up mb-6 space-y-4 text-center">
        <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
          Top Performing Companies
        </h2>
        <p className="text-muted-foreground mx-auto max-w-2xl">
          Discover the data leaders with exceptional growth and performance
          metrics
        </p>
        <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
      </div>

      <div className="animate-slide-up-delayed grid w-full grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
        {safeData
          .filter(
            item => item && typeof item === 'object' && item.symbol && item.name
          )
          .slice(0, 3)
          .map((item, index) => {
            return (
              <Card
                key={item.symbol || index}
                className="card-pancake hover:pancake-shadow group overflow-hidden"
              >
                {/* Ranking badge */}
                {index < 3 && (
                  <div className="absolute top-4 right-4 z-10">
                    <span
                      className={`rounded-full px-3 py-1.5 text-xs font-bold shadow-lg transition-transform duration-300 group-hover:scale-110 ${index === 0 ? 'bg-gradient-to-r from-yellow-400 to-yellow-500 text-yellow-900' : ''} ${index === 1 ? 'bg-gradient-to-r from-gray-400 to-gray-500 text-gray-900' : ''} ${index === 2 ? 'bg-gradient-to-r from-amber-400 to-amber-500 text-amber-900' : ''} `}
                    >
                      #{index + 1}
                    </span>
                  </div>
                )}

                <CardHeader className="relative pb-4">
                  <div className="flex flex-col space-y-3">
                    <div className="flex items-start justify-between">
                      <span
                        className={`${getMarketColor(item.exchange)} bg-secondary border-secondary rounded-full border px-3 py-1.5 text-sm font-semibold transition-transform duration-300 group-hover:scale-105`}
                      >
                        {item.exchange || 'N/A'}
                      </span>
                    </div>
                    <div className="flex items-center space-x-3">
                      <p className="font-semibold">{item.symbol || 'N/A'}</p>
                      <span className="text-muted-foreground">•</span>
                      <p className="text-muted-foreground truncate">
                        {item.name || 'Unknown Company'}
                      </p>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="from-blue-500/5 to-purple-500/5 transition-colors duration-300 group-hover:bg-gradient-to-br">
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <p className="text-muted-foreground text-sm">Signal</p>
                        <p
                          className={`font-semibold transition-colors ${
                            item.startBuy && item.startBuy.active
                              ? 'text-green-500 group-hover:text-green-400'
                              : item.startAction &&
                                  item.startAction.type === 'sell' &&
                                  item.startAction.active
                                ? 'text-rose-500 group-hover:text-rose-400'
                                : 'text-yellow-500 group-hover:text-yellow-400'
                          }`}
                        >
                          {item.startBuy && item.startBuy.active
                            ? 'Buy'
                            : item.startAction &&
                                item.startAction.type === 'sell' &&
                                item.startAction.active
                              ? 'Sell'
                              : 'Hold'}
                        </p>
                      </div>

                      <div className="flex items-center justify-between">
                        <p className="text-muted-foreground text-sm">Growth</p>
                        <p
                          className={`font-semibold transition-colors ${
                            parseFloat(item.epsGrowth || '0') >= 0
                              ? 'text-green-500 group-hover:text-green-400'
                              : 'text-rose-500 group-hover:text-rose-400'
                          }`}
                        >
                          {item.epsGrowth || '0%'}
                        </p>
                      </div>

                      <div className="flex items-center justify-between">
                        <p className="text-muted-foreground text-sm">Status</p>
                        <p className="text-sm">
                          {item.startAction &&
                          item.startAction.type === 'hold' &&
                          item.startAction.active
                            ? 'Waiting for Hold'
                            : item.startAction &&
                                item.startAction.type === 'sell' &&
                                !item.startAction.active
                              ? 'Waiting for Sell'
                              : 'Active'}
                        </p>
                      </div>
                    </div>
                    <div className="mt-2 flex items-center justify-between border-t border-blue-500/10 pt-3">
                      <p className="text-muted-foreground text-xs">
                        Last Report: {item.lastEarningsDate || 'N/A'}
                      </p>
                      <a
                        href={`https://www.tradingview.com/symbols/${item.symbol}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-1 text-xs text-blue-500 transition-colors group-hover:gap-2 hover:text-blue-400"
                      >
                        View Date{' '}
                        <span className="transition-transform group-hover:translate-x-1">
                          →
                        </span>
                      </a>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
      </div>
    </div>
  );
}
