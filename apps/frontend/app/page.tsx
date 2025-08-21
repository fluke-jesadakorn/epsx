import {
  fetchEpsCardData,
  fetchPublicRankingData,
} from '@/app/actions/publicRanking';
import { StreamingWrapper } from '@/components/common/StreamingWrapper';
import ChatSection from '@/components/home/ChatSection';
import ClientEpsCardSection from '@/components/home/ClientEpsCardSection';
import DataTechSection from '@/components/home/DataTechSection';
import HeroSection from '@/components/home/HeroSection';
import PricingSection from '@/components/home/PricingSection';
import { PublicRankingPreview } from '@/components/home/PublicRankingPreview';

// ISR configuration for homepage - revalidate every 5 minutes
export const revalidate = 300;

export default async function HomePage() {
  // Server-side data fetching for better SSR performance - Public ranks 101-105
  const [initialData, epsCardData] = await Promise.all([
    fetchPublicRankingData(1, 5), // For PublicRankingPreview - ranks 101-105
    fetchEpsCardData(1, 3), // For ClientEpsCardSection - ranks 101-103
  ]);
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="animate-bounce-slow absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="animate-float absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
        <div className="animate-pulse-gentle absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
        <div className="animate-float-reverse absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 blur-3xl" />

        {/* Mesh gradient overlays for depth */}
        <div className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)]" />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)]"
          style={{ animationDelay: '1s' }}
        />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)]"
          style={{ animationDelay: '2s' }}
        />

        {/* Decorative geometric shapes */}
        <div className="animate-spin-slow absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
        <div className="animate-bounce-gentle absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />
      </div>

      {/* Main content with PancakeSwap styling */}
      <div className="relative z-10">
        {/* Chat Section with enhanced PancakeSwap styling */}
        <StreamingWrapper priority="high" identifier="chat">
          <ChatSection />
        </StreamingWrapper>

        {/* Hero Section with vibrant PancakeSwap animations */}
        <StreamingWrapper priority="high" identifier="hero">
          <HeroSection className="relative z-10" />
        </StreamingWrapper>

        {/* Data Tech Section with gradient accents */}
        <StreamingWrapper priority="medium" identifier="data-tech">
          <DataTechSection />
        </StreamingWrapper>

        {/* EPS Cards Section - Top Performing Companies */}
        <StreamingWrapper priority="medium" identifier="eps-cards">
          <div className="animate-fade-in-delayed container mx-auto px-4 py-12">
            <div className="relative">
              <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
              <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl" />
              <ClientEpsCardSection initialData={epsCardData} />
            </div>
          </div>
        </StreamingWrapper>

        {/* Pricing Section with enhanced PancakeSwap styling */}
        <StreamingWrapper priority="medium" identifier="pricing">
          <div className="relative">
            {/* Add some floating elements around pricing */}
            <div className="animate-bounce-gentle absolute top-10 left-10 h-8 w-8 rounded-full bg-gradient-to-br from-purple-400/30 to-pink-400/30" />
            <div className="animate-float absolute right-10 bottom-10 h-6 w-6 rounded-full bg-gradient-to-br from-green-400/30 to-emerald-400/30" />
            <PricingSection />
          </div>
        </StreamingWrapper>

        {/* Data Rank Table with vibrant PancakeSwap-style card */}
        <div className="animate-fade-in-delayed-3 container mx-auto px-4 py-16">
          <div className="relative">
            {/* Enhanced background decorations */}
            <div className="animate-float-gentle absolute -top-12 left-1/4 h-24 w-24 rotate-12 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
            <div className="animate-bounce-slow absolute right-1/4 -bottom-12 h-20 w-20 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />

            {/* Main card with PancakeSwap styling */}
            <div className="relative overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 p-8 shadow-2xl backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80">
              {/* Card background pattern */}
              <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-blue-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-blue-900/10" />
              <div className="absolute top-0 right-0 h-32 w-32 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10 blur-2xl" />
              <div className="absolute bottom-0 left-0 h-40 w-40 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10 blur-2xl" />

              <div className="relative z-10">
                <div className="mb-10 text-center">
                  {/* Updated title for public rankings */}
                  <h2 className="animate-gradient-x mb-6 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-4xl font-bold text-transparent sm:text-5xl dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
                    Track{' '}
                    <span className="bg-gradient-to-r from-purple-500 to-purple-600 bg-clip-text text-transparent">
                      Performance
                    </span>{' '}
                    <span className="bg-gradient-to-r from-pink-500 to-pink-600 bg-clip-text text-transparent">
                      Growth
                    </span>{' '}
                    Rankings
                  </h2>
                  <p className="mx-auto max-w-3xl text-lg leading-relaxed text-gray-600 dark:text-gray-300">
                    Unlock deeper insights and optimize data center performance
                    with real-time analytics and advanced data tracking systems
                    for smarter operational decisions
                  </p>
                  <p className="mt-2 text-sm text-blue-600 dark:text-blue-400">
                    🔒 Upgrade to access Top 100 rankings with advanced insights
                  </p>
                  {/* Decorative elements */}
                  <div className="mt-6 flex items-center justify-center gap-4">
                    <div className="h-2 w-2 animate-pulse rounded-full bg-blue-400" />
                    <div
                      className="h-3 w-3 animate-pulse rounded-full bg-purple-400"
                      style={{ animationDelay: '0.5s' }}
                    />
                    <div
                      className="h-2 w-2 animate-pulse rounded-full bg-blue-400"
                      style={{ animationDelay: '1s' }}
                    />
                  </div>
                </div>

                <StreamingWrapper priority="low" identifier="rankings">
                  <PublicRankingPreview initialData={initialData} />
                </StreamingWrapper>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
