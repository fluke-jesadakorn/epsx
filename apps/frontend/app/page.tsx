import { StreamingWrapper } from '@/components/common/StreamingWrapper';
import DynamicPricingSection from '@/components/home/DynamicPricingSection';
import ServerTopPerformers from '@/components/home/ServerTopPerformers';

// DISABLE ISR caching to show real Data Analytics data immediately
export const revalidate = 0;

import type { StockFinancialData } from '@/types/financialChartData';

export default function HomePage() {
  // Use empty data for PublicRankingPreview component
  const initialData: StockFinancialData[] = [];
  return (
    <div>
      {/* Promotional Banner */}
      <div className="bg-gradient-to-r from-orange-500 via-red-500 to-pink-500 text-white">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-center">
            <div className="flex items-center gap-2">
              <span className="text-lg font-bold">25% OFF</span>
              <span className="text-sm opacity-90">
                Limited time offer on all plans!
              </span>
            </div>
          </div>
        </div>
      </div>

      <div className="relative min-h-screen overflow-hidden">
        {/* PancakeSwap-style vibrant background */}
        <div className="fixed inset-0 z-0">
          {/* Main gradient background */}
          <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

          {/* Floating gradient orbs - PancakeSwap style */}
          <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
          <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
          <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
          <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 blur-3xl" />

          {/* Mesh gradient overlays for depth */}
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)]" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)]" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)]" />

          {/* Decorative geometric shapes */}
          <div className="absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
          <div className="absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />
        </div>

        {/* Main content with PancakeSwap styling */}
        <div className="relative z-10">
          {/* Chat Section with enhanced PancakeSwap styling - TEMPORARILY DISABLED */}
          {/* <StreamingWrapper priority="high" identifier="chat">
          <ChatSection />
        </StreamingWrapper> */}

          {/* EPS Cards Section - Top Performing Companies */}
          <StreamingWrapper priority="medium" identifier="eps-cards">
            <ServerTopPerformers />
          </StreamingWrapper>

          {/* Dynamic Pricing Section with affiliate tracking */}
          <StreamingWrapper priority="medium" identifier="pricing">
            <DynamicPricingSection />
          </StreamingWrapper>

          {/* Data Rank Table with vibrant PancakeSwap-style card */}
          <div className="container mx-auto px-4 py-16">
            <div className="relative">
              {/* Enhanced background decorations */}
              <div className="absolute -top-12 left-1/4 h-24 w-24 rotate-12 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
              <div className="absolute right-1/4 -bottom-12 h-20 w-20 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />

              {/* Main card with PancakeSwap styling */}
              <div className="relative overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 p-8 shadow-2xl backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80">
                {/* Card background pattern */}
                <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-blue-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-blue-900/10" />
                <div className="absolute top-0 right-0 h-32 w-32 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10 blur-2xl" />
                <div className="absolute bottom-0 left-0 h-40 w-40 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10 blur-2xl" />

                <div className="relative z-10">
                  <div className="text-center space-y-8">
                    {/* Premium Unlock Section */}
                    <div className="space-y-6">
                      <div className="inline-flex items-center gap-3 bg-gradient-to-r from-yellow-500 to-orange-500 text-white px-6 py-3 rounded-full font-bold text-lg shadow-xl">
                        <span className="text-2xl">👑</span>
                        <span>UNLOCK PREMIUM ACCESS</span>
                        <span className="text-2xl">🚀</span>
                      </div>
                      
                      <h2 className="bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-5xl font-bold text-transparent sm:text-6xl dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
                        Get Full Analytics Power
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

                    {/* Call to Action */}
                    <div className="mt-12 space-y-6">
                      <div className="bg-gradient-to-r from-orange-500 to-pink-500 text-white p-8 rounded-3xl shadow-2xl">
                        <div className="space-y-4">
                          <h3 className="text-3xl font-bold">🔥 Limited Time Offer!</h3>
                          <p className="text-xl">Start from just <span className="text-4xl font-bold">$1/month</span></p>
                          
                          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mt-6">
                            <button className="bg-white text-orange-600 font-bold py-4 px-8 rounded-2xl text-lg hover:bg-orange-50 shadow-xl">
                              🚀 Start Free Trial
                            </button>
                            <button className="bg-yellow-400 text-orange-900 font-bold py-4 px-8 rounded-2xl text-lg hover:bg-yellow-300 shadow-xl">
                              💎 View Plans
                            </button>
                          </div>
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
    </div>
  );
}
