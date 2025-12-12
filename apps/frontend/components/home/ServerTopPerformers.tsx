import { getAnalyticsData, type SymbolCardData } from '@/lib/unified-server-data';
import { StockDataCard } from '@/shared/components';

interface ServerTopPerformersProps {
  className?: string;
}

const TopPerformersBox = ({ top3Data }: { top3Data: SymbolCardData[] }) => {
  return (
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

      {/* Analytics-style card grid */}
      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:gap-6 sm:px-0">
        {top3Data.map(cardData => {
          const latestQuarter = cardData.quarterly_performance?.[0];
          
          return (
            <StockDataCard
              key={cardData.symbol}
              symbol={cardData.symbol}
              rank={cardData.rank}
              epsGrowth={latestQuarter?.eps_growth || 0}
              price={latestQuarter?.price || 0}
              currency={cardData.currency}
              daysUntilNextAction={cardData.next_quarter_estimate?.days_until_announcement}
              variant="premium"
            />
          );
        })}
      </div>
    </div>
  );
};

export default async function ServerTopPerformers({ className }: ServerTopPerformersProps) {
  try {
    // Fetch top 3 performing companies from backend
    const data = await getAnalyticsData({
      page: 1,
      limit: 3,
      sort_by: 'growth_factor',
    });

    if (!data || !data.rankings || data.rankings.length === 0) {
      return (
        <div className="container mx-auto px-4 py-12">
          <div className="text-center">
            <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
              Top Performing Companies
            </h2>
            <p className="text-muted-foreground mx-auto max-w-2xl mt-4">
              Discover the data leaders with exceptional growth and performance metrics
            </p>
            <div className="pancake-gradient mx-auto h-1 w-24 rounded-full mt-4" />
            <p className="mt-8 text-gray-600 dark:text-gray-400">
              Loading top performers...
            </p>
          </div>
        </div>
      );
    }

    // Get the top 3 performers
    const top3Data = data.rankings.slice(0, 3);

    return (
      <div className={`container mx-auto px-4 py-12 ${className || ''}`}>
        <div className="relative">
          <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
          <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl" />
          
          <TopPerformersBox top3Data={top3Data} />
        </div>
      </div>
    );
  } catch (error) {
    console.error('Failed to fetch top performers:', error);
    
    return (
      <div className="container mx-auto px-4 py-12">
        <div className="text-center">
          <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
            Top Performing Companies
          </h2>
          <p className="text-muted-foreground mx-auto max-w-2xl mt-4">
            Discover the data leaders with exceptional growth and performance metrics
          </p>
          <div className="pancake-gradient mx-auto h-1 w-24 rounded-full mt-4" />
          <p className="mt-8 text-gray-600 dark:text-gray-400">
            Unable to load data at this time. Please try again later.
          </p>
        </div>
      </div>
    );
  }
}