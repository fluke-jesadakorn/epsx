import StockRankTable from '@/components/home/StockRankTable';
import HeroSection from '@/components/home/HeroSection';
import PricingSection from '@/components/home/PricingSection';
import EpsCardSection from '@/components/home/EpsCardSection';
import ChatSection from '@/components/home/ChatSection';
import { fetchStockScreenerData } from '@/app/actions/stockData';
import { Suspense } from 'react';

// Define columns for home page - showing a compact view
const homeColumns = [
  { key: 'number' as const, header: 'No.' },
  { key: 'symbol' as const, header: 'Symbol' },
  { key: 'name' as const, header: 'Name' },
  { key: 'price' as const, header: 'Price' },
  {
    key: 'changePercent' as const,
    header: 'Change %',
    tooltip: 'Price Change Percentage',
  },
  {
    key: 'marketCap' as const,
    header: 'Market Cap',
    tooltip: 'Market Capitalization',
  },
  {
    key: 'startBuy' as const,
    header: 'Start Buy',
    tooltip: 'When to start buying',
  },
  {
    key: 'startAction' as const,
    header: 'Hold or Sell',
    tooltip: 'When to start holding/selling',
  },
  { key: 'chart' as const, header: 'Chart', tooltip: 'Open TradingView Chart' },
];

async function HomePage() {
  const data = await fetchStockScreenerData();
  // Get the last 10 items from the data
  const lastTenItems = data.slice(-10);

  return (
    <div className="relative">
      {/* Shared background gradients with animation */}
      <div className="fixed inset-0 z-0">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,_rgba(33,150,243,0.15)_0%,_transparent_35%)] animate-pulse-slow" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,_rgba(156,39,176,0.15)_0%,_transparent_35%)] animate-pulse-slow" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_bottom,_rgba(63,81,181,0.15)_0%,_transparent_35%)] animate-pulse-slow" />
        
        {/* Decorative floating elements */}
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-blue-500/5 rounded-full blur-3xl animate-float" />
        <div className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-purple-500/5 rounded-full blur-3xl animate-float-delayed" />
      </div>
      
      {/* Content container with z-index to appear above background */}
      <div className="relative z-10">
        <ChatSection />
        <HeroSection className="relative z-10" />
        <div className="p-4 sm:p-6 lg:p-8 max-w-7xl mx-auto">
          <EpsCardSection initialData={data} initialTotal={data.length} />
        </div>
        <PricingSection />
        <Suspense fallback={<div>Loading...</div>}>
          <div className="p-4 sm:p-6 lg:p-8 max-w-7xl mx-auto">
            <StockRankTable data={lastTenItems} columns={homeColumns} />
          </div>
        </Suspense>
      </div>
    </div>
  );
}

export default HomePage;

// Revalidate page every 5 minutes
export const revalidate = 300;
