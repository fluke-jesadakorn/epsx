import DynamicPricingSection from '@/components/home/dynamic-pricing-section';
import HeroSection from '@/components/home/hero-section';
import ServerNewsSection from '@/components/home/server-news-section';
import ServerTopPerformers from '@/components/home/server-top-performers';
import { Suspense } from 'react';

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

          {/* News Section */}
          <Suspense fallback={null}>
            <ServerNewsSection />
          </Suspense>
        </div>
      </div>
    </div>
  );
}
