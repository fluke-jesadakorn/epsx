import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Download, Filter, RefreshCw } from 'lucide-react';
import { formatCurrency } from '@/shared/utils/formatting/currency';

interface QuarterlyPerformanceData {
  quarter: string;
  date: string;
  price: number;
  eps: number;
  eps_growth: number;
  price_growth: number;
  is_estimated?: boolean;
}

interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  active_status: string;
  quarterly_performance: QuarterlyPerformanceData[];
  next_quarter_estimate?: {
    quarter: string;
    announcement_date: string;
    announcement_timestamp: number;
    days_until_announcement: number;
    estimated_eps: number;
    estimated_price_target?: number;
    confidence: string;
  };
  next_earnings_date?: number;
  last_earnings_date?: number;
  next_earnings_date_formatted?: string;
  days_until_next_earnings?: number;
  progress_percentage?: number;
}

interface AdvancedFilters {
  country: string;
  sector: string;
  min_eps: number | undefined;
  min_growth: number | undefined;
  sort_by: string;
  page: number;
  limit: number;
}

interface FilterOptions {
  countries: Array<{ value: string; label: string }>;
  sectors: string[];
}

const formatPercentage = (value: number) => {
  return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

export function SymbolCard({
  cardData,
  isOverlay = false,
}: {
  cardData: SymbolCardData;
  isOverlay?: boolean;
}) {
  const quarters = cardData.quarterly_performance.slice(0, 2);
  const latestQuarter = quarters[0] as QuarterlyPerformanceData | undefined;
  const previousQuarter = quarters[1] as QuarterlyPerformanceData | undefined;

  const getActionInfo = (status: string) => {
    switch (status) {
      case 'TRACK':
        return { action: 'KEEP', emoji: '🟢' };
      case 'STOP':
        return { action: 'PAUSE', emoji: '🔴' };
      default:
        return { action: 'KEEP', emoji: '🟢' };
    }
  };

  const actionInfo = getActionInfo(cardData.active_status);

  const nextDate = cardData.next_earnings_date_formatted ?? 'TBD';
  const daysUntil = cardData.days_until_next_earnings ?? 0;
  const progressPercentage = cardData.progress_percentage ?? 0;

  return (
    <div
      className={`mx-auto w-full max-w-sm touch-manipulation overflow-hidden rounded-3xl border-2 border-transparent bg-white shadow-2xl shadow-pink-500/20 transition-all duration-300 dark:bg-slate-900 dark:shadow-cyan-500/20 ${
        isOverlay
          ? 'scale-105 cursor-grabbing shadow-2xl'
          : 'hover:border-pink-200 dark:hover:border-cyan-400/50'
      }`}
    >
      <div className="bg-gradient-to-r from-pink-400 via-purple-500 to-indigo-600 px-6 py-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <span className="text-4xl font-black tracking-wide text-white drop-shadow-lg">
              {cardData.symbol}
            </span>
            <span className="text-2xl font-bold text-pink-100 opacity-80">
              #{cardData.rank}
            </span>
          </div>
          <a
            href={`https://www.tradingview.com/symbols/${cardData.symbol}`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xl text-white transition-transform hover:scale-125 hover:text-yellow-300"
          >
            🔗
          </a>
        </div>
      </div>

      <div className="bg-gradient-to-br from-pink-50 to-purple-50 py-8 text-center dark:from-slate-800 dark:to-slate-700">
        <div className="inline-flex items-center gap-3 rounded-full border border-pink-200/50 bg-white px-6 py-3 shadow-lg dark:border-cyan-400/30 dark:bg-slate-800">
          <span className="text-2xl">{actionInfo.emoji}</span>
          <span className="bg-gradient-to-r from-pink-600 to-purple-600 bg-clip-text text-2xl font-bold text-transparent dark:from-cyan-400 dark:to-blue-400">
            {actionInfo.action}
          </span>
        </div>
      </div>

      <div className="bg-gradient-to-br from-pink-50 to-purple-50 px-6 py-6 dark:from-slate-800 dark:to-slate-700">
        <div className="mb-2 text-center">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
            NEXT ACTION
          </span>
        </div>

        <div className="mb-4 text-center">
          <span className="text-5xl font-bold text-green-500 dark:text-green-400">
            {daysUntil}d
          </span>
        </div>

        <div className="h-3 w-full rounded-full bg-slate-200 dark:bg-slate-600">
          <div
            className="h-3 rounded-full bg-gradient-to-r from-green-400 to-green-500"
            style={{ width: `${progressPercentage}%` }}
          />
        </div>
      </div>

      {latestQuarter ? (
        <div className="bg-white py-6 text-center dark:bg-slate-900">
          <div
            className={`inline-flex items-center gap-2 rounded-2xl px-4 py-2 shadow-lg ${
              latestQuarter.eps_growth >= 0
                ? 'bg-gradient-to-r from-green-400 to-emerald-500'
                : 'bg-gradient-to-r from-red-400 to-red-500'
            }`}
          >
            <span className="text-2xl">
              {latestQuarter.eps_growth >= 0 ? '↗️' : '↘️'}
            </span>
            <span className="text-2xl font-bold text-white drop-shadow-sm">
              {formatPercentage(latestQuarter.eps_growth)}
            </span>
          </div>
        </div>
      ) : null}

      <div className="bg-white px-6 dark:bg-slate-900">
        <div className="border-t-2 border-dashed border-pink-200 dark:border-cyan-400/30" />
      </div>

      <div className="space-y-4 bg-white px-6 pt-6 pb-6 dark:bg-slate-900">
        {quarters.length >= 2 && previousQuarter != null && latestQuarter != null && (
          <>
            <div className="rounded-2xl border border-purple-200/50 bg-gradient-to-r from-purple-50 to-pink-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
              <div className="mb-2 text-sm font-semibold text-purple-600 dark:text-cyan-400">
                {previousQuarter.date || 'Apr 30, 2025'}
              </div>
              <div className="space-y-1 text-sm text-purple-700 dark:text-cyan-200">
                <div>
                  • Growth: {formatPercentage(previousQuarter.eps_growth || 0)} | EPS:{' '}
                  {(previousQuarter.eps || 0).toFixed(2)}
                </div>
                <div>
                  • Price: {formatPercentage(previousQuarter.price_growth || 0)} |{' '}
                  {formatCurrency(previousQuarter.price || 0)}
                </div>
              </div>
            </div>

            <div className="rounded-2xl border border-green-200/50 bg-gradient-to-r from-green-50 to-emerald-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
              <div className="mb-2 text-sm font-semibold text-green-600 dark:text-cyan-400">
                {latestQuarter.date || 'Jul 30, 2025'}
              </div>
              <div className="space-y-1 text-sm text-green-700 dark:text-cyan-200">
                <div>
                  • Growth: {formatPercentage(latestQuarter.eps_growth || 0)} | EPS:{' '}
                  {(latestQuarter.eps || 0).toFixed(2)}
                </div>
                <div>
                  • Price: {formatPercentage(latestQuarter.price_growth || 0)} |{' '}
                  {formatCurrency(latestQuarter.price || 0)}
                </div>
              </div>
            </div>
          </>
        )}

        <div className="rounded-2xl border border-amber-200/50 bg-gradient-to-r from-amber-50 to-orange-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
          <div className="text-sm font-semibold text-amber-600 dark:text-cyan-400">
            Next: {nextDate} {daysUntil > 0 && `(${daysUntil} days)`}
          </div>
        </div>

        <div className="mt-4">
          <a
            href={`https://www.tradingview.com/symbols/${cardData.symbol}`}
            target="_blank"
            rel="noopener noreferrer"
            className="block w-full"
          >
            <button className="group relative w-full overflow-hidden rounded-xl bg-gradient-to-r from-blue-600 to-blue-700 py-3 text-sm font-bold text-white transition-all duration-300 hover:shadow-lg hover:shadow-blue-500/25">
              <span className="relative flex items-center justify-center gap-2">
                View Details
                <span className="h-4 w-4 transition-transform group-hover:translate-x-1">
                  →
                </span>
              </span>
            </button>
          </a>
        </div>
      </div>
    </div>
  );
}

export function SortableSymbolCard({ cardData }: { cardData: SymbolCardData }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({ id: cardData.symbol });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
    zIndex: isDragging ? 50 : 'auto',
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners} className="touch-none">
      <SymbolCard cardData={cardData} />
    </div>
  );
}

export function LoadingGrid() {
  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {Array.from({ length: 8 }).map((_, i) => (
        <Card key={`card-${i}`} className="animate-pulse">
          <CardHeader>
            <div className="mb-2 h-6 rounded bg-gray-200" />
            <div className="h-4 w-3/4 rounded bg-gray-200" />
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {Array.from({ length: 3 }).map((_, j) => (
                <div key={`item-${j}`} className="flex justify-between">
                  <div className="h-3 w-1/3 rounded bg-gray-200" />
                  <div className="h-3 w-1/4 rounded bg-gray-200" />
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

export function ErrorView({
  error,
  onRetry,
  className = '',
}: {
  error: string;
  onRetry: () => void;
  className?: string;
}) {
  return (
    <div className={`py-12 text-center ${className}`}>
      <p className="mb-4 text-red-600">{error}</p>
      <Button onClick={onRetry}>Try Again</Button>
    </div>
  );
}

export function DashboardHeader({
  dataLength,
  total,
  showFilters,
  onToggleFilters,
  onRefresh,
  onExport,
  loading,
}: {
  dataLength: number;
  total: number;
  showFilters: boolean;
  onToggleFilters: () => void;
  onRefresh: () => void;
  onExport: (format: 'json' | 'csv') => void;
  loading: boolean;
}) {
  return (
    <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h2 className="text-2xl font-bold">
          <span className="mr-2">📋</span>
          <span className="bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
            Performance Watch
          </span>
        </h2>
        <p className="text-gray-600 dark:text-gray-300">
          Showing {dataLength} of {total} companies
        </p>
      </div>

      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={onToggleFilters}
          className="flex items-center gap-2"
        >
          <Filter className="h-4 w-4" />
          Filters
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onRefresh}
          className="flex items-center gap-2"
        >
          <RefreshCw className="h-4 w-4" />
          Refresh
        </Button>

        <Select onValueChange={onExport} disabled={loading}>
          <SelectTrigger className="w-24">
            <Download className="h-4 w-4" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="json">Export JSON</SelectItem>
            <SelectItem value="csv">Export CSV</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}

export function FilterPanel({
  filters,
  filterOptions,
  onUpdateFilters,
  onReset,
}: {
  filters: AdvancedFilters;
  filterOptions: FilterOptions;
  onUpdateFilters: (newFilters: Partial<AdvancedFilters>) => void;
  onReset: () => void;
}) {
  return (
    <Card className="p-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
        <div>
          <Label htmlFor="country">Country</Label>
          <Select
            value={filters.country}
            onValueChange={(value) => onUpdateFilters({ country: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="All Countries" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Countries</SelectItem>
              {filterOptions.countries.map((country) => (
                <SelectItem key={country.value} value={country.value}>
                  {country.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <Label htmlFor="sector">Sector</Label>
          <Select
            value={filters.sector}
            onValueChange={(value) => onUpdateFilters({ sector: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="All Sectors" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Sectors</SelectItem>
              {filterOptions.sectors.map((sector) => (
                <SelectItem key={sector} value={sector}>
                  {sector}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <Label htmlFor="min_eps">Min EPS</Label>
          <Input
            type="number"
            value={filters.min_eps ?? ''}
            onChange={(e) =>
              onUpdateFilters({
                min_eps: e.target.value ? parseFloat(e.target.value) : undefined,
              })
            }
            placeholder="0.0"
            step="0.1"
          />
        </div>

        <div>
          <Label htmlFor="min_growth">Min Growth %</Label>
          <Input
            type="number"
            value={filters.min_growth ?? ''}
            onChange={(e) =>
              onUpdateFilters({
                min_growth: e.target.value ? parseFloat(e.target.value) : undefined,
              })
            }
            placeholder="0.0"
            step="0.1"
          />
        </div>
      </div>

      <div className="mt-4 flex items-center gap-2">
        <Button variant="outline" size="sm" onClick={onReset}>
          Reset Filters
        </Button>
      </div>
    </Card>
  );
}

export function StatusLegend() {
  return (
    <div className="mb-6 rounded-lg border border-slate-600 bg-slate-800/90 p-4">
      <div className="mb-4 flex items-center gap-2">
        <span className="text-blue-400">📊</span>
        <h3 className="text-lg font-semibold text-white">Status Legend</h3>
      </div>

      <div className="mb-3">
        <h4 className="mb-3 text-sm font-medium text-slate-300">Status Indicators</h4>
      </div>

      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        <div className="flex items-center gap-3 rounded-lg border border-slate-600/40 bg-slate-700/60 p-3 transition-all duration-200 hover:bg-slate-700">
          <span className="inline-flex items-center rounded bg-green-500 px-2 py-1 text-xs font-medium text-white">
            TRACK
          </span>
          <span className="text-sm text-slate-300">
            Strong performance, actively tracking
          </span>
        </div>

        <div className="flex items-center gap-3 rounded-lg border border-slate-600/40 bg-slate-700/60 p-3 transition-all duration-200 hover:bg-slate-700 md:col-span-1">
          <span className="inline-flex items-center rounded bg-red-500 px-2 py-1 text-xs font-medium text-white">
            STOP
          </span>
          <span className="text-sm text-slate-300">
            Weak performance, avoid investment
          </span>
        </div>
      </div>
    </div>
  );
}

export function EmptyState({ onReset }: { onReset: () => void }) {
  return (
    <div className="py-12 text-center">
      <p className="mb-4 text-gray-600 dark:text-gray-300">No data available</p>
      <Button onClick={onReset}>Reset Filters</Button>
    </div>
  );
}

export function PaginationControls({
  page,
  totalPages,
  hasNext,
  hasPrev,
  onPageChange,
}: {
  page: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
  onPageChange: (page: number) => void;
}) {
  return (
    <div className="mt-8 flex items-center justify-center gap-2">
      <Button
        variant="outline"
        size="sm"
        onClick={() => onPageChange(page - 1)}
        disabled={!hasPrev}
      >
        Previous
      </Button>
      <span className="text-sm text-gray-600 dark:text-gray-300">
        Page {page} of {totalPages}
      </span>
      <Button
        variant="outline"
        size="sm"
        onClick={() => onPageChange(page + 1)}
        disabled={!hasNext}
      >
        Next
      </Button>
    </div>
  );
}
