/* eslint-disable @typescript-eslint/no-unnecessary-condition, @typescript-eslint/strict-boolean-expressions, complexity, sonarjs/cognitive-complexity */
import { getRankingsAction } from '@/app/actions/analytics';
import { StockDataCard } from '@/shared/components';

interface QuarterlyPerformance {
  quarter: string;
  date: string;
  price: number;
  eps: number;
  eps_growth: number;
  price_growth: number;
}

interface NextQuarterEstimate {
  quarter: string;
  estimated_eps: number;
  announcement_date: string;
  announcement_timestamp: number;
  days_until_announcement: number;
  confidence: string;
}

interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  active_status: string;
  quarterly_performance: QuarterlyPerformance[];
  next_quarter_estimate?: NextQuarterEstimate;
  currency: string;
}

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
      <div className="grid grid-cols-1 justify-items-center gap-6 px-4 sm:grid-cols-2 lg:grid-cols-3">
        {top3Data.map(cardData => {
          const latestQuarter = cardData.quarterly_performance[0];

          return (
            <StockDataCard
              key={cardData.symbol}
              symbol={cardData.symbol}
              rank={cardData.rank}
               
              epsGrowth={latestQuarter?.eps_growth || 0}
               
              price={latestQuarter?.price || 0}
              currency={cardData.currency}
              daysUntilNextAction={cardData.next_quarter_estimate?.days_until_announcement}
              // Always use premium style for top performers on homepage
              variant="premium"
            />
          );
        })}
      </div>
    </div>
  );
};

export default async function ServerTopPerformers({ className }: ServerTopPerformersProps) {
  let data: SymbolCardData[] = [];
  let error: string | null = null;

  try {
    const result = await getRankingsAction({
      page: 1,
      limit: 3,
      sort_by: 'growth_factor'
    });
     
    if (result.data && Array.isArray(result.data)) {
      data = (result.data as unknown[]).map((ranking: unknown, index: number) => {
        const rankingObj = ranking as Record<string, unknown>;
        const qData = Array.isArray(rankingObj?.quarterly_performance) ? rankingObj.quarterly_performance : Array.isArray(rankingObj?.quarterly_data) ? rankingObj.quarterly_data : [];

        return {
          rank: typeof rankingObj?.rank === 'number' ? rankingObj.rank : typeof rankingObj?.ranking_position === 'number' ? rankingObj.ranking_position : index + 1,
          symbol: typeof rankingObj?.symbol === 'string' ? rankingObj.symbol : '',
          latest_date: typeof qData[0] === 'object' && qData[0] !== null && 'date' in qData[0] ? (qData[0] as Record<string, unknown>).date : typeof rankingObj?.latest_date === 'string' ? rankingObj.latest_date : new Date().toISOString(),
          value: typeof rankingObj?.value === 'number' ? rankingObj.value : typeof rankingObj?.price_current === 'number' ? rankingObj.price_current : 0,
          active_status: typeof rankingObj?.active_status === 'string' ? rankingObj.active_status : 'unknown',
          quarterly_performance: (qData as unknown[]).map((q: unknown) => {
            const qObj = q as Record<string, unknown>;
            return {
              quarter: typeof qObj?.quarter === 'string' ? qObj.quarter : '',
              date: typeof qObj?.date === 'string' ? qObj.date : '',
              price: typeof qObj?.price === 'number' ? qObj.price : 0,
              eps: typeof qObj?.eps === 'number' ? qObj.eps : 0,
              eps_growth: typeof qObj?.eps_growth === 'number' ? qObj.eps_growth : 0,
              price_growth: typeof qObj?.price_growth === 'number' ? qObj.price_growth : 0,
            };
          }),
          next_quarter_estimate: rankingObj?.next_quarter_estimate && typeof rankingObj.next_quarter_estimate === 'object' ? {
            quarter: typeof (rankingObj.next_quarter_estimate as Record<string, unknown>).quarter === 'string' ? (rankingObj.next_quarter_estimate as Record<string, unknown>).quarter : '',
            estimated_eps: typeof (rankingObj.next_quarter_estimate as Record<string, unknown>).estimated_eps === 'number' ? (rankingObj.next_quarter_estimate as Record<string, unknown>).estimated_eps : 0,
            announcement_date: typeof (rankingObj.next_quarter_estimate as Record<string, unknown>).announcement_date === 'string' ? (rankingObj.next_quarter_estimate as Record<string, unknown>).announcement_date : '',
            announcement_timestamp: typeof (rankingObj.next_quarter_estimate as Record<string, unknown>).announcement_timestamp === 'number' ? (rankingObj.next_quarter_estimate as Record<string, unknown>).announcement_timestamp : 0,
            days_until_announcement: typeof (rankingObj.next_quarter_estimate as Record<string, unknown>).days_until_announcement === 'number' ? (rankingObj.next_quarter_estimate as Record<string, unknown>).days_until_announcement : 0,
            confidence: typeof (rankingObj.next_quarter_estimate as Record<string, unknown>).confidence === 'string' ? (rankingObj.next_quarter_estimate as Record<string, unknown>).confidence : 'Medium'
          } : undefined,
          currency: 'USD'
        };
      });
    } else {
      const resultObj = result as Record<string, unknown>;
      error = typeof resultObj?.message === 'string' ? resultObj.message : 'No valid data received';
    }
  } catch (err) {
      // Error logged silently
    error = err instanceof Error ? err.message : 'Unknown error';
  }

  return (
    <div className={`container mx-auto px-4 py-12 ${className ?? ''}`}>
      <div className="relative">
        <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 dark:from-orange-600/10 dark:to-yellow-600/10 blur-xl" />
        <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 dark:from-blue-700/10 dark:to-cyan-700/10 blur-xl" />

        {error ? (
          <div className="flex w-full flex-col gap-8">
            <div className="mb-6 space-y-4 text-center">
              <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
                Top Performing Companies
              </h2>
              <p className="text-muted-foreground mx-auto max-w-2xl">
                Discover the data leaders with exceptional growth and performance metrics
              </p>
              <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
              <p className="mt-8 text-gray-600 dark:text-gray-400">
                Unable to load data at this time. Please try again later.
              </p>
            </div>
          </div>
        ) : data.length === 0 ? (
          <div className="flex w-full flex-col gap-8">
            <div className="mb-6 space-y-4 text-center">
              <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
                Top Performing Companies
              </h2>
              <p className="text-muted-foreground mx-auto max-w-2xl">
                Discover the data leaders with exceptional growth and performance metrics
              </p>
              <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
              <p className="mt-8 text-gray-600 dark:text-gray-400">
                No data available at this time.
              </p>
            </div>
          </div>
        ) : (
          <TopPerformersBox top3Data={data} />
        )}
      </div>
    </div>
  );
}