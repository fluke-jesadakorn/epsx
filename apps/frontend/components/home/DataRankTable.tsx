'use client';

import { Card, CardContent } from '@/components/ui/card';
import {
  ChevronDown,
  ChevronUp,
  Crown,
  LayoutGrid,
  Lock,
  Table2,
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import React, { useEffect, useState } from 'react';

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

import type { PermissionTemplateName } from '@/app/constants/packages';
import { LockedRankingCard, UpgradePrompt } from '@/components/ui/prompt';
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
  userPermissions?: string[]; // User permissions for ranking access control
  rankingLimit?: number; // @deprecated Use userPermissions instead - max items to display, from user profile
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
      className={`w-full rounded-3xl border-0 bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 shadow-lg transition-all duration-200 hover:shadow-2xl dark:from-[#232946] dark:via-[#1a1a2e] dark:to-[#0f1021] ${
        isPressed ? 'scale-[0.98] opacity-90' : ''
      } relative`}
      onTouchStart={() => setIsPressed(true)}
      onTouchEnd={() => setIsPressed(false)}
      onMouseDown={() => setIsPressed(true)}
      onMouseUp={() => setIsPressed(false)}
      onMouseLeave={() => setIsPressed(false)}
    >
      {/* Rank Number Badge */}
      <div className="absolute top-4 left-4 z-10 flex h-10 w-auto items-center justify-center rounded-full border-4 border-white bg-gradient-to-br from-yellow-400 via-pink-400 to-purple-500 text-base font-extrabold text-white shadow-xl dark:border-[#232946] dark:from-yellow-600 dark:via-pink-700 dark:to-purple-800">
        {index + 1}
      </div>
      <CardContent className="p-6 pt-8">
        {/* Primary Information */}
        <div className="mb-3 flex items-start justify-between">
          <div>
            <div className="text-primary text-xl font-extrabold tracking-wide drop-shadow-sm dark:text-white">
              {data.symbol}
            </div>
            <div className="text-muted-foreground line-clamp-1 text-sm font-medium">
              {data.name}
            </div>
          </div>
          <div className="text-right">
            <div className="text-lg font-bold text-blue-600 drop-shadow dark:text-blue-300">
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
        <div className="text-muted-foreground mb-2 flex items-center gap-2 text-xs sm:text-sm">
          <span className="font-semibold text-purple-500 dark:text-purple-300">
            Market Size:
          </span>
          <span className="font-bold">{data.marketSize}</span>
        </div>

        {/* Action Buttons Row */}
        <div className="mt-3 flex items-center justify-between gap-2">
          <Button
            size="sm"
            variant="ghost"
            className="text-primary flex w-full items-center gap-1 rounded-full bg-gradient-to-r from-blue-100 to-purple-100 font-bold shadow transition hover:scale-105 dark:from-[#232946] dark:to-[#1a1a2e] dark:text-white"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? (
              <ChevronUp className="h-4 w-4" />
            ) : (
              <ChevronDown className="h-4 w-4" />
            )}
            {expanded ? 'Less' : 'More'}
          </Button>
          <Button
            asChild
            size="sm"
            variant="secondary"
            className="w-[100px] rounded-full bg-gradient-to-r from-yellow-300 to-pink-300 font-bold text-white shadow transition hover:scale-105 dark:from-yellow-700 dark:to-pink-700"
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
          className={`border-border/50 mt-4 grid gap-3 overflow-hidden border-t pt-4 text-xs transition-all duration-300 sm:text-sm ${
            expanded ? 'max-h-[500px] opacity-100' : 'max-h-0 opacity-0'
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
                className={`${data.entryPhase?.active ? 'font-bold text-green-500' : ''}`}
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
                className={`${data.phaseStatus?.active ? 'font-bold text-yellow-500' : ''}`}
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
  userPermissions = ['epsx:rankings:view:5'], // Default to basic access
  rankingLimit, // Keep for backward compatibility
}: DataRankTableProps): React.JSX.Element {
  const router = useRouter();

  // Helper function to calculate next tier limit for upgrade prompts
  const getNextTierLimit = (currentTier: PermissionTemplateName): number => {
    const tierLimits: Record<PermissionTemplateName, number> = {
      'Free Template': 3,
      'Bronze Template': 5, 
      'Silver Template': 25, 
      'Gold Template': 50, 
      'Platinum Template': -1, // Unlimited
      'Enterprise Template': -1, // Unlimited
    };
    return tierLimits[currentTier] || 25;
  };

  // Helper function to calculate locked rankings for upgrade prompts
  const getLockedRankingsCount = (currentLimit: number): number => {
    if (currentLimit === -1) return 0; // Unlimited access
    const nextLimit = getNextTierLimit(userLevel);
    if (nextLimit === -1) return 50; // Show some locked content for unlimited upgrade
    return Math.min(nextLimit - currentLimit, 10); // Cap at 10 locked rankings
  };

  // Simplified access control - backend will handle permissions
  const maxRankings = -1; // Unlimited access - backend will handle actual restrictions
  const userLevel = 'basic' as PermissionTemplateName;
  const upgradeRequired = false; // No frontend upgrade prompts
  const canViewRanking = (index: number) => true; // Backend will handle actual access control

  // Ensure data is always an array to prevent runtime errors
  let safeData = Array.isArray(data) ? data : [];

  // Apply user-based ranking limit (with backward compatibility)
  const effectiveLimit = rankingLimit
    ? Math.min(rankingLimit, maxRankings === -1 ? 1000 : maxRankings) // Handle unlimited case
    : (maxRankings === -1 ? safeData.length : maxRankings);

  safeData = safeData.slice(0, effectiveLimit);

  const [viewMode, setViewMode] = useState<'table' | 'card'>(defaultView);
  const [isMobile, setIsMobile] = useState(false);
  const [isNarrow, setIsNarrow] = useState(false);

  const handleUpgrade = () => {
    router.push('/payment');
  };

  useEffect(() => {
    const checkScreenSize = () => {
      const width = typeof window !== 'undefined' ? window.innerWidth : 0;
      setIsMobile(width < 768);
      setIsNarrow(width < 1200);
      // Always apply responsive switching - prioritize cards on mobile for better UX
      setViewMode(width < 768 ? 'card' : defaultView);
    };

    checkScreenSize();

    if (typeof window !== 'undefined') {
      window.addEventListener('resize', checkScreenSize);
      return () => window.removeEventListener('resize', checkScreenSize);
    }

    return () => {}; // Empty cleanup function for consistency
  }, [defaultView]);

  const renderTableView = () => (
    <div className="relative w-full">
      <div className="custom-scrollbar max-w-[100vw] overflow-x-auto rounded-3xl border-0 bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 shadow-xl dark:from-[#232946] dark:via-[#1a1a2e] dark:to-[#0f1021]">
        <div className="min-w-[1200px]">
          <Table>
            <TableHeader className="sticky top-0 z-10 bg-gradient-to-r from-blue-200/40 via-purple-200/40 to-pink-200/40 backdrop-blur-sm dark:from-blue-900/40 dark:via-purple-900/40 dark:to-pink-900/40">
              <TableRow className="hover:bg-transparent">
                {columns.map(column => (
                  <TableHead
                    key={column.key}
                    className="text-primary/80 font-bold tracking-wide dark:text-white"
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
                  className={`transition-colors hover:bg-blue-100/40 dark:hover:bg-blue-900/20 ${
                    !canViewRanking(index)
                      ? 'pointer-events-none opacity-50'
                      : ''
                  }`}
                >
                  {columns.map(column => (
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
                      {canViewRanking(index) ? (
                        renderCell(row, column, index)
                      ) : (
                        <div className="flex items-center gap-2">
                          <Lock className="h-4 w-4 text-gray-400" />
                          <span className="text-gray-400">Locked</span>
                        </div>
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))}

              {/* Show locked rows for premium content */}
              {upgradeRequired &&
                Array.from({ length: Math.min(3, 15 - safeData.length) }).map(
                  (_, index) => (
                    <TableRow
                      key={`locked-row-${index}`}
                      className="pointer-events-none bg-gray-50 opacity-50 dark:bg-gray-900/50"
                    >
                      {columns.map(column => (
                        <TableCell key={column.key}>
                          <div className="flex items-center gap-2">
                            <Lock className="h-4 w-4 text-gray-400" />
                            <span className="text-gray-400">Locked</span>
                          </div>
                        </TableCell>
                      ))}
                    </TableRow>
                  )
                )}
            </TableBody>
          </Table>

          {/* Table upgrade prompt */}
          {upgradeRequired && (
            <div className="border-t bg-gradient-to-r from-blue-50 to-purple-50 p-6 dark:from-blue-950/50 dark:to-purple-950/50">
              <div className="space-y-3 text-center">
                <Crown className="mx-auto h-8 w-8 text-yellow-500" />
                <h3 className="text-lg font-semibold">Unlock More Rankings</h3>
                <p className="text-muted-foreground text-sm">
                  Upgrade to see up to {getNextTierLimit(userLevel)} top-ranked
                  stocks
                </p>
                <Button className="gap-2" onClick={handleUpgrade}>
                  <Crown className="h-4 w-4" />
                  Upgrade Now
                </Button>
              </div>
            </div>
          )}
        </div>
      </div>
      {isNarrow && (
        <div className="text-muted-foreground mt-2 text-center text-sm">
          Scroll horizontally to see more →
        </div>
      )}
    </div>
  );

  const renderCardView = () => (
    <div className="space-y-6">
      <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {safeData.map((item, index) =>
          canViewRanking(index) ? (
            <DataCard
              key={`${item.symbol}-${index}`}
              data={item}
              index={index}
            />
          ) : (
            <LockedRankingCard
              key={`locked-${index}`}
              index={index}
              userLevel={userLevel}
              onUpgrade={handleUpgrade}
            />
          )
        )}
        {/* Show additional locked cards for premium tiers */}
        {upgradeRequired &&
          Array.from({ length: Math.min(5, 20 - safeData.length) }).map(
            (_, index) => (
              <LockedRankingCard
                key={`extra-locked-${index}`}
                index={safeData.length + index}
                userLevel={userLevel}
                onUpgrade={handleUpgrade}
              />
            )
          )}
      </div>

      {/* Upgrade prompt at bottom */}
      {upgradeRequired && (
        <UpgradePrompt
          currentLevel={userLevel}
          lockedRankings={getLockedRankingsCount(maxRankings)}
          className="mt-6"
        />
      )}
    </div>
  );

  // Render cell content for table view
  const renderCell = (
    row: TableDataMetrics,
    column: ColumnDef,
    index: number
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
              className={`text-xs ${row.entryPhase?.active ? 'font-medium text-green-500' : 'text-muted-foreground'}`}
            >
              {row.entryPhase?.date || 'N/A'}
            </span>
            {row.entryPhase?.active && (
              <span className="text-xs font-medium text-green-500">Active</span>
            )}
          </div>
        );
      case 'phaseStatus':
        return (
          <div className="flex flex-col gap-1">
            <span
              className={`text-xs ${row.phaseStatus?.active ? 'font-medium text-yellow-500' : 'text-muted-foreground'}`}
            >
              {row.phaseStatus?.date || 'N/A'}
            </span>
            {row.phaseStatus?.active && (
              <span className="text-xs font-medium text-yellow-500">
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
      <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-3xl font-extrabold text-transparent drop-shadow">
            Top Data Rankings
          </h2>
          <div className="mt-2 h-1 w-24 rounded-full bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500" />
        </div>

        {/* Only show view toggle for table default view */}
        {!isMobile && defaultView === 'table' && (
          <div className="flex items-center gap-2">
            <Button
              variant={viewMode === 'table' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setViewMode('table')}
              className="flex w-[100px] items-center gap-2 rounded-full font-bold shadow"
            >
              <Table2 className="h-4 w-4" />
              Table
            </Button>
            <Button
              variant={viewMode === 'card' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setViewMode('card')}
              className="flex w-[100px] items-center gap-2 rounded-full font-bold shadow"
            >
              <LayoutGrid className="h-4 w-4" />
              Cards
            </Button>
          </div>
        )}
      </div>

      {/* Content based on view mode */}
      <div className="-mx-4 px-4 sm:-mx-8 sm:px-8">
        {viewMode === 'table' ? renderTableView() : renderCardView()}
      </div>
    </div>
  );
}

export default DataRankTable;
