import ServerCardDashboard from '@/components/analytics/ServerCardDashboard';

interface AnalyticsPageProps {
  searchParams: {
    page?: string;
    limit?: string;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: string;
    min_growth?: string;
    showFilters?: string;
  };
}

export default function AnalyticsPage({ searchParams }: AnalyticsPageProps) {
  return (
    <div className="relative min-h-screen overflow-hidden bg-gradient-to-br from-pink-50 via-yellow-50 to-orange-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-orange-900/20">
      {/* PancakeSwap-inspired vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Primary gradient background with PancakeSwap colors */}
        <div className="absolute inset-0 bg-gradient-to-br from-pink-100 via-yellow-100 to-orange-100 dark:from-gray-900 dark:via-purple-900/30 dark:to-orange-900/30" />

        {/* Multiple animated gradient orbs - PancakeSwap style */}
        <div className="animate-bounce-slow absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-pink-400/40 to-orange-400/40 blur-3xl" />
        <div className="animate-float absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-yellow-400/35 to-pink-400/35 blur-3xl" />
        <div className="animate-pulse-gentle absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="animate-float-reverse absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-purple-400/25 to-pink-400/25 blur-3xl" />
        <div className="animate-spin-very-slow absolute bottom-40 right-20 h-60 w-60 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-3xl" />

        {/* Pancake-inspired floating shapes */}
        <div className="animate-float absolute top-1/4 left-1/6 h-20 w-20 rounded-full bg-gradient-to-br from-yellow-300/30 to-orange-300/30" />
        <div className="animate-bounce-gentle absolute top-3/4 right-1/6 h-16 w-16 rotate-45 rounded-lg bg-gradient-to-br from-pink-300/25 to-purple-300/25" />
        <div className="animate-spin-slow absolute bottom-1/4 left-1/3 h-12 w-12 rounded-full bg-gradient-to-br from-orange-300/20 to-yellow-300/20" />

        {/* Enhanced mesh gradient overlays */}
        <div className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_20%_30%,_rgba(236,72,153,0.15)_0%,_transparent_50%)]" />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_80%_20%,_rgba(251,191,36,0.12)_0%,_transparent_50%)]"
          style={{ animationDelay: '1s' }}
        />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_60%_80%,_rgba(249,115,22,0.1)_0%,_transparent_60%)]"
          style={{ animationDelay: '2s' }}
        />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_40%_60%,_rgba(168,85,247,0.08)_0%,_transparent_50%)]"
          style={{ animationDelay: '3s' }}
        />

        {/* Floating pancake-inspired decorative elements */}
        <div className="animate-spin-slow absolute top-1/3 left-1/5 h-8 w-8 rounded-full bg-gradient-to-br from-yellow-400/40 to-orange-400/40 blur-sm" />
        <div className="animate-bounce-gentle absolute top-1/2 right-1/5 h-6 w-6 rounded-full bg-gradient-to-br from-pink-400/35 to-purple-400/35 blur-sm" />
        <div className="animate-float absolute bottom-1/3 left-2/3 h-10 w-10 rotate-45 rounded-lg bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-sm" />
      </div>

      {/* Main content with enhanced PancakeSwap styling */}
      <div className="relative z-10">
        <div className="container mx-auto px-4 py-8">
          {/* Enhanced header with PancakeSwap-inspired styling */}
          <div className="mb-12 text-center">
            {/* Floating decorative elements around header */}
            <div className="relative">
              <div className="animate-float absolute -top-4 -left-8 h-8 w-8 rounded-full bg-gradient-to-br from-pink-400/50 to-orange-400/50 blur-sm" />
              <div className="animate-bounce-gentle absolute -top-6 -right-8 h-6 w-6 rotate-45 rounded-lg bg-gradient-to-br from-yellow-400/60 to-pink-400/60 blur-sm" />
              
              <h1 className="animate-gradient-x mb-4 bg-gradient-to-r from-pink-500 via-orange-500 to-yellow-500 bg-clip-text text-5xl font-bold text-transparent sm:text-6xl dark:from-pink-400 dark:via-orange-400 dark:to-yellow-400">
                🥞 Analytics Hub
              </h1>
            </div>
            
            <p className="mx-auto max-w-2xl text-xl text-gray-700 dark:text-gray-200 font-medium">
              Sweet analytics platform with delicious insights and 
              <span className="bg-gradient-to-r from-pink-500 to-orange-500 bg-clip-text text-transparent font-bold"> real-time data</span>
            </p>
            
            {/* Enhanced decorative elements */}
            <div className="mt-8 flex items-center justify-center gap-6">
              <div className="h-3 w-3 animate-pulse rounded-full bg-gradient-to-r from-pink-400 to-orange-400" />
              <div className="h-2 w-12 animate-pulse rounded-full bg-gradient-to-r from-yellow-400 to-pink-400" style={{ animationDelay: '0.3s' }} />
              <div className="h-4 w-4 animate-pulse rounded-full bg-gradient-to-r from-orange-400 to-yellow-400" style={{ animationDelay: '0.6s' }} />
              <div className="h-2 w-12 animate-pulse rounded-full bg-gradient-to-r from-pink-400 to-purple-400" style={{ animationDelay: '0.9s' }} />
              <div className="h-3 w-3 animate-pulse rounded-full bg-gradient-to-r from-yellow-400 to-orange-400" style={{ animationDelay: '1.2s' }} />
            </div>

            {/* Fun stats or badges */}
            <div className="mt-8 flex items-center justify-center gap-4 flex-wrap">
              <div className="px-4 py-2 rounded-full bg-gradient-to-r from-pink-400/20 to-orange-400/20 backdrop-blur-sm border border-white/20">
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-200">🔥 Live Data</span>
              </div>
              <div className="px-4 py-2 rounded-full bg-gradient-to-r from-yellow-400/20 to-pink-400/20 backdrop-blur-sm border border-white/20">
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-200">⚡ Fast Analytics</span>
              </div>
              <div className="px-4 py-2 rounded-full bg-gradient-to-r from-orange-400/20 to-yellow-400/20 backdrop-blur-sm border border-white/20">
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-200">🎯 Smart Insights</span>
              </div>
            </div>
          </div>

          {/* Server Card Dashboard with enhanced decorative elements */}
          <div className="relative">
            {/* Multiple floating decorative orbs around the main content */}
            <div className="animate-float absolute -top-12 -left-12 h-24 w-24 rounded-full bg-gradient-to-br from-pink-400/30 to-orange-400/30 blur-xl" />
            <div className="animate-bounce-gentle absolute -top-8 -right-16 h-20 w-20 rounded-full bg-gradient-to-br from-yellow-400/25 to-pink-400/25 blur-xl" />
            <div className="animate-pulse-gentle absolute -bottom-12 -left-16 h-28 w-28 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
            <div className="animate-spin-very-slow absolute -bottom-8 -right-12 h-24 w-24 rounded-full bg-gradient-to-br from-purple-400/15 to-pink-400/15 blur-xl" />
            
            {/* Small accent decorations */}
            <div className="animate-float absolute top-4 left-4 h-4 w-4 rounded-full bg-gradient-to-br from-pink-400/60 to-orange-400/60 blur-sm" />
            <div className="animate-bounce-gentle absolute top-8 right-8 h-3 w-3 rotate-45 rounded-sm bg-gradient-to-br from-yellow-400/70 to-pink-400/70 blur-sm" />
            
            <ServerCardDashboard searchParams={searchParams} />
          </div>
        </div>
      </div>
    </div>
  );
}
