import { StreamingWrapper } from '@/components/common/streaming-wrapper';
import DynamicPricingSection from '@/components/home/dynamic-pricing-section';
import HeroSection from '@/components/home/hero-section';
import ServerTopPerformers from '@/components/home/server-top-performers';
import { Crown, Rocket } from 'lucide-react';
import { Suspense } from 'react';

import type { StockFinancialData } from '@/types/financialChartData';

// DISABLE ISR caching to show real Data Analytics data immediately
export const revalidate = 0;

// Loading skeleton for Top Performers section
function TopPerformersLoading() {
  return (
    <div className="container mx-auto px-4 py-12">
      <div className="relative">
        <div className="flex w-full flex-col gap-8">
          <div className="mb-6 space-y-4 text-center">
            <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
              Top Performing Companies
            </h2>
            <p className="text-muted-foreground mx-auto max-w-2xl">
              Discover the data leaders with exceptional growth and performance metrics
            </p>
            <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
          </div>
          <div className="grid grid-cols-1 justify-items-center gap-4 px-2 sm:grid-cols-2 sm:px-0 lg:grid-cols-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="w-full max-w-sm animate-pulse">
                <div className="h-64 rounded-2xl bg-slate-700/50" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default async function HomePage(props: {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>
}) {
  const searchParams = await props.searchParams;
  const refCode = (searchParams?.ref || searchParams?.affiliate || searchParams?.aff) as string | undefined;

  // Use empty data for PublicRankingPreview component (unused in simplified homepage)
  const _initialData: StockFinancialData[] = [];

  return (
    <div>

      <div className="relative min-h-screen overflow-hidden">
        {/* PancakeSwap-style vibrant background */}
        <div className="fixed inset-0 z-0">
          {/* Main gradient background */}
          <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

          {/* Floating gradient orbs - PancakeSwap style (dimmed in dark mode) */}
          <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 dark:from-orange-600/10 dark:to-yellow-600/10 blur-3xl" />
          <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 dark:from-blue-700/10 dark:to-cyan-700/10 blur-3xl" />
          <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 dark:from-purple-700/10 dark:to-pink-700/10 blur-3xl" />
          <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 dark:from-green-700/8 dark:to-emerald-700/8 blur-3xl" />

          {/* Mesh gradient overlays for depth (hidden in dark mode) */}
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)] dark:bg-none" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)] dark:bg-none" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)] dark:bg-none" />

          {/* Decorative geometric shapes (dimmed in dark mode) */}
          <div className="absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10 dark:from-orange-800/5 dark:to-yellow-800/5" />
          <div className="absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10 dark:from-blue-800/5 dark:to-cyan-800/5" />
        </div>

        {/* Main content with PancakeSwap styling */}
        <div className="relative z-[1]">
          {/* Hero Section - Renders immediately, no blocking */}
          <HeroSection />

          {/* EPS Cards Section - Wrapped in Suspense for independent loading */}
          <Suspense fallback={<TopPerformersLoading />}>
            <ServerTopPerformers />
          </Suspense>

          {/* Dynamic Pricing Section with affiliate tracking */}
          <StreamingWrapper priority="medium" identifier="pricing">
            <DynamicPricingSection initialAffiliateCode={refCode} />
          </StreamingWrapper>

          {/* Data Rank Table with vibrant PancakeSwap-style card */}
          <div className="container mx-auto px-4 py-16">
            <div className="relative">
              {/* Enhanced background decorations */}
              <div className="absolute -top-12 left-1/4 h-24 w-24 rotate-12 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10 dark:from-orange-700/5 dark:to-yellow-700/5" />
              <div className="absolute right-1/4 -bottom-12 h-20 w-20 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10 dark:from-blue-700/5 dark:to-cyan-700/5" />

              {/* Main card with PancakeSwap styling */}
              <div className="relative overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 p-8 shadow-2xl backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80">
                {/* Card background pattern */}
                <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-blue-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-blue-900/10" />
                <div className="absolute top-0 right-0 h-32 w-32 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10 dark:from-orange-700/5 dark:to-yellow-700/5 blur-2xl" />
                <div className="absolute bottom-0 left-0 h-40 w-40 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10 dark:from-blue-700/5 dark:to-cyan-700/5 blur-2xl" />

                <div className="relative z-10">
                  <div className="text-center space-y-8">
                    {/* Premium Unlock Section */}
                    <div className="space-y-6">
                      <div className="inline-flex items-center gap-3 bg-gradient-to-r from-yellow-500 to-orange-500 text-white px-6 py-3 rounded-full font-bold text-lg shadow-xl">
                        <span className="text-2xl">👑</span>
                        <span>UNLOCK PREMIUM ACCESS</span>
                        <span className="text-2xl">🚀</span>
                      </div>

                      <h2 className="flex flex-col items-center justify-center gap-4 bg-clip-text text-5xl font-bold text-transparent sm:flex-row sm:text-6xl">
                        <Crown className="h-12 w-12 text-yellow-500 sm:h-16 sm:w-16" />
                        <span className="bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
                          Get Full Analytics Power
                        </span>
                        <Rocket className="h-12 w-12 text-orange-500 sm:h-16 sm:w-16" />
                      </h2>

                      <p className="mx-auto max-w-4xl text-xl leading-relaxed text-gray-600 dark:text-gray-300">
                        <strong>Stop missing out!</strong> Join thousands of investors who are already making data-driven decisions with our premium analytics platform.
                      </p>
                    </div>

                    {/* Benefits Grid */}
                    <div className="grid md:grid-cols-3 gap-6 mt-12">
                      <div className="bg-gradient-to-br from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 p-6 rounded-2xl border border-green-200 dark:border-green-700">
                        <div className="text-4xl mb-4">📊</div>
                        <h3 className="text-xl font-bold text-green-800 dark:text-green-300 mb-2">Top 100 Rankings</h3>
                        <p className="text-green-700 dark:text-green-400">Access complete rankings with detailed analytics and growth predictions</p>
                      </div>

                      <div className="bg-gradient-to-br from-blue-50 to-cyan-50 dark:from-blue-900/20 dark:to-cyan-900/20 p-6 rounded-2xl border border-blue-200 dark:border-blue-700">
                        <div className="text-4xl mb-4">⚡</div>
                        <h3 className="text-xl font-bold text-blue-800 dark:text-blue-300 mb-2">Real-time Data</h3>
                        <p className="text-blue-700 dark:text-blue-400">Live market updates and instant alerts on performance changes</p>
                      </div>

                      <div className="bg-gradient-to-br from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 p-6 rounded-2xl border border-purple-200 dark:border-purple-700">
                        <div className="text-4xl mb-4">🎯</div>
                        <h3 className="text-xl font-bold text-purple-800 dark:text-purple-300 mb-2">Advanced Insights</h3>
                        <p className="text-purple-700 dark:text-purple-400">AI-powered recommendations and trend analysis</p>
                      </div>
                    </div>

                    <div className="flex items-center justify-center gap-6 text-sm text-gray-600 dark:text-gray-400">
                      <div className="flex items-center gap-2">
                        <span className="text-green-500">✓</span>
                        <span>No Credit Card Required</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-500">✓</span>
                        <span>Instant Access</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-500">✓</span>
                        <span>Cancel Anytime</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
