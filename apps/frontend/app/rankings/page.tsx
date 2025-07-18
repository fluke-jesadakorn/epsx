import * as React from 'react';
import type { Metadata } from 'next';
import { PaginatedStockGrid } from '@/components/shared/PaginatedStockGrid';
import { fetchPaginatedStockData } from '@/app/actions/stockRankingPaginated';

export const metadata: Metadata = {
  title: 'Stock Rankings with Pagination - EPSX',
  description: 'Browse stock rankings with advanced pagination controls',
};

export default async function StockRankingsPage() {
  // Fetch initial data for the first page
  const initialData = await fetchPaginatedStockData(1, 10);

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold tracking-tight">Stock Rankings</h1>
          <p className="text-muted-foreground mt-2">
            Browse through our comprehensive stock rankings with advanced pagination controls
          </p>
        </div>
        
        <PaginatedStockGrid initialData={initialData} />
      </div>
    </div>
  );
}

// Revalidate page every 5 minutes
export const revalidate = 300;
