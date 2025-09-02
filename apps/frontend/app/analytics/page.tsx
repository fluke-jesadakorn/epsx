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
    <div className="min-h-screen bg-gradient-to-br from-pink-100 via-orange-50 to-yellow-100 dark:from-pink-900/20 dark:via-orange-900/20 dark:to-yellow-900/20">
      {/* PancakeSwap + Windows Phone inspired background */}
      <div className="fixed inset-0 z-0">
        {/* PancakeSwap-style vibrant patterns */}
        <div className="absolute inset-0 opacity-20 dark:opacity-10">
          <div className="absolute top-20 left-20 h-32 w-32 rotate-45 bg-gradient-to-br from-pink-400 to-rose-500 rounded-3xl"></div>
          <div className="absolute top-40 right-32 h-24 w-24 rotate-12 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full"></div>
          <div className="absolute bottom-32 left-1/3 h-28 w-28 -rotate-12 bg-gradient-to-br from-orange-400 to-yellow-500 rounded-2xl"></div>
          <div className="absolute bottom-20 right-20 h-20 w-20 rotate-45 bg-gradient-to-br from-purple-500 to-pink-500 rounded-full"></div>
        </div>
        
        {/* Enhanced PancakeSwap-style floating elements */}
        <div className="absolute top-1/4 left-10 h-6 w-6 animate-pulse rounded-full bg-gradient-to-r from-cyan-400 to-blue-500 opacity-80"></div>
        <div className="absolute top-1/3 right-16 h-4 w-4 animate-bounce rounded-full bg-gradient-to-r from-yellow-400 to-orange-500 opacity-90" style={{ animationDelay: '1s' }}></div>
        <div className="absolute bottom-1/4 left-1/4 h-8 w-8 animate-pulse rounded-full bg-gradient-to-r from-green-400 to-teal-500 opacity-70" style={{ animationDelay: '2s' }}></div>
        <div className="absolute top-1/2 right-1/3 h-5 w-5 animate-bounce rounded-full bg-gradient-to-r from-pink-400 to-purple-500 opacity-60" style={{ animationDelay: '3s' }}></div>
      </div>

      {/* Main content */}
      <div className="relative z-10">
        <div className="container mx-auto px-4 py-8">
          {/* PancakeSwap inspired header with Windows Phone structure */}
          <div className="mb-12">
            {/* Vibrant header block */}
            <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-pink-500 via-orange-500 to-yellow-500 p-8 shadow-2xl">
              <div className="absolute inset-0 bg-gradient-to-r from-white/10 to-transparent"></div>
              <div className="absolute top-0 right-0 w-32 h-32 bg-white/5 rounded-full -translate-y-16 translate-x-16"></div>
              <div className="relative z-10">
                <h1 className="mb-4 text-4xl font-bold tracking-wide text-white sm:text-5xl drop-shadow-lg">
                  🥞 Analytics Hub
                </h1>
                <p className="text-lg font-medium text-white/95 max-w-2xl drop-shadow-sm">
                  Sweet DeFi analytics with delicious insights and powerful data
                </p>
                
                {/* PancakeSwap-style action tiles */}
                <div className="mt-8 grid grid-cols-1 sm:grid-cols-3 gap-4">
                  <div className="bg-gradient-to-br from-green-400 to-emerald-500 hover:from-green-500 hover:to-emerald-600 transition-all duration-300 p-4 cursor-pointer group rounded-2xl shadow-lg hover:shadow-xl hover:scale-105">
                    <div className="text-3xl mb-2">🔥</div>
                    <div className="text-white font-bold text-sm drop-shadow-sm">Live Data</div>
                  </div>
                  <div className="bg-gradient-to-br from-orange-400 to-red-500 hover:from-orange-500 hover:to-red-600 transition-all duration-300 p-4 cursor-pointer group rounded-2xl shadow-lg hover:shadow-xl hover:scale-105">
                    <div className="text-3xl mb-2">⚡</div>
                    <div className="text-white font-bold text-sm drop-shadow-sm">Fast Analytics</div>
                  </div>
                  <div className="bg-gradient-to-br from-purple-400 to-pink-500 hover:from-purple-500 hover:to-pink-600 transition-all duration-300 p-4 cursor-pointer group rounded-2xl shadow-lg hover:shadow-xl hover:scale-105">
                    <div className="text-3xl mb-2">🎯</div>
                    <div className="text-white font-bold text-sm drop-shadow-sm">Smart Insights</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <ServerCardDashboard searchParams={searchParams} />
        </div>
      </div>
    </div>
  );
}
