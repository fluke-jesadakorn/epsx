import ChatSection from '@/components/home/ChatSection';
import { PublicRankingPreview } from '@/components/home/PublicRankingPreview';
import DataTechSection from '@/components/home/DataTechSection';
import HeroSection from '@/components/home/HeroSection';
import PricingSection from '@/components/home/PricingSection';
import { fetchPublicRankingData } from '@/app/actions/publicRanking';
import { StreamingWrapper } from '@/components/common/StreamingWrapper';

// ISR configuration for homepage - revalidate every 5 minutes
export const revalidate = 300;

export default async function HomePage() {
  // Server-side data fetching for better SSR performance
  const initialData = await fetchPublicRankingData(10, 10);
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="absolute -top-40 -left-40 w-96 h-96 bg-gradient-to-br from-orange-400/30 to-yellow-400/30 rounded-full blur-3xl animate-bounce-slow" />
        <div className="absolute top-20 -right-32 w-80 h-80 bg-gradient-to-br from-blue-400/25 to-cyan-400/25 rounded-full blur-3xl animate-float" />
        <div className="absolute bottom-20 left-20 w-72 h-72 bg-gradient-to-br from-purple-400/20 to-pink-400/20 rounded-full blur-3xl animate-pulse-gentle" />
        <div className="absolute top-1/2 right-1/4 w-64 h-64 bg-gradient-to-br from-green-400/15 to-emerald-400/15 rounded-full blur-3xl animate-float-reverse" />

        {/* Mesh gradient overlays for depth */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)] animate-pulse-slow" />
        <div
          className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)] animate-pulse-slow"
          style={{ animationDelay: '1s' }}
        />
        <div
          className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)] animate-pulse-slow"
          style={{ animationDelay: '2s' }}
        />

        {/* Decorative geometric shapes */}
        <div className="absolute top-1/4 left-1/4 w-32 h-32 bg-gradient-to-br from-orange-300/10 to-yellow-300/10 rounded-2xl rotate-45 animate-spin-slow" />
        <div className="absolute bottom-1/3 right-1/3 w-24 h-24 bg-gradient-to-br from-blue-300/10 to-cyan-300/10 rounded-full animate-bounce-gentle" />
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

        {/* EPS Cards Section - Temporarily commented out to use same data format as analytics
        <div className="container mx-auto px-4 py-12 animate-fade-in-delayed">
          <div className="relative">
            <div className="absolute -top-8 -left-8 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full blur-xl" />
            <div className="absolute -bottom-8 -right-8 w-20 h-20 bg-gradient-to-br from-blue-400/20 to-cyan-400/20 rounded-full blur-xl" />
            <ClientEpsCardSection initialData={data} />
          </div>
        </div>
        */}

        {/* Pricing Section with enhanced PancakeSwap styling */}
        <StreamingWrapper priority="medium" identifier="pricing">
          <div className="relative">
            {/* Add some floating elements around pricing */}
            <div className="absolute top-10 left-10 w-8 h-8 bg-gradient-to-br from-purple-400/30 to-pink-400/30 rounded-full animate-bounce-gentle" />
            <div className="absolute bottom-10 right-10 w-6 h-6 bg-gradient-to-br from-green-400/30 to-emerald-400/30 rounded-full animate-float" />
            <PricingSection />
          </div>
        </StreamingWrapper>

        {/* Data Rank Table with vibrant PancakeSwap-style card */}
        <div className="container mx-auto px-4 py-16 animate-fade-in-delayed-3">
          <div className="relative">
            {/* Enhanced background decorations */}
            <div className="absolute -top-12 left-1/4 w-24 h-24 bg-gradient-to-br from-orange-300/10 to-yellow-300/10 rounded-2xl rotate-12 animate-float-gentle" />
            <div className="absolute -bottom-12 right-1/4 w-20 h-20 bg-gradient-to-br from-blue-300/10 to-cyan-300/10 rounded-full animate-bounce-slow" />

            {/* Main card with PancakeSwap styling */}
            <div className="relative bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-3xl p-8 shadow-2xl border border-orange-200/50 dark:border-orange-400/20 overflow-hidden">
              {/* Card background pattern */}
              <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-blue-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-blue-900/10" />
              <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full blur-2xl" />
              <div className="absolute bottom-0 left-0 w-40 h-40 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full blur-2xl" />

              <div className="relative z-10">
                <div className="text-center mb-10">
                  {/* Updated title for public rankings */}
                  <h2 className="text-4xl sm:text-5xl font-bold mb-6 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent animate-gradient-x">
                    Track <span className="bg-gradient-to-r from-purple-500 to-purple-600 bg-clip-text text-transparent">Performance</span> <span className="bg-gradient-to-r from-pink-500 to-pink-600 bg-clip-text text-transparent">Growth</span> Rankings
                  </h2>
                  <p className="text-lg text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed">
                    Unlock deeper insights and optimize data center performance with real-time analytics and advanced data tracking systems for smarter operational decisions
                  </p>
                  <p className="text-sm text-blue-600 dark:text-blue-400 mt-2">
                    🔒 Upgrade to access Top 100 rankings with advanced insights
                  </p>

                  {/* Decorative elements */}
                  <div className="flex justify-center items-center gap-4 mt-6">
                    <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" />
                    <div
                      className="w-3 h-3 bg-yellow-400 rounded-full animate-pulse"
                      style={{ animationDelay: '0.5s' }}
                    />
                    <div
                      className="w-2 h-2 bg-blue-400 rounded-full animate-pulse"
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

