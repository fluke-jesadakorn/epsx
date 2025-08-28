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
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-950 dark:via-slate-900 dark:to-indigo-950">
      {/* Subtle background */}
      <div className="fixed inset-0 z-0">
        <div className="absolute inset-0 bg-gradient-to-br from-slate-50/80 via-blue-50/60 to-indigo-50/80 dark:from-gray-950/90 dark:via-slate-900/80 dark:to-indigo-950/90" />
        
        {/* Minimal decorative elements */}
        <div className="absolute top-1/4 left-1/6 h-32 w-32 rounded-full bg-gradient-to-br from-blue-400/10 to-indigo-400/10 blur-2xl" />
        <div className="absolute bottom-1/4 right-1/6 h-40 w-40 rounded-full bg-gradient-to-br from-indigo-400/10 to-blue-400/10 blur-2xl" />
      </div>

      {/* Main content */}
      <div className="relative z-10">
        <div className="container mx-auto px-4 py-8">
          {/* Professional header */}
          <div className="mb-8 text-center">
            <h1 className="mb-4 bg-gradient-to-r from-slate-800 via-blue-700 to-indigo-800 bg-clip-text text-4xl font-bold text-transparent sm:text-5xl dark:from-slate-200 dark:via-blue-300 dark:to-indigo-200">
              Analytics Dashboard
            </h1>
            
            <p className="mx-auto max-w-2xl text-lg text-slate-600 dark:text-slate-300">
              Professional stock analytics with 
              <span className="font-semibold text-blue-600 dark:text-blue-400"> real-time insights</span>
            </p>
            
            {/* Status badges */}
            <div className="mt-6 flex items-center justify-center gap-4 flex-wrap">
              <div className="px-3 py-1 rounded-full bg-green-100/80 border border-green-200/60 dark:bg-green-900/30 dark:border-green-800/50">
                <span className="text-sm font-medium text-green-700 dark:text-green-300">Live Data</span>
              </div>
              <div className="px-3 py-1 rounded-full bg-blue-100/80 border border-blue-200/60 dark:bg-blue-900/30 dark:border-blue-800/50">
                <span className="text-sm font-medium text-blue-700 dark:text-blue-300">Fast Analytics</span>
              </div>
              <div className="px-3 py-1 rounded-full bg-indigo-100/80 border border-indigo-200/60 dark:bg-indigo-900/30 dark:border-indigo-800/50">
                <span className="text-sm font-medium text-indigo-700 dark:text-indigo-300">Smart Insights</span>
              </div>
            </div>
          </div>

          <ServerCardDashboard searchParams={searchParams} />
        </div>
      </div>
    </div>
  );
}
