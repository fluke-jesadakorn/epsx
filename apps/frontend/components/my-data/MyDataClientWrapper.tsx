'use client';
import { useState } from "react";
import { PositionAction, type PortfolioPosition } from '@/types/analytics';
import { AnalyticsNavigation } from '@/components/shared/AnalyticsNavigation';
import PositionCard from './PositionCard';
import PortfolioHeader from './PortfolioHeader';
import { mockPortfolioData } from './mockPortfolioData';

export default function MyDataClientWrapper() {
  const [portfolioData, setPortfolioData] = useState(mockPortfolioData);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleActionChange = (symbol: string, action: PositionAction) => {
    setPortfolioData(prev => ({
      ...prev,
      positions: prev.positions.map(position => 
        position.symbol === symbol 
          ? { ...position, actionStatus: action }
          : position
      )
    }));
  };

  const handleRefresh = async () => {
    setLoading(true);
    setError(null);
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1200));
      
      // Update processing time
      setPortfolioData(prev => ({
        ...prev,
        processingTime: Math.floor(Math.random() * 50000) + 30000,
        lastUpdated: new Date().toISOString()
      }));
    } catch (err) {
      console.error('Error refreshing portfolio data:', err);
      setError('Failed to refresh portfolio data. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="animate-bounce-slow absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="animate-float absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
        <div className="animate-pulse-gentle absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
        <div className="animate-float-reverse absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 blur-3xl" />

        {/* Mesh gradient overlays for depth */}
        <div className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)]" />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)]"
          style={{ animationDelay: '1s' }}
        />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)]"
          style={{ animationDelay: '2s' }}
        />

        {/* Decorative geometric shapes */}
        <div className="animate-spin-slow absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
        <div className="animate-bounce-gentle absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />
      </div>

      <div className="relative z-10">
        <div className="container mx-auto px-4 py-8">
          {/* Header with enhanced styling */}
          <div className="mb-8 text-center">
            <h1 className="animate-gradient-x mb-4 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-4xl font-bold text-transparent sm:text-5xl dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
              My Portfolio Analytics
            </h1>
            <p className="mx-auto max-w-2xl text-lg text-gray-600 dark:text-gray-300">
              Track and analyze your assets with professional-grade analytics
            </p>
            <AnalyticsNavigation currentPage="my-data" />
            {/* Decorative elements */}
            <div className="mt-6 flex items-center justify-center gap-4">
              <div className="h-2 w-2 animate-pulse rounded-full bg-orange-400" />
              <div
                className="h-3 w-3 animate-pulse rounded-full bg-yellow-400"
                style={{ animationDelay: '0.5s' }}
              />
              <div
                className="h-2 w-2 animate-pulse rounded-full bg-orange-400"
                style={{ animationDelay: '1s' }}
              />
            </div>
          </div>

          {/* Portfolio content with floating decorations */}
          <div className="relative">
            <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
            <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl" />
            
            <div className="rounded-3xl border border-orange-200/50 bg-white/80 p-6 backdrop-blur-xl shadow-2xl dark:border-orange-400/20 dark:bg-slate-800/80">
            {/* Portfolio Header */}
            <PortfolioHeader 
              processingTime={portfolioData.processingTime}
              onRefresh={handleRefresh}
              isLoading={loading}
            />

            {/* Error Message */}
            {error && (
              <div className="rounded-2xl bg-red-50 p-4 mb-6 border border-red-200 dark:bg-red-900/20 dark:border-red-800">
                <div className="flex items-center">
                  <svg className="h-5 w-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <span className="ml-2 text-sm font-medium text-red-800 dark:text-red-200">
                    {error}
                  </span>
                </div>
              </div>
            )}

            {/* Portfolio Positions Grid */}
            <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-4 gap-6">
              {portfolioData.positions.map((position) => (
                <PositionCard 
                  key={position.symbol}
                  position={position}
                  onActionChange={handleActionChange}
                />
              ))}
            </div>

            {/* Empty State */}
            {portfolioData.positions.length === 0 && (
              <div className="text-center py-12">
                <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-blue-100 dark:bg-blue-900/30">
                  <svg className="h-8 w-8 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                  </svg>
                </div>
                <h3 className="mt-4 text-lg font-medium text-gray-900 dark:text-white">
                  No portfolio positions
                </h3>
                <p className="mt-2 text-gray-500 dark:text-gray-400">
                  Add assets to your portfolio to start tracking their performance.
                </p>
                <div className="mt-6">
                  <button
                    className="rounded-2xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105"
                    onClick={() => {}}
                  >
                    Add Assets
                  </button>
                </div>
              </div>
            )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}