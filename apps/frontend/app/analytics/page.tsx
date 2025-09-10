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
    <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - Analytics theme */}
        <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-blue-400/30 to-indigo-400/30 blur-3xl" />
        <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-purple-400/25 to-pink-400/25 blur-3xl" />
        <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-cyan-400/20 to-teal-400/20 blur-3xl" />
        <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-indigo-400/15 to-blue-400/15 blur-3xl" />

        {/* Mesh gradient overlays for depth */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(59,130,246,0.1)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(168,85,247,0.08)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(99,102,241,0.06)_0%,_transparent_60%)]" />

        {/* Decorative geometric shapes */}
        <div className="absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-blue-300/10 to-indigo-300/10" />
        <div className="absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-purple-300/10 to-pink-300/10" />
      </div>

      {/* Main content */}
      <div className="relative z-10">
        <div className="container mx-auto max-w-7xl px-4 py-8">
          <div className="mb-12">
            <div className="p-8 text-center">
              <div className="relative">
                {/* Background decorative elements */}
                <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-blue-400/20 to-indigo-400/20 blur-xl" />
                <div className="absolute -right-8 -top-4 h-20 w-20 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-xl" />
                
                <h1 className="mb-4 text-4xl font-bold tracking-wide sm:text-5xl insight-gradient-text">
                  Analytics Hub
                </h1>
                <p className="text-lg font-medium mx-auto max-w-2xl insight-gradient-secondary-text">
                  Sweet DeFi analytics with delicious insights and powerful data
                </p>
                
                {/* Decorative dots */}
                <div className="mt-6 flex items-center justify-center gap-4">
                  <div className="h-2 w-2 rounded-full bg-blue-400" />
                  <div className="h-3 w-3 rounded-full bg-purple-400" />
                  <div className="h-2 w-2 rounded-full bg-indigo-400" />
                </div>
              </div>
            </div>
          </div>

          <div className="relative">
            {/* Dashboard background decorations */}
            <div className="absolute -top-12 left-1/4 h-24 w-24 rotate-12 rounded-2xl bg-gradient-to-br from-blue-300/10 to-indigo-300/10" />
            <div className="absolute right-1/4 -bottom-12 h-20 w-20 rounded-full bg-gradient-to-br from-purple-300/10 to-pink-300/10" />
            
            <ServerCardDashboard searchParams={searchParams} />
          </div>
        </div>
      </div>
    </div>
  );
}
