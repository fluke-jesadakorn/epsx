import ServerCardDashboard from '@/components/analytics/server-card-dashboard';

interface PortfolioPageProps {
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

export default function PortfolioPage({ searchParams }: PortfolioPageProps) {
  return (
    <>
      <div className="relative min-h-screen overflow-hidden">
        {/* PancakeSwap-style vibrant background */}
        <div className="fixed inset-0 z-0">
          {/* Main gradient background */}
          <div className="absolute inset-0 bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

          {/* Floating gradient orbs - Portfolio theme */}
          <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-green-400/30 to-emerald-400/30 blur-3xl" />
          <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-teal-400/25 to-cyan-400/25 blur-3xl" />
          <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-emerald-400/20 to-green-400/20 blur-3xl" />
          <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-teal-400/15 blur-3xl" />

          {/* Mesh gradient overlays for depth */}
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(34,197,94,0.1)_0%,_transparent_50%)]" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(20,184,166,0.08)_0%,_transparent_50%)]" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(16,185,129,0.06)_0%,_transparent_60%)]" />

          {/* Decorative geometric shapes */}
          <div className="absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-green-300/10 to-emerald-300/10" />
          <div className="absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-teal-300/10 to-cyan-300/10" />
        </div>

        {/* Main content */}
        <div className="relative z-10">
          <div className="container mx-auto max-w-7xl px-4 py-8">
            <div className="mb-12">
              <div className="p-8 text-center">
                <div className="relative">
                  {/* Background decorative elements */}
                  <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-green-400/20 to-emerald-400/20 blur-xl" />
                  <div className="absolute -top-4 -right-8 h-20 w-20 rounded-full bg-gradient-to-br from-teal-400/20 to-cyan-400/20 blur-xl" />

                  <h1 className="insight-gradient-text mb-4 text-4xl font-bold tracking-wide sm:text-5xl">
                    Portfolio Tracker
                  </h1>
                  <p className="insight-gradient-secondary-text mx-auto max-w-2xl text-lg font-medium">
                    Track your positive growth investments with surplus-only data
                    and real-time insights
                  </p>

                  {/* Decorative dots */}
                  <div className="mt-6 flex items-center justify-center gap-4">
                    <div className="h-2 w-2 rounded-full bg-green-400" />
                    <div className="h-3 w-3 rounded-full bg-emerald-400" />
                    <div className="h-2 w-2 rounded-full bg-teal-400" />
                  </div>
                </div>
              </div>
            </div>

            <div className="relative">
              {/* Dashboard background decorations */}
              <div className="absolute -top-12 left-1/4 h-24 w-24 rotate-12 rounded-2xl bg-gradient-to-br from-green-300/10 to-emerald-300/10" />
              <div className="absolute right-1/4 -bottom-12 h-20 w-20 rounded-full bg-gradient-to-br from-teal-300/10 to-cyan-300/10" />

              <ServerCardDashboard searchParams={searchParams} isPortfolio={true} />
            </div>
          </div>
        </div>
      </div>
    </>
  );
}