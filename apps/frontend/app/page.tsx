import DynamicPricingSection from '@/components/home/dynamic-pricing-section';
import HeroSection from '@/components/home/hero-section';
import ServerTopPerformers from '@/components/home/server-top-performers';
import { Crown, Rocket } from 'lucide-react';
import { Suspense } from 'react';

import type { StockFinancialData } from '@/types/financialChartData';

// ISR: revalidate homepage every 60s for fresh analytics data
export const revalidate = 60;

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
  const refCode = (searchParams.ref ?? searchParams.affiliate ?? searchParams.aff) as string | undefined;

  // Use empty data for PublicRankingPreview component (unused in simplified homepage)
  const _initialData: StockFinancialData[] = [];

  return (
    <div>

      <div className="relative min-h-screen overflow-hidden bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
        {/* Main content */}
        <div className="relative z-[1]">
          {/* Hero Section - Renders immediately, no blocking */}
          <HeroSection />

          {/* EPS Cards Section - Wrapped in Suspense for independent loading */}
          <Suspense fallback={<TopPerformersLoading />}>
            <ServerTopPerformers />
          </Suspense>

          {/* Dynamic Pricing Section with affiliate tracking */}
          <DynamicPricingSection initialAffiliateCode={refCode} />

          {/* Data Rank Table with vibrant PancakeSwap-style card */}
          <div className="container mx-auto px-4 py-16">
            <div className="relative">
              {/* Main card */}
              <div className="overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 p-8 shadow-xl dark:border-orange-400/20 dark:bg-slate-800/80">

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
