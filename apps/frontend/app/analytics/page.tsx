import { PlanGatedRankings } from '@/components/analytics/PlanGatedRankings';
import ServerCardDashboard from '@/components/analytics/ServerCardDashboard';

interface AnalyticsPageProps {
  searchParams: Promise<{
    page?: string;
    limit?: string;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: string;
    min_growth?: string;
    showFilters?: string;
    search?: string;
  }>;
}

export default async function AnalyticsPage({ searchParams }: AnalyticsPageProps) {
  const resolvedSearchParams = await searchParams;
  return (
    <>
      <div className="relative min-h-screen overflow-hidden">
        {/* PancakeSwap-style vibrant background */}
        <div className="fixed inset-0 z-0">
          {/* Main gradient background */}
          <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

          {/* Floating gradient orbs - Analytics theme (dimmed in dark mode) */}
          <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-blue-400/30 to-indigo-400/30 dark:from-blue-700/10 dark:to-indigo-700/10 blur-3xl" />
          <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-purple-400/25 to-pink-400/25 dark:from-purple-700/10 dark:to-pink-700/10 blur-3xl" />
          <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-cyan-400/20 to-teal-400/20 dark:from-cyan-700/10 dark:to-teal-700/10 blur-3xl" />
          <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-indigo-400/15 to-blue-400/15 dark:from-indigo-700/8 dark:to-blue-700/8 blur-3xl" />

          {/* Mesh gradient overlays for depth (hidden in dark mode) */}
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(59,130,246,0.1)_0%,_transparent_50%)] dark:bg-none" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(168,85,247,0.08)_0%,_transparent_50%)] dark:bg-none" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(99,102,241,0.06)_0%,_transparent_60%)] dark:bg-none" />

          {/* Decorative geometric shapes (dimmed in dark mode) */}
          <div className="absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-blue-300/10 to-indigo-300/10 dark:from-blue-800/5 dark:to-indigo-800/5" />
          <div className="absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-purple-300/10 to-pink-300/10 dark:from-purple-800/5 dark:to-pink-800/5" />
        </div>

        {/* Main content */}
        <div className="relative z-10">
          <div className="container mx-auto max-w-7xl px-4 py-8">
            <div className="mb-12">
              <div className="p-8 text-center">
                <div className="relative">
                  {/* Background decorative elements */}
                  <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-blue-400/20 to-indigo-400/20 dark:from-blue-700/10 dark:to-indigo-700/10 blur-xl" />
                  <div className="absolute -top-4 -right-8 h-20 w-20 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 dark:from-purple-700/10 dark:to-pink-700/10 blur-xl" />

                  <h1 className="insight-gradient-text mb-4 text-4xl font-bold tracking-wide sm:text-5xl">
                    Analytics Hub
                  </h1>
                  <p className="insight-gradient-secondary-text mx-auto max-w-2xl text-lg font-medium">
                    Sweet DeFi analytics with delicious insights and powerful
                    data
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
              <div className="absolute -top-12 left-1/4 h-24 w-24 rotate-12 rounded-2xl bg-gradient-to-br from-blue-300/10 to-indigo-300/10 dark:from-blue-800/5 dark:to-indigo-800/5" />
              <div className="absolute right-1/4 -bottom-12 h-20 w-20 rounded-full bg-gradient-to-br from-purple-300/10 to-pink-300/10 dark:from-purple-800/5 dark:to-pink-800/5" />

              {/* Wrap with PlanGatedRankings for upgrade prompts */}
              <PlanGatedRankings totalRankings={100}>
                <ServerCardDashboard
                  searchParams={{
                    ...resolvedSearchParams,
                  }}
                />
              </PlanGatedRankings>
            </div>
          </div>
        </div>
      </div >
    </>
  );
}

