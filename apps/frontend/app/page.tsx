import { StreamingWrapper } from '@/components/common/StreamingWrapper';
import ChatSection from '@/components/home/ChatSection';
import ClientEpsCardSection from '@/components/home/ClientEpsCardSection';
import HeroSection from '@/components/home/HeroSection';
import { PublicRankingPreview } from '@/components/home/PublicRankingPreview';
import DynamicPricingSection from '@/components/home/DynamicPricingSection';
import { getAnalyticsData } from '@/lib/server-data';

// DISABLE ISR caching to show real TradingView data immediately
export const revalidate = 0;

// Use proper types for financial data

import type { TableDataMetrics } from '@/types/stockFetchData';
import type { StockFinancialData } from '@/types/financialChartData';

export default async function HomePage() {
  // Fetch real data from analytics API
  let initialData: StockFinancialData[] = [];
  let epsCardData: TableDataMetrics[] = [];
  
  try {
    const analyticsResponse = await getAnalyticsData({ page: 1, limit: 10, sort_by: 'growth_factor' });
    
    // Check if we have successful data or if authentication is required
    if (analyticsResponse && analyticsResponse.success && analyticsResponse.rankings && analyticsResponse.rankings.length > 0) {
      // Convert analytics data to format expected by components
      initialData = analyticsResponse.rankings.slice(0, 6).map((item, index) => ({
        symbol: item.symbol,
        rank: item.rank,
        quarters: item.quarterly_performance || [],
        currentPrice: item.quarterly_performance?.[0]?.price || 0
      }));
      
      // Use top 3 for EPS card data - convert to TableDataMetrics format
      epsCardData = analyticsResponse.rankings.slice(0, 3).map((item, index) => ({
        symbol: item.symbol,
        name: item.symbol, // Use symbol as name for now
        valueIndex: `${index + 1}`,
        growthRate: `${(item.quarterly_performance?.[0]?.eps_growth || 0).toFixed(2)}%`,
        activityScore: item.active_status === 'TRACK' ? '95' : '75',
        marketSize: 'Large',
        growthFactor: `${(item.quarterly_performance?.[0]?.eps_growth || 0).toFixed(1)}x`,
        sector: 'Technology', // Default sector
        country: 'US', // Default country
        exchange: 'NYSE', // Default exchange
        currency: 'USD',
        entryPhase: {
          date: item.quarterly_performance?.[0]?.date || new Date().toISOString(),
          active: item.active_status === 'TRACK'
        },
        phaseStatus: {
          date: item.quarterly_performance?.[0]?.date || new Date().toISOString(),
          type: item.active_status === 'TRACK' ? 'monitor' : 'exit',
          active: true
        },
        metricScore: `${Math.round((item.quarterly_performance?.[0]?.eps_growth || 0) * 10)}`,
        growthIndicator: item.active_status === 'TRACK' ? 'Strong' : 'Weak',
        currentMetric: `${(item.quarterly_performance?.[0]?.eps || 0).toFixed(2)}`,
        predictedMetric: `${((item.quarterly_performance?.[0]?.eps || 0) * 1.1).toFixed(2)}`,
        lastAnalysisDate: item.quarterly_performance?.[0]?.date || new Date().toISOString(),
        nextAnalysisDate: new Date(Date.now() + 90 * 24 * 60 * 60 * 1000).toISOString(),
        epsGrowth: `${(item.quarterly_performance?.[0]?.eps_growth || 0).toFixed(2)}%`,
        startBuy: { active: item.active_status === 'TRACK' },
        startAction: { type: item.active_status === 'TRACK' ? 'hold' : 'hold', active: true },
        lastEarningsDate: item.quarterly_performance?.[0]?.date || 'N/A'
      }));
    } else {
      // Log the case where authentication is required or data is unavailable
      console.log('Analytics data not available - user may need to authenticate for premium data');
    }
  } catch (error) {
    console.log('Home page: Analytics fetch failed, showing without data:', error);
    // Keep empty arrays as fallback for graceful degradation
  }
  return (
    <div>
      {/* Promotional Banner */}
      <div className="bg-gradient-to-r from-orange-500 via-red-500 to-pink-500 text-white">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-center">
            <div className="flex items-center gap-2">
              <span className="font-bold text-lg">25% OFF</span>
              <span className="text-sm opacity-90">Limited time offer on all plans!</span>
            </div>
          </div>
        </div>
      </div>
      
      <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
        <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
        <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 blur-3xl" />

        {/* Mesh gradient overlays for depth */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)]" />

        {/* Decorative geometric shapes */}
        <div className="absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
        <div className="absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />
      </div>

      {/* Main content with PancakeSwap styling */}
      <div className="relative z-10">
        {/* Chat Section with enhanced PancakeSwap styling */}
        <StreamingWrapper priority="high" identifier="chat">
          <ChatSection />
        </StreamingWrapper>

        {/* Hero Section with vibrant PancakeSwap animations */}
        <StreamingWrapper priority="high" identifier="hero">
          <HeroSection className="relative z-10" />
        </StreamingWrapper>


        {/* EPS Cards Section - Top Performing Companies */}
        <StreamingWrapper priority="medium" identifier="eps-cards">
          <div className="container mx-auto px-4 py-12">
            <div className="relative">
              <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
              <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl" />
              <ClientEpsCardSection initialData={epsCardData} />
            </div>
          </div>
        </StreamingWrapper>

        {/* Dynamic Pricing Section with affiliate tracking */}
        <StreamingWrapper priority="medium" identifier="pricing">
          <DynamicPricingSection />
        </StreamingWrapper>

        {/* Data Rank Table with vibrant PancakeSwap-style card */}
        <div className="container mx-auto px-4 py-16">
          <div className="relative">
            {/* Enhanced background decorations */}
            <div className="absolute -top-12 left-1/4 h-24 w-24 rotate-12 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
            <div className="absolute right-1/4 -bottom-12 h-20 w-20 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />

            {/* Main card with PancakeSwap styling */}
            <div className="relative overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 p-8 shadow-2xl backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80">
              {/* Card background pattern */}
              <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-blue-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-blue-900/10" />
              <div className="absolute top-0 right-0 h-32 w-32 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10 blur-2xl" />
              <div className="absolute bottom-0 left-0 h-40 w-40 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10 blur-2xl" />

              <div className="relative z-10">
                <div className="mb-10 text-center">
                  {/* Updated title for public rankings */}
                  <h2 className="mb-6 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-4xl font-bold text-transparent sm:text-5xl dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
                    Track{' '}
                    <span className="bg-gradient-to-r from-purple-500 to-purple-600 bg-clip-text text-transparent">
                      Performance
                    </span>{' '}
                    <span className="bg-gradient-to-r from-pink-500 to-pink-600 bg-clip-text text-transparent">
                      Growth
                    </span>{' '}
                    Rankings
                  </h2>
                  <p className="mx-auto max-w-3xl text-lg leading-relaxed text-gray-600 dark:text-gray-300">
                    Unlock deeper insights and optimize data center performance
                    with real-time analytics and advanced data tracking systems
                    for smarter operational decisions
                  </p>
                  <p className="mt-2 text-sm text-blue-600 dark:text-blue-400">
                    🔒 Upgrade to access Top 100 rankings with advanced insights
                  </p>
                  {/* Decorative elements */}
                  <div className="mt-6 flex items-center justify-center gap-4">
                    <div className="h-2 w-2 rounded-full bg-blue-400" />
                    <div className="h-3 w-3 rounded-full bg-purple-400" />
                    <div className="h-2 w-2 rounded-full bg-blue-400" />
                  </div>
                </div>

                <StreamingWrapper priority="low" identifier="rankings">
                  <PublicRankingPreview initialData={initialData} />
                </StreamingWrapper>
              </div>
            </div>
          </div>
        </div>
      </div>
      </div>
    </div>
  );
}
