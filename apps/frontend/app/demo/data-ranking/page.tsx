import * as React from 'react';
import DataRankTable from '@/components/home/DataRankTable';
import { fetchStockRankingData } from '@/app/actions/stockRanking';
import { SkeletonLoader } from '@/components/common/Skeleton';
import type { TableDataMetrics } from '@/types/stockFetchData';

/**
 * Demo page showcasing DataRankTable with role-based access control
 * This shows how the DataRankTable component works with user subscriptions
 */
async function DataRankingDemoPage() {
  // Fetch stock data and convert to TableDataMetrics format
  const stockData = await fetchStockRankingData(0, 100);
  
  // Convert StockFinancialData to TableDataMetrics format for DataRankTable
  const tableData: TableDataMetrics[] = stockData.map((stock, index) => {
    const latestQuarter = stock.quarters[stock.quarters.length - 1];
    const previousQuarter = stock.quarters[stock.quarters.length - 2];
    const growthRate = previousQuarter 
      ? ((latestQuarter?.eps - previousQuarter.eps) / previousQuarter.eps * 100)
      : 0;
    
    return {
      symbol: stock.symbol || `STOCK${index + 1}`,
      name: `${stock.symbol} Corp` || `Company ${index + 1}`,
      valueIndex: (stock.currentPrice || latestQuarter?.price || Math.random() * 100 + 50).toFixed(2),
      growthRate: (growthRate || (Math.random() - 0.5) * 20).toFixed(2),
      activityScore: (Math.random() * 1000000).toFixed(0),
      marketSize: (Math.random() * 10000000000).toFixed(0),
      growthFactor: (Math.random() * 50 + 5).toFixed(2),
      sector: ['Technology', 'Healthcare', 'Finance', 'Energy', 'Consumer'][index % 5],
      country: ['US', 'CA', 'GB', 'DE', 'JP'][index % 5],
      exchange: ['NASDAQ', 'NYSE', 'LSE', 'TSX'][index % 4],
      currency: 'USD',
      entryPhase: {
        date: new Date().toISOString().split('T')[0],
        active: Math.random() > 0.5,
      },
      phaseStatus: {
        date: new Date().toISOString().split('T')[0],
        type: Math.random() > 0.5 ? 'monitor' : 'exit',
        active: Math.random() > 0.5,
      },
      metricScore: (latestQuarter?.eps || Math.random() * 10).toFixed(2),
      growthIndicator: growthRate > 0 ? 'Positive' : 'Negative',
      currentMetric: (stock.currentPrice || latestQuarter?.price || Math.random() * 100).toFixed(2),
      predictedMetric: ((stock.currentPrice || latestQuarter?.price || 100) * (1 + Math.random() * 0.2)).toFixed(2),
      lastAnalysisDate: new Date().toISOString().split('T')[0],
      nextAnalysisDate: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
    };
  });

  return (
    <React.Suspense fallback={<SkeletonLoader />}>
      <div className="container mx-auto px-4 py-8">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold mb-4">
            <span className="bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent">
              🔐 Role-Based Data Rankings Demo
            </span>
          </h1>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            Experience how DataRankTable adapts to different subscription levels. 
            Your view is personalized based on your current plan.
          </p>
          <div className="mt-4 p-4 bg-secondary/50 rounded-lg max-w-md mx-auto">
            <h3 className="font-semibold mb-2">Subscription Benefits:</h3>
            <ul className="text-sm space-y-1">
              <li><strong>BASIC:</strong> 5 rankings</li>
              <li><strong>SILVER:</strong> 25 rankings</li>
              <li><strong>GOLD:</strong> 50 rankings</li>
              <li><strong>PLATINUM:</strong> 100 rankings</li>
            </ul>
          </div>
        </div>
        
        <DataRankTable 
          data={tableData}
          defaultView="card"
          className="max-w-7xl mx-auto"
        />
      </div>
    </React.Suspense>
  );
}

export default DataRankingDemoPage;

// Cache for 5 minutes
export const revalidate = 300;
