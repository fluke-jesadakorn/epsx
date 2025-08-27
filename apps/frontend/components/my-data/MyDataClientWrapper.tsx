'use client';
import { useState, useEffect } from "react";
import type { UnifiedRankingItem } from '@/types/analytics';
import { formatPercentage } from '@/types/analytics';

type Asset = {
  symbol: string;
  name: string;
  amount?: number;
};

export default function MyDataClientWrapper({ initialAssets = [] }: { initialAssets?: Asset[] }) {
  const [assets, setAssets] = useState<Asset[]>(initialAssets);
  const [input, setInput] = useState("");
  const [step, setStep] = useState<1 | 2>(1);
  const [analyticsData, setAnalyticsData] = useState<UnifiedRankingItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAddAsset = () => {
    if (!input.trim()) return;
    setAssets([...assets, { symbol: input.trim().toUpperCase(), name: input.trim() }]);
    setInput("");
  };

  const handleRemoveAsset = (symbol: string) => {
    setAssets(assets.filter((a) => a.symbol !== symbol));
  };

  const fetchAnalyticsData = async () => {
    if (assets.length === 0) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // Create a mock API call that simulates fetching real data
      // In a real implementation, this would call an actual API endpoint
      await new Promise(resolve => setTimeout(resolve, 800));
      
      // Mock data based on assets
      const mockData: UnifiedRankingItem[] = assets.map((asset, index) => ({
        symbol: asset.symbol,
        company_name: `${asset.symbol} Inc.`,
        ranking_position: index + 1,
        current_price: 100 + Math.random() * 200,
        current_price_date: new Date().toISOString(),
        quarterly_data: [
          {
            quarter: "Q2 2025",
            date: new Date().toISOString(),
            price: 100 + Math.random() * 200,
            eps: 2 + Math.random() * 5,
            eps_growth: 5 + Math.random() * 20,
            price_growth: -2 + Math.random() * 10,
          },
          {
            quarter: "Q1 2025",
            date: new Date().toISOString(),
            price: 100 + Math.random() * 200,
            eps: 2 + Math.random() * 5,
            eps_growth: 5 + Math.random() * 20,
            price_growth: -2 + Math.random() * 10,
          }
        ],
        market_data: {
          market_cap: 1000000000 + Math.random() * 10000000000,
          volume_24h: 1000000 + Math.random() * 10000000,
          country: "United States",
          sector: "Technology",
          exchange: "NASDAQ"
        },
        analytics: {
          growth_factor: 10 + Math.random() * 30,
          ranking_score: 80 + Math.random() * 20,
          trend: "bullish",
          volatility: 0.5 + Math.random() * 2
        }
      }));
      
      setAnalyticsData(mockData);
    } catch (err) {
      console.error('Error fetching analytics data:', err);
      setError('Failed to load analytics data. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleAnalyzePortfolio = async () => {
    setStep(2);
    await fetchAnalyticsData();
  };

  // Fetch data when assets change
  useEffect(() => {
    if (step === 2 && assets.length > 0) {
      fetchAnalyticsData();
    }
  }, [step, assets]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
      {/* Background decorations */}
      <div className="fixed inset-0 z-0">
        <div className="animate-bounce-slow absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="animate-float absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
        <div className="animate-pulse-gentle absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
      </div>

      <div className="relative z-10 container mx-auto px-4 py-10">
        <div className="max-w-4xl mx-auto">
          <div className="text-center mb-10">
            <h1 className="animate-gradient-x mb-4 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-3xl font-bold text-transparent sm:text-4xl dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
              My Portfolio Analytics
            </h1>
            <p className="mx-auto max-w-2xl text-lg text-gray-600 dark:text-gray-300">
              Track and analyze your assets with professional-grade analytics
            </p>
          </div>

          <div className="rounded-3xl border border-orange-200/50 bg-white/80 p-6 backdrop-blur-xl shadow-2xl dark:border-orange-400/20 dark:bg-slate-800/80">
            {step === 1 && (
              <div>
                <h2 className="text-xl font-bold mb-6 text-gray-900 dark:text-white">
                  Step 1: Add Your Assets
                </h2>
                
                <div className="mb-6">
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Enter asset symbol
                  </label>
                  <div className="flex gap-2">
                    <input
                      className="flex-1 rounded-2xl border border-gray-300 bg-white/80 px-4 py-3 text-gray-900 placeholder-gray-500 backdrop-blur-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 dark:border-gray-600 dark:bg-slate-700/80 dark:text-white dark:placeholder-gray-400"
                      placeholder="Enter stock symbol (e.g. AAPL, GOOGL, TSLA)"
                      value={input}
                      onChange={(e) => setInput(e.target.value.toUpperCase())}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") handleAddAsset();
                      }}
                    />
                    <button
                      className="rounded-2xl bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-3 font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105"
                      onClick={handleAddAsset}
                    >
                      Add
                    </button>
                  </div>
                </div>

                {assets.length > 0 && (
                  <div className="mb-6">
                    <h3 className="text-lg font-semibold mb-3 text-gray-900 dark:text-white">
                      Your Assets ({assets.length})
                    </h3>
                    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
                      {assets.map((asset) => (
                        <div 
                          key={asset.symbol} 
                          className="flex items-center justify-between rounded-xl bg-white/60 p-4 backdrop-blur-sm dark:bg-slate-700/60 border border-gray-200 dark:border-gray-600"
                        >
                          <span className="font-semibold text-gray-900 dark:text-white">
                            {asset.symbol}
                          </span>
                          <button
                            className="rounded-lg bg-red-100 p-2 text-red-600 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50"
                            onClick={() => handleRemoveAsset(asset.symbol)}
                          >
                            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className="flex justify-end">
                  <button
                    className="rounded-2xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
                    disabled={assets.length === 0}
                    onClick={handleAnalyzePortfolio}
                  >
                    Analyze Portfolio
                  </button>
                </div>
              </div>
            )}

            {step === 2 && (
              <MyDataAnalysis 
                assets={assets} 
                analyticsData={analyticsData}
                loading={loading}
                error={error}
                onBack={() => setStep(1)} 
                onRefresh={fetchAnalyticsData}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function MyDataAnalysis({ 
  assets, 
  analyticsData, 
  loading, 
  error, 
  onBack, 
  onRefresh 
}: { 
  assets: Asset[], 
  analyticsData: any[], 
  loading: boolean, 
  error: string | null, 
  onBack: () => void, 
  onRefresh: () => void 
}) {
  if (loading) {
    return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">
            Step 2: Portfolio Analysis
          </h2>
          <button
            className="rounded-xl bg-gray-200 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-300 dark:bg-slate-700 dark:text-gray-300 dark:hover:bg-slate-600"
            onClick={onBack}
          >
            Edit Assets
          </button>
        </div>
        
        <div className="flex justify-center py-10">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-orange-500"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">
            Step 2: Portfolio Analysis
          </h2>
          <button
            className="rounded-xl bg-gray-200 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-300 dark:bg-slate-700 dark:text-gray-300 dark:hover:bg-slate-600"
            onClick={onBack}
          >
            Edit Assets
          </button>
        </div>
        
        <div className="rounded-2xl bg-red-50 p-4 mb-6 border border-red-200 dark:bg-red-900/20 dark:border-red-800">
          <div className="flex items-center">
            <svg className="h-5 w-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span className="ml-2 text-sm font-medium text-red-800 dark:text-red-200">
              {error}
            </span>
          </div>
          <div className="mt-4">
            <button
              className="rounded-xl bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
              onClick={onRefresh}
            >
              Try Again
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white">
          Step 2: Portfolio Analysis
        </h2>
        <div className="flex gap-2">
          <button
            className="rounded-xl bg-gray-200 p-2 text-gray-700 hover:bg-gray-300 dark:bg-slate-700 dark:text-gray-300 dark:hover:bg-slate-600"
            onClick={onRefresh}
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            className="rounded-xl bg-gray-200 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-300 dark:bg-slate-700 dark:text-gray-300 dark:hover:bg-slate-600"
            onClick={onBack}
          >
            Edit Assets
          </button>
        </div>
      </div>

      {analyticsData.length > 0 ? (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {analyticsData.map((assetData) => {
              const latestQuarter = assetData.quarterly_data?.[0];
              const previousQuarter = assetData.quarterly_data?.[1];
              
              return (
                <div 
                  key={assetData.symbol} 
                  className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm transition-all duration-200 hover:shadow-md dark:border-gray-700 dark:bg-slate-700/60"
                >
                  <div className="flex items-start justify-between mb-4">
                    <div>
                      <h3 className="text-xl font-bold text-gray-900 dark:text-white">
                        {assetData.symbol}
                      </h3>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {assetData.company_name}
                      </p>
                    </div>
                    <span className={`inline-flex items-center rounded-full px-3 py-1 text-xs font-medium ${
                      assetData.active_status === 'Active' 
                        ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' 
                        : 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
                    }`}>
                      Active
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-4 mb-4">
                    <div className="rounded-lg bg-blue-50 p-4 dark:bg-blue-900/20">
                      <p className="text-xs font-medium text-blue-600 dark:text-blue-400">
                        Current Price
                      </p>
                      <p className="text-xl font-bold text-blue-800 dark:text-blue-200">
                        ${assetData.current_price?.toFixed(2) || 'N/A'}
                      </p>
                    </div>
                    
                    <div className="rounded-lg bg-green-50 p-4 dark:bg-green-900/20">
                      <p className="text-xs font-medium text-green-600 dark:text-green-400">
                        Growth Factor
                      </p>
                      <p className={`text-xl font-bold ${
                        (assetData.analytics.growth_factor || 0) >= 0 
                          ? 'text-green-800 dark:text-green-200' 
                          : 'text-red-800 dark:text-red-200'
                      }`}>
                        {formatPercentage(assetData.analytics.growth_factor)}
                      </p>
                    </div>
                  </div>

                  {latestQuarter && previousQuarter && (
                    <div className="border-t border-gray-200 pt-4 dark:border-gray-700">
                      <h4 className="text-sm font-semibold text-gray-700 mb-3 dark:text-gray-300">
                        Quarterly Performance
                      </h4>
                      <div className="grid grid-cols-2 gap-3">
                        <div className="text-center">
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            {previousQuarter.quarter}
                          </p>
                          <p className={`text-lg font-bold ${
                            previousQuarter.eps_growth >= 0 
                              ? 'text-green-600' 
                              : 'text-red-600'
                          }`}>
                            {formatPercentage(previousQuarter.eps_growth)}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            EPS: {previousQuarter.eps.toFixed(2)}
                          </p>
                        </div>
                        <div className="text-center">
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            {latestQuarter.quarter}
                          </p>
                          <p className={`text-lg font-bold ${
                            latestQuarter.eps_growth >= 0 
                              ? 'text-green-600' 
                              : 'text-red-600'
                          }`}>
                            {formatPercentage(latestQuarter.eps_growth)}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            EPS: {latestQuarter.eps.toFixed(2)}
                          </p>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-slate-700">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-300">
                    Symbol
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-300">
                    Price
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-300">
                    Growth Factor
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-300">
                    Market Cap
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-300">
                    Status
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200 dark:bg-slate-800 dark:divide-gray-700">
                {analyticsData.map((assetData) => (
                  <tr key={assetData.symbol} className="hover:bg-gray-50 dark:hover:bg-slate-700/50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                      {assetData.symbol}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-300">
                      ${assetData.current_price?.toFixed(2) || 'N/A'}
                    </td>
                    <td className={`px-6 py-4 whitespace-nowrap text-sm font-medium ${
                      (assetData.analytics.growth_factor || 0) >= 0 
                        ? 'text-green-600 dark:text-green-400' 
                        : 'text-red-600 dark:text-red-400'
                    }`}>
                      {formatPercentage(assetData.analytics.growth_factor)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-300">
                      {assetData.market_data.market_cap 
                        ? `${(assetData.market_data.market_cap / 1e9).toFixed(1)}B` 
                        : 'N/A'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400`}>
                        Active
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <div className="text-center py-10">
          <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
            No data found
          </h3>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            We couldn't find analytics data for your assets. Try adding different symbols.
          </p>
          <div className="mt-6">
            <button
              className="rounded-2xl bg-gradient-to-r from-orange-500 to-yellow-500 px-4 py-2 font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105"
              onClick={onBack}
            >
              Add More Assets
            </button>
          </div>
        </div>
      )}

      <div className="mt-8 flex justify-center">
        <button
          className="rounded-2xl bg-gray-200 px-6 py-3 font-medium text-gray-700 hover:bg-gray-300 dark:bg-slate-700 dark:text-gray-300 dark:hover:bg-slate-600"
          onClick={onBack}
        >
          Back to Asset Selection
        </button>
      </div>
    </div>
  );
}