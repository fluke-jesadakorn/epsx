'use client';

import React from 'react';

import { Card, CardHeader, CardContent } from '@epsx/ui';

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
      className={`flex flex-col gap-8 w-full ${className || ''}`}
      style={style}
    >
      {/* Enhanced Section Header */}
      <div className="text-center space-y-4 mb-6 animate-slide-up">
        <h2 className="text-3xl sm:text-4xl font-bold pancake-gradient-text">
          Top Performing Companies
        </h2>
        <p className="text-muted-foreground max-w-2xl mx-auto">
          Discover the data leaders with exceptional growth and performance
          metrics
        </p>
        <div className="w-24 h-1 pancake-gradient mx-auto rounded-full" />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full animate-slide-up-delayed">
        {safeData
          .filter(
            (item) =>
              item && typeof item === 'object' && item.symbol && item.name,
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
                      className={`
                        font-bold text-xs px-3 py-1.5 rounded-full shadow-lg group-hover:scale-110 transition-transform duration-300
                        ${index === 0 ? 'bg-gradient-to-r from-yellow-400 to-yellow-500 text-yellow-900' : ''}
                        ${index === 1 ? 'bg-gradient-to-r from-gray-400 to-gray-500 text-gray-900' : ''}
                        ${index === 2 ? 'bg-gradient-to-r from-amber-400 to-amber-500 text-amber-900' : ''}
                      `}
                    >
                      #{index + 1}
                    </span>
                  </div>
                )}

                <CardHeader className="pb-4 relative">
                  <div className="flex flex-col space-y-3">
                    <div className="flex justify-between items-start">
                      <span
                        className={`${getMarketColor(item.exchange)} font-semibold px-3 py-1.5 rounded-full bg-secondary border border-secondary group-hover:scale-105 transition-transform duration-300 text-sm`}
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
                <CardContent className="group-hover:bg-gradient-to-br from-blue-500/5 to-purple-500/5 transition-colors duration-300">
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <div className="flex justify-between items-center">
                        <p className="text-sm text-muted-foreground">Signal</p>
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

                      <div className="flex justify-between items-center">
                        <p className="text-sm text-muted-foreground">Growth</p>
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

                      <div className="flex justify-between items-center">
                        <p className="text-sm text-muted-foreground">Status</p>
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
                    <div className="pt-3 mt-2 border-t border-blue-500/10 flex justify-between items-center">
                      <p className="text-xs text-muted-foreground">
                        Last Report: {item.lastEarningsDate || 'N/A'}
                      </p>
                      <a
                        href={`https://www.tradingview.com/symbols/${item.symbol}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-xs text-blue-500 hover:text-blue-400 transition-colors flex items-center gap-1 group-hover:gap-2"
                      >
                        View Chart{' '}
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
