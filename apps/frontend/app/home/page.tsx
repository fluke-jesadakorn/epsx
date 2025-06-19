'use client';

import { useEffect, useState } from 'react';
import { SkeletonLoader } from '@/components/common/Skeleton';
import DataRankTable from '@/components/home/DataRankTable';
import HeroSection from '@/components/home/HeroSection';
import DataTechSection from '@/components/home/DataTechSection';
import PricingSection from '@/components/home/PricingSection';
import EpsCardSection from '@/components/home/EpsCardSection';
import ChatSection from '@/components/home/ChatSection';
import { fetchStockScreenerData } from '@/app/actions/stock';

// Define columns for home page - showing a compact view
const homeColumns = [
  { key: 'number' as const, header: 'No.' },
  { key: 'symbol' as const, header: 'Symbol' },
  { key: 'name' as const, header: 'Name' },
  { key: 'valueIndex' as const, header: 'Value Index' },
  {
    key: 'growthRate' as const,
    header: 'Growth Rate',
    tooltip: 'Value Change Percentage',
  },
  {
    key: 'marketSize' as const,
    header: 'Market Size',
    tooltip: 'Total Market Presence',
  },
  {
    key: 'entryPhase' as const,
    header: 'Entry Phase',
    tooltip: 'Optimal Entry Time',
  },
  {
    key: 'phaseStatus' as const,
    header: 'Phase Status',
    tooltip: 'Current Phase Status',
  },
  {
    key: 'chart' as const,
    header: 'Analytics',
    tooltip: 'Open Analytics View',
  },
];

function HomePage() {
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const result = await fetchStockScreenerData();
        setData(result);
      } catch (error) {
        console.error('Error fetching data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  if (loading) {
    return <SkeletonLoader />;
  }

  // Get the last 10 items from the data
  const lastTenItems = data.slice(-10);

  return (
    <div className="relative min-h-screen">
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
        <DataTechSection />
        <div className="p-4 sm:p-6 lg:p-8 max-w-7xl mx-auto my-8 sm:my-16">
          <EpsCardSection initialData={data} />
        </div>
        <PricingSection />
        <div className="p-4 sm:p-6 lg:p-8 max-w-7xl mx-auto my-8 sm:my-16">
          <DataRankTable
            data={lastTenItems}
            columns={homeColumns}
            defaultView="card"
          />
        </div>
      </div>
    </div>
  );
}

export default HomePage;
