import * as React from 'react';
import type { Metadata } from 'next';

import { AnalyticsRankingDashboard } from '@/components/analytics/AnalyticsRankingDashboard';
import { PaginatedStockGrid } from '@/components/shared/PaginatedStockGrid';
import { fetchPaginatedStockData } from '@/app/actions/stockRankingPaginated';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

export const metadata: Metadata = {
  title: 'Analytics Dashboard - EPSX',
  description: 'Comprehensive stock ranking analytics and insights based on your subscription level',
};

export default async function AnalyticsPage() {
  // Fetch initial data for the paginated grid
  const initialData = await fetchPaginatedStockData(1, 10);

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold tracking-tight">Analytics Dashboard</h1>
          <p className="text-muted-foreground mt-2">
            Comprehensive stock ranking analytics and insights based on your subscription level
          </p>
        </div>
        
        <Tabs defaultValue="grid" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="grid">Grid View</TabsTrigger>
            <TabsTrigger value="table">Table View</TabsTrigger>
          </TabsList>
          
          <TabsContent value="grid" className="space-y-6">
            <PaginatedStockGrid initialData={initialData} />
          </TabsContent>
          
          <TabsContent value="table" className="space-y-6">
            <AnalyticsRankingDashboard />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

// Revalidate page every 5 minutes
export const revalidate = 300;
