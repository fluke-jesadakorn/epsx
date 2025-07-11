'use client';

import { ChevronDown, ChevronUp, LayoutGrid, Table2 } from 'lucide-react';
import React, { useState, useEffect } from 'react';

import { Card, CardContent } from '@/components/ui/card';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

import { Button } from '../ui/button';

import type { TableDataMetrics } from '@/types/stockFetchData';

interface ColumnDef {
  key: keyof TableDataMetrics | 'number' | 'chart';
  header: string;
  tooltip?: string;
  render?: (row: TableDataMetrics, index: number) => React.ReactNode;
}

interface DataRankTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: TableDataMetrics[];
  columns?: ColumnDef[];
  defaultView?: 'table' | 'card';
  rankingLimit?: number; // max items to display, from user profile
}

const defaultColumns: ColumnDef[] = [
  { key: 'number', header: 'No.' },
  { key: 'symbol', header: 'Symbol' },
  { key: 'name', header: 'Name' },
  {
    key: 'growthRate',
    header: 'Growth Rate',
    tooltip: 'Value Change Percentage',
  },
  {
    key: 'marketSize',
    header: 'Market Size',
    tooltip: 'Total Market Presence',
  },
  { key: 'sector', header: 'Sector' },
  { key: 'country', header: 'Country' },
  { key: 'exchange', header: 'Exchange' },
  { key: 'entryPhase', header: 'Entry Phase', tooltip: 'Optimal Entry Time' },
  {
    key: 'phaseStatus',
    header: 'Phase Status',
    tooltip: 'Current Phase Status',
  },
  {
    key: 'metricScore',
    header: 'Metric Score',
    tooltip: 'Current Metric Score',
  },
  {
    key: 'growthIndicator',
    header: 'Growth Indicator',
    tooltip: 'Growth Potential',
  },
  {
    key: 'currentMetric',
    header: 'Current Metric',
    tooltip: 'Current Metric Value',
  },
  {
    key: 'predictedMetric',
    header: 'Predicted Metric',
    tooltip: 'Predicted Metric Value',
  },
  {
    key: 'lastAnalysisDate',
    header: 'Last Analysis',
    tooltip: 'Date of Last Analysis',
  },
  {
    key: 'nextAnalysisDate',
    header: 'Next Analysis',
    tooltip: 'Date of Next Analysis',
  },
  { key: 'chart', header: 'Analytics', tooltip: 'Open Analytics View' },
];

interface DataCardProps {
  data: TableDataMetrics;
  index: number;
}

function DataCard({ data, index }: DataCardProps): React.JSX.Element {
  const [expanded, setExpanded] = useState(false);
  const [isPressed, setIsPressed] = useState(false);

  return (
    <Card
      className={`w-full transition-all duration-200 hover:shadow-2xl border-0 bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-[#232946] dark:via-[#1a1a2e] dark:to-[#0f1021] rounded-3xl shadow-lg ${
        isPressed ? 'scale-[0.98] opacity-90' : ''
      } relative`}
      onTouchStart={() => setIsPressed(true)}
      onTouchEnd={() => setIsPressed(false)}
      onMouseDown={() => setIsPressed(true)}
      onMouseUp={() => setIsPressed(false)}
      onMouseLeave={() => setIsPressed(false)}
    >
      {/* Rank Number Badge */}
      <div className="absolute top-4 w-auto left-4 z-10 h-10 rounded-full bg-gradient-to-br from-yellow-400 via-pink-400 to-purple-500 dark:from-yellow-600 dark:via-pink-700 dark:to-purple-800 flex items-center justify-center text-white text-base font-extrabold shadow-xl border-4 border-white dark:border-[#232946]">
        {index + 1}
      </div>
      <CardContent className="p-6 pt-8">
        {/* Primary Information */}
        <div className="flex justify-between items-start mb-3">
          <div>
            <div className="text-xl font-extrabold text-primary dark:text-white drop-shadow-sm tracking-wide">
              {data.symbol}
            </div>
            <div className="text-sm text-muted-foreground line-clamp-1 font-medium">
              {data.name}
            </div>
          </div>
          <div className="text-right">
            <div className="font-bold text-lg text-blue-600 dark:text-blue-300 drop-shadow">
              {data.valueIndex} {data.currency}
            </div>
            <div
              className={`text-sm font-bold ${
                parseFloat(data.growthRate) >= 0
                  ? 'text-green-500'
                  : 'text-rose-400 dark:text-rose-300'
              }`}
            >
              {parseFloat(data.growthRate) >= 0 ? '+' : ''}
              {data.growthRate}%
            </div>
          </div>
        </div>

        {/* Market Size - Always visible */}
        <div className="text-xs sm:text-sm text-muted-foreground mb-2 flex items-center gap-2">
          <span className="font-semibold text-purple-500 dark:text-purple-300">
            Market Size:
          </span>
          <span className="font-bold">{data.marketSize}</span>
        </div>

        {/* Action Buttons Row */}
        <div className="flex justify-between items-center gap-2 mt-3">
          <Button
            size="sm"
            variant="ghost"
            className="w-full rounded-full bg-gradient-to-r from-blue-100 to-purple-100 dark:from-[#232946] dark:to-[#1a1a2e] text-primary dark:text-white font-bold shadow hover:scale-105 transition"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? (
              <ChevronUp className="h-4 w-4 mr-1" />
            ) : (
              <ChevronDown className="h-4 w-4 mr-1" />
            )}
            {expanded ? 'Less' : 'More'}
          </Button>
          <Button
            asChild
            size="sm"
            variant="secondary"
            className="w-[100px] rounded-full bg-gradient-to-r from-yellow-300 to-pink-300 dark:from-yellow-700 dark:to-pink-700 text-white font-bold shadow hover:scale-105 transition"
          >
            <a
              href={`https://www.tradingview.com/chart?symbol=${data.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              Analytics
            </a>
          </Button>
        </div>

        {/* Expandable Content */}
        <div
          className={`mt-4 pt-4 border-t border-border/50 grid gap-3 text-xs sm:text-sm overflow-hidden transition-all duration-300 ${
            expanded ? 'opacity-100 max-h-[500px]' : 'opacity-0 max-h-0'
          }`}
        >
          <div className="grid grid-cols-2 gap-2">
            <div>
              <div className="text-muted-foreground font-semibold">Sector</div>
              <div className="font-bold">{data.sector}</div>
            </div>
            <div>
              <div className="text-muted-foreground font-semibold">
                Growth Factor
              </div>
              <div className="font-bold">{data.growthFactor}</div>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <div className="text-muted-foreground mb-1 font-semibold">
                Entry Phase
              </div>
              <div
                className={`${data.entryPhase?.active ? 'text-green-500 font-bold' : ''}`}
              >
                {data.entryPhase?.date || 'N/A'}
                {data.entryPhase?.active && (
                  <span className="block text-xs">Active</span>
                )}
              </div>
            </div>
            <div>
              <div className="text-muted-foreground mb-1 font-semibold">
                Phase Status
              </div>
              <div
                className={`${data.phaseStatus?.active ? 'text-yellow-500 font-bold' : ''}`}
              >
                {data.phaseStatus?.date || 'N/A'}
                {data.phaseStatus?.active && (
                  <span className="block text-xs">
                    {data.phaseStatus.type === 'monitor' ? 'Monitor' : 'Exit'}
                  </span>
                )}
              </div>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div>
              <div className="text-muted-foreground font-semibold">Country</div>
              <div className="font-bold">{data.country}</div>
            </div>
            <div>
              <div className="text-muted-foreground font-semibold">
                Exchange
              </div>
              <div className="font-bold">{data.exchange}</div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function DataRankTable({
  style,
  className,
  data,
  columns = defaultColumns,
  defaultView = 'card',
  rankingLimit,
}: DataRankTableProps): React.JSX.Element {
  // Ensure data is always an array to prevent runtime errors
  let safeData = Array.isArray(data) ? data : [];
  if (typeof rankingLimit === 'number' && rankingLimit > 0) {
    safeData = safeData.slice(0, rankingLimit);
  }
  const [viewMode, setViewMode] = useState<'table' | 'card'>(defaultView);
  const [isMobile, setIsMobile] = useState(false);
  const [isNarrow, setIsNarrow] = useState(false);

  useEffect(() => {
    const checkScreenSize = () => {
      const width = typeof window !== 'undefined' ? window.innerWidth : 0;
      setIsMobile(width < 768);
      setIsNarrow(width < 1200);
      // Only apply responsive switching for table view
      if (defaultView === 'table') {
        setViewMode(width < 768 ? 'card' : 'table');
      }
    };

    checkScreenSize();

    if (typeof window !== 'undefined') {
      window.addEventListener('resize', checkScreenSize);
      return () => window.removeEventListener('resize', checkScreenSize);
    }
  }, [defaultView]);

  const renderTableView = () => (
    <div className="relative w-full">
      <div className="rounded-3xl border-0 bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-[#232946] dark:via-[#1a1a2e] dark:to-[#0f1021] shadow-xl max-w-[100vw] overflow-x-auto custom-scrollbar">
        <div className="min-w-[1200px]">
          <Table>
            <TableHeader className="bg-gradient-to-r from-blue-200/40 via-purple-200/40 to-pink-200/40 dark:from-blue-900/40 dark:via-purple-900/40 dark:to-pink-900/40 backdrop-blur-sm sticky top-0 z-10">
              <TableRow className="hover:bg-transparent">
                {columns.map((column) => (
                  <TableHead
                    key={column.key}
                    className="font-bold text-primary/80 dark:text-white tracking-wide"
                  >
                    {column.tooltip ? (
                      <span title={column.tooltip}>{column.header}</span>
                    ) : (
                      column.header
                    )}
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {safeData.map((row, index) => (
                <TableRow
                  key={`${row.symbol}-${index}`}
                  className="hover:bg-blue-100/40 dark:hover:bg-blue-900/20 transition-colors"
                >
                  {columns.map((column) => (
                    <TableCell
                      key={column.key}
                      className={
                        column.key === 'symbol' ||
                        column.key === 'name' ||
                        column.key === 'valueIndex'
                          ? 'font-bold'
                          : column.key === 'sector' ||
                              column.key === 'country' ||
                              column.key === 'exchange'
                            ? 'text-muted-foreground'
                            : ''
                      }
                    >
                      {renderCell(row, column, index)}
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>
      {isNarrow && (
        <div className="mt-2 text-sm text-muted-foreground text-center">
          Scroll horizontally to see more →
        </div>
      )}
    </div>
  );

  const renderCardView = () => (
    <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
      {safeData.map((item, index) => (
        <DataCard key={`${item.symbol}-${index}`} data={item} index={index} />
      ))}
    </div>
  );

  // Render cell content for table view
  const renderCell = (
    row: TableDataMetrics,
    column: ColumnDef,
    index: number,
  ) => {
    if (column.render) {
      return column.render(row, index);
    }

    switch (column.key) {
      case 'number':
        return index + 1;
      // valueIndex removed
      case 'growthRate':
        const growthRate = parseFloat(row.growthRate || '0');
        return (
          <span
            className={`font-medium ${growthRate >= 0 ? 'text-green-500' : 'text-rose-500'}`}
          >
            {growthRate >= 0 ? '+' : ''}
            {!isNaN(growthRate) ? `${growthRate.toFixed(2)}%` : 'N/A'}
          </span>
        );
      // activityScore removed
      case 'marketSize':
        return row.marketSize || 'N/A';
      // growthFactor removed
      case 'sector':
        return row.sector || 'N/A';
      case 'country':
        return row.country || 'N/A';
      case 'exchange':
        return row.exchange || 'N/A';
      case 'entryPhase':
        return (
          <div className="flex flex-col gap-1">
            <span
              className={`text-xs ${row.entryPhase?.active ? 'text-green-500 font-medium' : 'text-muted-foreground'}`}
            >
              {row.entryPhase?.date || 'N/A'}
            </span>
            {row.entryPhase?.active && (
              <span className="text-xs text-green-500 font-medium">Active</span>
            )}
          </div>
        );
      case 'phaseStatus':
        return (
          <div className="flex flex-col gap-1">
            <span
              className={`text-xs ${row.phaseStatus?.active ? 'text-yellow-500 font-medium' : 'text-muted-foreground'}`}
            >
              {row.phaseStatus?.date || 'N/A'}
            </span>
            {row.phaseStatus?.active && (
              <span className="text-xs text-yellow-500 font-medium">
                {row.phaseStatus?.type === 'monitor' ? 'Monitor' : 'Exit'}
              </span>
            )}
          </div>
        );
      case 'metricScore':
        return row.metricScore || 'N/A';
      case 'growthIndicator':
        return row.growthIndicator || 'N/A';
      case 'currentMetric':
        return row.currentMetric || 'N/A';
      case 'predictedMetric':
        return row.predictedMetric || 'N/A';
      case 'lastAnalysisDate':
        return row.lastAnalysisDate || 'N/A';
      case 'nextAnalysisDate':
        return row.nextAnalysisDate || 'N/A';
      case 'chart':
        return (
          <Button asChild size="sm" variant="secondary">
            <a
              href={`https://www.tradingview.com/chart?symbol=${row.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              View
            </a>
          </Button>
        );
      default:
        const value = row[column.key as keyof TableDataMetrics];
        if (typeof value === 'string') {
          return value;
        }
        return null;
    }
  };

  return (
    <div
      className={`w-full space-y-6 p-4 sm:p-8 ${className || ''}`}
      style={style}
    >
      {/* Title and View Toggle */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-8">
        <div>
          <h2 className="text-3xl font-extrabold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent drop-shadow">
            Top Data Rankings
          </h2>
          <div className="w-24 h-1 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 rounded-full mt-2" />
        </div>

        {/* Only show view toggle for table default view */}
        {!isMobile && defaultView === 'table' && (
          <div className="flex items-center gap-2">
            <Button
              variant={viewMode === 'table' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setViewMode('table')}
              className="w-[100px] rounded-full font-bold shadow"
            >
              <Table2 className="h-4 w-4 mr-2" />
              Table
            </Button>
            <Button
              variant={viewMode === 'card' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setViewMode('card')}
              className="w-[100px] rounded-full font-bold shadow"
            >
              <LayoutGrid className="h-4 w-4 mr-2" />
              Cards
            </Button>
          </div>
        )}
      </div>

      {/* Content based on view mode */}
      <div className="-mx-4 sm:-mx-8 px-4 sm:px-8">
        {viewMode === 'table' ? renderTableView() : renderCardView()}
      </div>
    </div>
  );
}

export default DataRankTable;
