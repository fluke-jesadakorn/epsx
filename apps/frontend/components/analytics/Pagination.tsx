'use client';

import React from 'react';
import { Button } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@epsx/ui';
import { Input } from '@epsx/ui';
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Crown,
  Settings,
} from 'lucide-react';
import type { CardDashboardPagination } from '@/types/financialChartData';

interface PaginationProps {
  pagination: CardDashboardPagination;
  filters: {
    page: number;
    limit: number;
  };
  loading: boolean;
  userTier?: string;
  maxAllowedLimit?: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  className?: string;
}

export function Pagination({
  pagination,
  filters,
  loading,
  userTier = 'BASIC',
  maxAllowedLimit = 12,
  onPageChange,
  onPageSizeChange,
  className,
}: PaginationProps) {
  const { page: currentPage, totalPages, total, hasNext, hasPrev } = pagination;
  const { page, limit } = filters;

  // Page size options based on user tier
  const getPageSizeOptions = () => {
    const basicOptions = [6, 12, 24];
    const premiumOptions = [6, 12, 24, 48, 96];
    const baseOptions = userTier === 'BASIC' ? basicOptions : premiumOptions;
    return baseOptions.filter(size => size <= maxAllowedLimit);
  };

  const pageSizeOptions = getPageSizeOptions();

  // Calculate page-based pagination info
  const startRecord = (page - 1) * limit + 1;
  const endRecord = Math.min(page * limit, total);
  
  // Generate page numbers for pagination
  const generatePageNumbers = () => {
    const maxVisiblePages = 7;
    const pages: (number | 'ellipsis')[] = [];
    
    if (totalPages <= maxVisiblePages) {
      // Show all pages if total is small
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
    } else {
      // Always show first page
      pages.push(1);
      
      if (currentPage > 4) {
        pages.push('ellipsis');
      }
      
      // Show pages around current page
      const start = Math.max(2, currentPage - 2);
      const end = Math.min(totalPages - 1, currentPage + 2);
      
      for (let i = start; i <= end; i++) {
        pages.push(i);
      }
      
      if (currentPage < totalPages - 3) {
        pages.push('ellipsis');
      }
      
      // Always show last page
      if (totalPages > 1) {
        pages.push(totalPages);
      }
    }
    
    return pages;
  };

  const handlePageInput = (pageValue: string) => {
    const pageNum = parseInt(pageValue);
    if (!isNaN(pageNum) && pageNum >= 1 && pageNum <= totalPages) {
      onPageChange(pageNum);
    }
  };

  const handleJumpToPage = (pageValue: string) => {
    const pageNum = parseInt(pageValue);
    if (!isNaN(pageNum) && pageNum >= 1 && pageNum <= totalPages) {
      onPageChange(pageNum);
    }
  };

  const pageNumbers = generatePageNumbers();

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Main Pagination Controls */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 p-4 bg-card border rounded-lg">
        {/* Records Info & Page Size */}
        <div className="flex items-center gap-4 text-sm">
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground">Show:</span>
            <Select
              value={limit.toString()}
              onValueChange={(value) => onPageSizeChange(parseInt(value))}
              disabled={loading}
            >
              <SelectTrigger className="w-20">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {pageSizeOptions.map((size) => (
                  <SelectItem key={size} value={size.toString()}>
                    {size}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {userTier && (
              <Badge variant="outline" className="text-xs">
                {userTier}
              </Badge>
            )}
          </div>
          
          <div className="text-muted-foreground">
            Showing {startRecord.toLocaleString()} to {endRecord.toLocaleString()} of{' '}
            {total.toLocaleString()} results
          </div>
        </div>

        {/* Navigation Controls */}
        <div className="flex items-center gap-2">
          {/* First/Previous */}
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(1)}
            disabled={!hasPrev || loading}
          >
            <ChevronsLeft className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(currentPage - 1)}
            disabled={!hasPrev || loading}
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>

          {/* Page Numbers */}
          <div className="flex items-center gap-1">
            {pageNumbers.map((pageNum, idx) => (
              pageNum === 'ellipsis' ? (
                <span key={`ellipsis-${idx}`} className="px-2 text-muted-foreground">
                  ...
                </span>
              ) : (
                <Button
                  key={pageNum}
                  variant={pageNum === currentPage ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => onPageChange(pageNum)}
                  disabled={loading}
                  className="w-10"
                >
                  {pageNum}
                </Button>
              )
            ))}
          </div>

          {/* Next/Last */}
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(currentPage + 1)}
            disabled={!hasNext || loading}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(totalPages)}
            disabled={!hasNext || loading}
          >
            <ChevronsRight className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Advanced Skip-based Controls */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 p-3 bg-muted/50 rounded-lg border">
        <div className="flex items-center gap-4 text-sm">
          <div className="flex items-center gap-2">
            <Settings className="h-4 w-4 text-muted-foreground" />
            <span className="text-muted-foreground">Advanced:</span>
          </div>
          
          {/* Skip Input */}
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Page:</span>
            <Input
              type="number"
              min="1"
              max={totalPages}
              value={page}
              onChange={(e) => handlePageInput(e.target.value)}
              className="w-20 h-8 text-xs"
              disabled={loading}
            />
          </div>
          
          {/* Jump to Page */}
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Jump to:</span>
            <Input
              type="number"
              min="1"
              max={totalPages}
              placeholder={currentPage.toString()}
              onChange={(e) => handleJumpToPage(e.target.value)}
              className="w-20 h-8 text-xs"
              disabled={loading}
            />
          </div>
        </div>

        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>Page {currentPage} of {totalPages}</span>
          <Badge variant="outline" className="text-xs">
            Page: {page}
          </Badge>
        </div>
      </div>

      {/* User Tier Upgrade Prompt (if applicable) */}
      {userTier === 'BASIC' && totalPages > 5 && (
        <div className="p-3 bg-gradient-to-r from-orange-50 to-yellow-50 dark:from-orange-950/20 dark:to-yellow-950/20 rounded-lg border border-orange-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Crown className="h-4 w-4 text-orange-500" />
              <span className="text-sm font-medium">
                Unlock unlimited pagination with Premium access
              </span>
            </div>
            <Button size="sm" variant="outline" className="text-xs">
              Upgrade
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}