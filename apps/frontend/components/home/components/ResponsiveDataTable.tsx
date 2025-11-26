'use client';

import React, { useState } from 'react';
import { FinancialCard } from './FinancialCard';
import { formatPrice, formatDate } from '@/utils/fmt';
import type { StockFinancialData } from '@/types/financialChartData';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from '@/components/ui/table';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { 
  Grid3X3, 
  TableProperties, 
  TrendingUp, 
  TrendingDown,
  ArrowUpDown,
  Search,
  Filter
} from 'lucide-react';

interface ResponsiveDataTableProps {
  data: StockFinancialData[];
  className?: string;
}

type ViewMode = 'cards' | 'table';
type SortField = 'symbol' | 'price' | 'eps' | 'growth';
type SortDirection = 'asc' | 'desc';

export function ResponsiveDataTable({ data, className = '' }: ResponsiveDataTableProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('cards');
  const [sortField, setSortField] = useState<SortField>('symbol');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [searchTerm, setSearchTerm] = useState('');
  const [showFilters, setShowFilters] = useState(false);

  // Filter and sort data
  const filteredData = data.filter(item =>
    item.symbol.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const sortedData = [...filteredData].sort((a, b) => {
    let aValue: any, bValue: any;
    
    switch (sortField) {
      case 'symbol':
        aValue = a.symbol;
        bValue = b.symbol;
        break;
      case 'price':
        aValue = a.currentPrice || 0;
        bValue = b.currentPrice || 0;
        break;
      case 'eps':
        aValue = a.quarters?.[0]?.eps || 0;
        bValue = b.quarters?.[0]?.eps || 0;
        break;
      case 'growth':
        aValue = a.quarters?.[0]?.eps_growth || 0;
        bValue = b.quarters?.[0]?.eps_growth || 0;
        break;
      default:
        aValue = a.symbol;
        bValue = b.symbol;
    }

    if (sortDirection === 'asc') {
      return aValue > bValue ? 1 : -1;
    } else {
      return aValue < bValue ? 1 : -1;
    }
  });

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  };

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) return <ArrowUpDown className="h-3 w-3" />;
    return sortDirection === 'asc' 
      ? <TrendingUp className="h-3 w-3" />
      : <TrendingDown className="h-3 w-3" />;
  };

  return (
    <div className={`w-full ${className}`}>
      {/* Controls - Mobile Optimized */}
      <div className="mb-6 space-y-4">
        {/* Search and View Toggle */}
        <div className="flex flex-col sm:flex-row gap-4 sm:items-center sm:justify-between">
          {/* Search */}
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              placeholder="Search stocks..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border rounded-lg bg-background focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          {/* View Mode Toggle */}
          <div className="flex items-center gap-2">
            <Button
              variant={viewMode === 'cards' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setViewMode('cards')}
              className="flex items-center gap-2"
            >
              <Grid3X3 className="h-4 w-4" />
              <span className="hidden sm:inline">Cards</span>
            </Button>
            <Button
              variant={viewMode === 'table' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setViewMode('table')}
              className="flex items-center gap-2"
            >
              <TableProperties className="h-4 w-4" />
              <span className="hidden sm:inline">Table</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowFilters(!showFilters)}
              className="flex items-center gap-2 md:hidden"
            >
              <Filter className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {/* Sort Controls - Mobile */}
        {(showFilters || viewMode === 'table') && (
          <Card className="md:hidden">
            <CardContent className="p-4">
              <div className="flex flex-wrap gap-2">
                <span className="text-sm font-medium text-muted-foreground mb-2 w-full">
                  Sort by:
                </span>
                {[
                  { field: 'symbol' as SortField, label: 'Symbol' },
                  { field: 'price' as SortField, label: 'Price' },
                  { field: 'eps' as SortField, label: 'EPS' },
                  { field: 'growth' as SortField, label: 'Growth' },
                ].map(({ field, label }) => (
                  <Button
                    key={field}
                    variant={sortField === field ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => handleSort(field)}
                    className="flex items-center gap-1"
                  >
                    {label}
                    {getSortIcon(field)}
                  </Button>
                ))}
              </div>
            </CardContent>
          </Card>
        )}
      </div>

      {/* Results Count */}
      <div className="mb-4 flex items-center justify-between">
        <Badge variant="secondary" className="text-xs">
          {sortedData.length} stocks found
        </Badge>
        {searchTerm && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setSearchTerm('')}
            className="text-xs"
          >
            Clear search
          </Button>
        )}
      </div>

      {/* Card View */}
      {viewMode === 'cards' && (
        <>
          {/* Mobile: Horizontal Scroll */}
          <div className="block md:hidden">
            <div className="overflow-x-auto pb-4">
              <div className="flex gap-4 w-max">
                {sortedData.map((item, index) => (
                  <div
                    key={`mobile-${item.symbol}-${index}`}
                    className="w-72 flex-shrink-0"
                  >
                    <FinancialCard data={item} index={index} />
                  </div>
                ))}
              </div>
              {/* Scroll indicator */}
              <div className="flex justify-center mt-4">
                <div className="flex gap-1">
                  {Array.from({ length: Math.min(5, sortedData.length) }).map((_, i) => (
                    <div key={i} className="w-2 h-2 bg-primary/30 rounded-full" />
                  ))}
                </div>
                <p className="text-xs text-muted-foreground ml-3 self-center">
                  Swipe to see more →
                </p>
              </div>
            </div>
          </div>

          {/* Desktop: Grid */}
          <div className="hidden md:block">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4 lg:gap-6">
              {sortedData.map((item, index) => (
                <div key={`desktop-${item.symbol}-${index}`}>
                  <FinancialCard data={item} index={index} />
                </div>
              ))}
            </div>
          </div>
        </>
      )}

      {/* Table View */}
      {viewMode === 'table' && (
        <div className="w-full">
          {/* Mobile: Simplified Cards */}
          <div className="block md:hidden space-y-3">
            {sortedData.map((item, index) => (
              <Card key={`table-mobile-${item.symbol}`} className="p-4">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className="text-xs">
                      #{index + 1}
                    </Badge>
                    <span className="font-semibold">{item.symbol}</span>
                  </div>
                  <span className="font-bold text-primary">
                    {item.currentPrice ? formatPrice(item.currentPrice) : 'N/A'}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-muted-foreground">EPS:</span>
                    <span className="ml-2 font-medium">
                      {item.quarters?.[0]?.eps?.toFixed(4) || 'N/A'}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Growth:</span>
                    <span className={`ml-2 font-medium ${
                      (item.quarters?.[0]?.eps_growth || 0) > 0 
                        ? 'text-green-600' 
                        : 'text-red-600'
                    }`}>
                      {item.quarters?.[0]?.eps_growth?.toFixed(1) || 'N/A'}%
                    </span>
                  </div>
                </div>
              </Card>
            ))}
          </div>

          {/* Desktop: Full Table */}
          <div className="hidden md:block rounded-lg border overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow className="bg-muted/50">
                  <TableHead className="w-12">#</TableHead>
                  <TableHead 
                    className="cursor-pointer hover:bg-muted/70 transition-colors"
                    onClick={() => handleSort('symbol')}
                  >
                    <div className="flex items-center gap-2">
                      Symbol
                      {getSortIcon('symbol')}
                    </div>
                  </TableHead>
                  <TableHead 
                    className="cursor-pointer hover:bg-muted/70 transition-colors text-right"
                    onClick={() => handleSort('price')}
                  >
                    <div className="flex items-center justify-end gap-2">
                      Price
                      {getSortIcon('price')}
                    </div>
                  </TableHead>
                  <TableHead 
                    className="cursor-pointer hover:bg-muted/70 transition-colors text-right"
                    onClick={() => handleSort('eps')}
                  >
                    <div className="flex items-center justify-end gap-2">
                      EPS
                      {getSortIcon('eps')}
                    </div>
                  </TableHead>
                  <TableHead 
                    className="cursor-pointer hover:bg-muted/70 transition-colors text-right"
                    onClick={() => handleSort('growth')}
                  >
                    <div className="flex items-center justify-end gap-2">
                      EPS Growth
                      {getSortIcon('growth')}
                    </div>
                  </TableHead>
                  <TableHead className="text-right">Latest Date</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedData.map((item, index) => (
                  <TableRow 
                    key={item.symbol}
                    className="hover:bg-muted/50 transition-colors"
                  >
                    <TableCell>
                      <Badge variant="outline" className="text-xs">
                        #{index + 1}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-medium">
                      {item.symbol}
                    </TableCell>
                    <TableCell className="text-right font-semibold">
                      {item.currentPrice ? formatPrice(item.currentPrice) : 'N/A'}
                    </TableCell>
                    <TableCell className="text-right">
                      {item.quarters?.[0]?.eps?.toFixed(4) || 'N/A'}
                    </TableCell>
                    <TableCell className="text-right">
                      <span className={
                        (item.quarters?.[0]?.eps_growth || 0) > 0 
                          ? 'text-green-600 font-semibold' 
                          : 'text-red-600 font-semibold'
                      }>
                        {item.quarters?.[0]?.eps_growth?.toFixed(1) || 'N/A'}%
                      </span>
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {item.quarters?.[0]?.date ? formatDate(item.quarters[0].date) : 'N/A'}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </div>
      )}

      {/* Empty State */}
      {sortedData.length === 0 && (
        <Card className="p-8 text-center">
          <div className="text-4xl mb-4">📊</div>
          <h3 className="text-lg font-semibold mb-2">No stocks found</h3>
          <p className="text-muted-foreground mb-4">
            Try adjusting your search terms or clearing the filter.
          </p>
          <Button onClick={() => setSearchTerm('')} variant="outline">
            Clear Search
          </Button>
        </Card>
      )}

      {/* Load More - Mobile Optimized */}
      {sortedData.length > 0 && (
        <div className="mt-8 text-center">
          <Button className="w-full sm:w-auto px-6 py-3 bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white font-semibold shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
            <span className="flex items-center justify-center gap-2">
              <span>📈 Load More Data</span>
              <span className="text-sm opacity-80">(+20 more)</span>
            </span>
          </Button>
          <p className="text-xs text-muted-foreground mt-3 block sm:hidden">
            Tap to load more financial data
          </p>
        </div>
      )}
    </div>
  );
}

export default ResponsiveDataTable;