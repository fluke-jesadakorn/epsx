import PlanStatusBar from '@/components/analytics/plan-status-bar';
import ServerCardDashboard from '@/components/analytics/server-card-dashboard';
import { ProgressiveAuthBanner } from '@/components/auth/progressive-auth-banner';
import { FrontendAuthModal } from '@/components/auth/frontend-auth-modal';
import { AnalyticsAuthWrapper } from '@/components/auth/analytics-auth-wrapper';
import { getMyPlanAccessAction } from '@/app/actions/plans';
import { BarChart3, Sparkles, TrendingUp } from 'lucide-react';

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

  // Fetch plan access server-side
  const planAccess = await getMyPlanAccessAction();

  return (
    <AnalyticsAuthWrapper>
      <div className="relative min-h-screen overflow-hidden bg-slate-950">
        {/* Premium dark gradient background */}
        <div className="fixed inset-0 z-0">
          {/* Base gradient */}
          <div className="absolute inset-0 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950" />

          {/* Animated gradient orbs */}
          <div className="absolute -top-40 -left-40 h-[500px] w-[500px] rounded-full bg-gradient-to-br from-purple-600/20 to-pink-600/10 blur-3xl animate-pulse" />
          <div className="absolute top-1/4 -right-32 h-[400px] w-[400px] rounded-full bg-gradient-to-br from-blue-600/15 to-cyan-600/10 blur-3xl animate-pulse" style={{ animationDelay: '1s' }} />
          <div className="absolute bottom-0 left-1/3 h-[350px] w-[350px] rounded-full bg-gradient-to-br from-indigo-600/15 to-purple-600/10 blur-3xl animate-pulse" style={{ animationDelay: '2s' }} />

          {/* Grid pattern overlay */}
          <div
            className="absolute inset-0 opacity-[0.03]"
            style={{
              backgroundImage: `linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px),
                                linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)`,
              backgroundSize: '50px 50px'
            }}
          />

          {/* Radial gradient overlay for depth */}
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_transparent_0%,_rgba(0,0,0,0.4)_100%)]" />
        </div>

        {/* Main content */}
        <div className="relative z-10">
          <div className="container mx-auto max-w-7xl px-4 py-8">
            {/* Header Section */}
            <div className="mb-10">
              <div className="relative rounded-3xl border border-white/5 bg-slate-900/50 backdrop-blur-xl p-8 overflow-hidden">
                {/* Header background decorations */}
                <div className="absolute top-0 right-0 h-32 w-32 bg-gradient-to-br from-purple-500/10 to-pink-500/10 blur-2xl" />
                <div className="absolute bottom-0 left-0 h-24 w-24 bg-gradient-to-br from-blue-500/10 to-cyan-500/10 blur-2xl" />

                <div className="relative flex flex-col lg:flex-row lg:items-center lg:justify-between gap-6">
                  {/* Title and description */}
                  <div className="flex items-start gap-4">
                    <div className="flex h-14 w-14 items-center justify-center rounded-2xl bg-gradient-to-br from-purple-500 via-pink-500 to-orange-500 shadow-lg shadow-purple-500/25">
                      <BarChart3 className="h-7 w-7 text-white" />
                    </div>
                    <div>
                      <h1 className="text-3xl font-bold text-white sm:text-4xl">
                        Analytics Hub
                      </h1>
                      <p className="mt-1 text-slate-400 max-w-xl">
                        Discover top-performing stocks with real-time EPS growth analytics
                      </p>
                    </div>
                  </div>

                  {/* Quick stats badges */}
                  <div className="flex flex-wrap gap-3">
                    <div className="flex items-center gap-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20 px-4 py-2">
                      <TrendingUp className="h-4 w-4 text-emerald-400" />
                      <span className="text-sm font-medium text-emerald-400">Live Data</span>
                    </div>
                    <div className="flex items-center gap-2 rounded-xl bg-purple-500/10 border border-purple-500/20 px-4 py-2">
                      <Sparkles className="h-4 w-4 text-purple-400" />
                      <span className="text-sm font-medium text-purple-400">AI-Powered</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Dashboard Section */}
            <div className='mb-8'>
              <PlanStatusBar planAccess={planAccess} />
            </div>

            {/* Progressive Auth Banner - shown to unauthenticated users */}
            <ProgressiveAuthBanner />

            <div className="relative">
              <ServerCardDashboard
                searchParams={{
                  ...resolvedSearchParams,
                }}
              />
            </div>
          </div>
        </div>
      </div>
    </AnalyticsAuthWrapper>
  );
}

