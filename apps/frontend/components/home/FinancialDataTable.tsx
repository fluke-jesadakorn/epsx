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

import type { StockFinancialData } from '@/types/financialChartData';
import { 
  getLatestQuarterData, 
  getLatestQuarterWithGrowth,
  calculateAverageEpsGrowth, 
  formatPrice, 
  formatEpsGrowth, 
  formatDate 
} from '@/utils/transformers/stockDataTransformer';

interface FinancialColumnDef {
  key: 'number' | 'symbol' | 'latestPrice' | 'latestEps' | 'latestGrowth' | 'latestDate' | 'avgGrowth' | 'quarters';
  header: string;
  tooltip?: string;
  render?: (row: StockFinancialData, index: number) => React.ReactNode;
}

interface FinancialDataTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: StockFinancialData[];
  columns?: FinancialColumnDef[];
  defaultView?: 'table' | 'card';
}

const defaultColumns: FinancialColumnDef[] = [
  { key: 'number', header: 'No.' },
  { key: 'symbol', header: 'Symbol' },
  { key: 'latestPrice', header: 'Latest Price' },
  { key: 'latestEps', header: 'Latest EPS' },
  { key: 'latestGrowth', header: 'EPS Growth %', tooltip: 'Latest quarter EPS growth percentage' },
  { key: 'latestDate', header: 'Latest Date' },
  { key: 'avgGrowth', header: 'Avg Growth %', tooltip: 'Average EPS growth across all quarters' },
  { key: 'quarters', header: 'Historical Data', tooltip: 'View quarterly financial data' },
];

interface FinancialDataCardProps {
  data: StockFinancialData;
  index: number;
}

function FinancialDataCard({ data, index }: FinancialDataCardProps): React.JSX.Element {
  const [expanded, setExpanded] = useState(false);
  const [isPressed, setIsPressed] = useState(false);
  const latestQuarter = getLatestQuarterData(data);
  const avgGrowth = calculateAverageEpsGrowth(data);

  return (
    <Card
      className={`w-full transition-all duration-200 hover:shadow-2xl border-0 bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-[#232946] dark:via-[#1a1a2e] dark:to-[#0f1021] rounded-3xl shadow-lg relative ${
        isPressed ? 'scale-[0.98] opacity-90' : ''
      }`}
      onTouchStart={() => setIsPressed(true)}
      onTouchEnd={() => setIsPressed(false)}
      onMouseDown={() => setIsPressed(true)}
      onMouseUp={() => setIsPressed(false)}
      onMouseLeave={() => setIsPressed(false)}
    >
      {/* Rank Number Badge */}
      <div className="absolute -top-3 -left-3 w-10 h-10 rounded-full bg-gradient-to-br from-yellow-400 via-pink-400 to-purple-500 dark:from-yellow-600 dark:via-pink-700 dark:to-purple-800 flex items-center justify-center text-white text-base font-extrabold shadow-xl border-4 border-white dark:border-[#232946]">
        {index + 1}
      </div>
      <CardContent className="p-6 pt-8">
        {/* Unified Data Fields (match table) */}
        <div className="grid grid-cols-2 gap-2 mb-3">
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Symbol</div>
            <div className="text-xl font-extrabold text-primary dark:text-white drop-shadow-sm tracking-wide">
              {data.symbol}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Latest Date</div>
            <div className="text-sm font-medium">
              {latestQuarter?.date ? formatDate(latestQuarter.date) : 'N/A'}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Latest Price</div>
            <div className="font-bold text-lg text-blue-600 dark:text-blue-300 drop-shadow">
              {(() => {
                // Use current price if available, otherwise fall back to latest quarter price
                const priceToShow = data.currentPrice !== undefined && data.currentPrice !== null 
                  ? data.currentPrice 
                  : latestQuarter?.price;
                return priceToShow !== undefined && priceToShow !== null ? formatPrice(priceToShow) : 'N/A';
              })()}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Latest EPS</div>
            <div className="font-bold">
              {latestQuarter?.eps !== undefined ? latestQuarter.eps.toFixed(4) : 'N/A'}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">EPS Growth %</div>
            <div
              className={`font-bold ${
                (latestQuarter?.eps_growth || 0) >= 0
                  ? 'text-green-500'
                  : 'text-rose-400 dark:text-rose-300'
              }`}
            >
              {latestQuarter?.eps_growth !== undefined ? formatEpsGrowth(latestQuarter.eps_growth) : 'N/A'}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Avg Growth %</div>
            <div
              className={`font-bold ${(avgGrowth || 0) >= 0 ? 'text-green-500' : 'text-rose-500'}`}
            >
              {formatEpsGrowth(avgGrowth)}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Quarters</div>
            <div className="font-bold">{data.quarters.length}</div>
          </div>
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
              <div className="text-muted-foreground font-semibold">Avg Growth</div>
              <div className="font-bold">{formatEpsGrowth(avgGrowth)}</div>
            </div>
            <div>
              <div className="text-muted-foreground font-semibold">
                Quarters
              </div>
              <div className="font-bold">{data.quarters.length}</div>
            </div>
          </div>

          <div className="space-y-2">
            <div className="text-muted-foreground font-semibold">Historical Data</div>
            {data.quarters.map((quarter, idx) => (
              <div key={idx} className="flex justify-between items-center text-xs bg-white/50 dark:bg-black/20 rounded-lg p-2">
                <span>Q{quarter.quarter} {quarter?.date ? formatDate(quarter.date) : 'N/A'}</span>
                <div className="text-right">
                  <div>{quarter?.price !== undefined ? formatPrice(quarter.price) : 'N/A'}</div>
                  <div className={`${(quarter?.eps_growth || 0) >= 0 ? 'text-green-500' : 'text-rose-400'}`}>
                    {quarter?.eps_growth !== undefined ? formatEpsGrowth(quarter.eps_growth) : 'N/A'}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function FinancialDataTable({
  style,
  className,
  data,
  columns = defaultColumns,
  defaultView = 'card',
}: FinancialDataTableProps): React.JSX.Element {
  // Ensure data is always an array to prevent runtime errors
  const safeData = Array.isArray(data) ? data : [];
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
                        column.key === 'symbol' ? 'font-bold' : ''
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
        <FinancialDataCard key={`${item.symbol}-${index}`} data={item} index={index} />
      ))}
    </div>
  );

  // Render cell content for table view
  const renderCell = (
    row: StockFinancialData,
    column: FinancialColumnDef,
    index: number,
  ) => {
    if (column.render) {
      return column.render(row, index);
    }

    const latestQuarter = getLatestQuarterData(row);
    const latestQuarterWithGrowth = getLatestQuarterWithGrowth(row);
    const avgGrowth = calculateAverageEpsGrowth(row);

    switch (column.key) {
      case 'number':
        return index + 1;
      case 'symbol':
        return row.symbol;
      case 'latestPrice':
        // Use current price if available, otherwise fall back to latest quarter price
        const priceToShow = row.currentPrice !== undefined && row.currentPrice !== null 
          ? row.currentPrice 
          : latestQuarter?.price;
        return priceToShow !== undefined && priceToShow !== null ? formatPrice(priceToShow) : 'N/A';
      case 'latestEps':
        return latestQuarter?.eps !== undefined ? latestQuarter.eps.toFixed(4) : 'N/A';
      case 'latestGrowth':
        return (
          <span
            className={`font-medium ${(latestQuarterWithGrowth?.eps_growth || 0) >= 0 ? 'text-green-500' : 'text-rose-500'}`}
          >
            {latestQuarterWithGrowth?.eps_growth !== undefined ? formatEpsGrowth(latestQuarterWithGrowth.eps_growth) : 'N/A'}
          </span>
        );
      case 'latestDate':
        return latestQuarter?.date ? formatDate(latestQuarter.date) : 'N/A';
      case 'avgGrowth':
        return (
          <span
            className={`font-medium ${(avgGrowth || 0) >= 0 ? 'text-green-500' : 'text-rose-500'}`}
          >
            {formatEpsGrowth(avgGrowth)}
          </span>
        );
      case 'quarters':
        return (
          <Button asChild size="sm" variant="secondary">
            <a
              href={`https://www.tradingview.com/chart?symbol=${row.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              View Chart
            </a>
          </Button>
        );
      default:
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
            Financial Data Rankings
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

export default FinancialDataTable;
