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
    search?: string;
  };
}

export default function AnalyticsPage({ searchParams }: AnalyticsPageProps) {
  return (
    <div className="min-h-screen bg-gradient-to-br from-pink-50 via-orange-50 to-yellow-50 dark:from-gray-900 dark:via-purple-900/30 dark:to-pink-900/20">
      {/* PancakeSwap-inspired vibrant background */}
      <div className="fixed inset-0 z-0">
        <div className="absolute inset-0 bg-gradient-to-br from-pink-100/60 via-orange-100/50 to-yellow-100/60 dark:from-gray-900/90 dark:via-purple-900/40 dark:to-pink-900/30" />
        
        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="absolute -top-40 -left-40 h-96 w-96 animate-pulse rounded-full bg-gradient-to-br from-pink-400/30 to-orange-400/30 blur-3xl" />
        <div className="absolute top-20 -right-32 h-80 w-80 animate-bounce rounded-full bg-gradient-to-br from-yellow-400/25 to-pink-400/25 blur-3xl" style={{ animationDuration: '8s' }} />
        <div className="absolute bottom-20 left-20 h-72 w-72 animate-pulse rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-3xl" style={{ animationDelay: '2s' }} />
        
        {/* Mesh gradient overlays */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_20%_30%,_rgba(236,72,153,0.1)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_80%_20%,_rgba(251,191,36,0.08)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_60%_80%,_rgba(249,115,22,0.06)_0%,_transparent_60%)]" />
      </div>

      {/* Main content */}
      <div className="relative z-10">
        <div className="container mx-auto px-4 py-8">
          {/* PancakeSwap-inspired header */}
          <div className="mb-8 text-center">
            <div className="relative mb-6">
              {/* Floating decorative elements */}
              <div className="absolute -top-4 -left-8 h-6 w-6 animate-bounce rounded-full bg-gradient-to-br from-pink-400/60 to-orange-400/60 blur-sm" style={{ animationDelay: '0.5s' }} />
              <div className="absolute -top-6 -right-8 h-4 w-4 animate-pulse rounded-full bg-gradient-to-br from-yellow-400/70 to-pink-400/70 blur-sm" />
              
              <h1 className="mb-4 bg-gradient-to-r from-pink-600 via-orange-500 to-yellow-600 bg-clip-text text-4xl font-bold text-transparent sm:text-5xl dark:from-pink-400 dark:via-orange-400 dark:to-yellow-400">
                🚀 Analytics Hub
              </h1>
            </div>
            
            <p className="mx-auto max-w-2xl text-lg font-medium text-slate-700 dark:text-slate-200">
              Sweet analytics with delicious insights and 
              <span className="bg-gradient-to-r from-pink-600 to-orange-600 bg-clip-text font-bold text-transparent dark:from-pink-400 dark:to-orange-400"> real-time data</span>
            </p>
            
            {/* PancakeSwap-style badges */}
            <div className="mt-6 flex items-center justify-center gap-4 flex-wrap">
              <div className="group cursor-pointer rounded-full bg-gradient-to-r from-green-400 to-emerald-500 px-4 py-2 shadow-lg transition-all hover:scale-105 hover:shadow-xl">
                <span className="text-sm font-bold text-white">🔥 Live Data</span>
              </div>
              <div className="group cursor-pointer rounded-full bg-gradient-to-r from-blue-400 to-cyan-500 px-4 py-2 shadow-lg transition-all hover:scale-105 hover:shadow-xl">
                <span className="text-sm font-bold text-white">⚡ Fast Analytics</span>
              </div>
              <div className="group cursor-pointer rounded-full bg-gradient-to-r from-purple-400 to-pink-500 px-4 py-2 shadow-lg transition-all hover:scale-105 hover:shadow-xl">
                <span className="text-sm font-bold text-white">🎯 Smart Insights</span>
              </div>
            </div>

            {/* Decorative dots */}
            <div className="mt-6 flex items-center justify-center gap-2">
              <div className="h-2 w-2 animate-pulse rounded-full bg-gradient-to-r from-pink-400 to-orange-400" />
              <div className="h-1 w-8 animate-pulse rounded-full bg-gradient-to-r from-yellow-400 to-pink-400" style={{ animationDelay: '0.3s' }} />
              <div className="h-2 w-2 animate-pulse rounded-full bg-gradient-to-r from-orange-400 to-yellow-400" style={{ animationDelay: '0.6s' }} />
              <div className="h-1 w-8 animate-pulse rounded-full bg-gradient-to-r from-pink-400 to-purple-400" style={{ animationDelay: '0.9s' }} />
              <div className="h-2 w-2 animate-pulse rounded-full bg-gradient-to-r from-yellow-400 to-orange-400" style={{ animationDelay: '1.2s' }} />
            </div>
          </div>

          <ServerCardDashboard searchParams={searchParams} />
        </div>
      </div>
    </div>
  );
}
