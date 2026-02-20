import PlanStatusBar from '@/components/analytics/plan-status-bar';
import ServerCardDashboard from '@/components/analytics/server-card-dashboard';
import { ProgressiveAuthBanner } from '@/components/auth/progressive-auth-banner';
import { AnalyticsAuthWrapper } from '@/components/auth/analytics-auth-wrapper';
import { WatchlistProvider } from '@/components/portfolio/watchlist-provider';
import { getMyPlanAccessAction } from '@/app/actions/plans';
import { getWatchlistAction } from '@/app/actions/watchlist';
import { BarChart3, TrendingUp, Sparkles } from 'lucide-react';

interface AnalyticsPageProps {
  searchParams: Promise<{
    page?: string;
    limit?: string;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: string;
    min_growth?: string;
    search?: string;
  }>;
}

export default async function AnalyticsPage({ searchParams }: AnalyticsPageProps) {
  const resolvedSearchParams = await searchParams;
  const [planAccess, watchlist] = await Promise.all([
    getMyPlanAccessAction(),
    getWatchlistAction(),
  ]);

  return (
    <AnalyticsAuthWrapper>
      <WatchlistProvider initial={watchlist}>
        <div className="relative min-h-screen bg-gray-50 dark:bg-slate-950">
          {/* Background */}
          <div className="fixed inset-0 z-0">
            <div className="absolute inset-0 bg-gradient-to-b from-white dark:from-slate-950 via-gray-50 dark:via-slate-900 to-white dark:to-slate-950" />
            <div className="absolute -top-40 -left-40 h-[400px] w-[400px] rounded-full bg-purple-600/15 blur-3xl" />
            <div className="absolute top-1/3 -right-32 h-[300px] w-[300px] rounded-full bg-blue-600/10 blur-3xl" />
            <div className="absolute inset-0 opacity-0 dark:opacity-100 bg-[radial-gradient(ellipse_at_center,_transparent_0%,_rgba(0,0,0,0.3)_100%)]" />
          </div>

          {/* Content */}
          <div className="relative z-10">
            <div className="mx-auto max-w-7xl px-4 py-6 sm:py-8">
              {/* Header */}
              <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500">
                    <BarChart3 className="h-5 w-5 text-white" />
                  </div>
                  <div>
                    <h1 className="text-2xl font-bold text-foreground">Analytics</h1>
                    <p className="text-sm text-slate-400">Top-performing stocks by EPS growth</p>
                  </div>
                </div>
                <div className="flex gap-2">
                  <div className="flex items-center gap-1.5 rounded-lg border border-emerald-500/20 bg-emerald-500/10 px-3 py-1.5">
                    <TrendingUp className="h-3.5 w-3.5 text-emerald-400" />
                    <span className="text-xs font-medium text-emerald-400">Live</span>
                  </div>
                  <div className="flex items-center gap-1.5 rounded-lg border border-purple-500/20 bg-purple-500/10 px-3 py-1.5">
                    <Sparkles className="h-3.5 w-3.5 text-purple-400" />
                    <span className="text-xs font-medium text-purple-400">AI-Powered</span>
                  </div>
                </div>
              </div>

              {/* Plan status */}
              <div className="mb-4">
                <PlanStatusBar planAccess={planAccess} />
              </div>

              <ProgressiveAuthBanner />

              <ServerCardDashboard searchParams={{ ...resolvedSearchParams }} />
            </div>
          </div>
        </div>
      </WatchlistProvider>
    </AnalyticsAuthWrapper>
  );
}
