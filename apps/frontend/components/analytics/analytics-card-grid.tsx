'use client';

import { cn } from '@/lib/utils';
import { StockDataCard } from '@/shared/components';
import type { AnalyticsPagination, SymbolCardData } from '@/shared/types/analytics';
import { Sparkles } from 'lucide-react';
import { useMemo } from 'react';

interface AnalyticsCardGridProps {
    rankings: SymbolCardData[];
    pagination?: AnalyticsPagination;
    className?: string;
}

function StockCard({ cardData, delay = 0 }: { cardData: SymbolCardData; delay?: number }): React.ReactElement {
    const latestQuarter = cardData.quarterly_performance[0];
    const isPremium = cardData.rank <= 5;

    return (
        <div
            className="animate-in fade-in slide-in-from-bottom-4 duration-300"
            style={{ animationDelay: `${delay}ms` }}
        >
            <StockDataCard
                symbol={cardData.symbol}
                rank={cardData.rank}
                epsGrowth={latestQuarter.eps_growth || 0}
                price={latestQuarter.price || 0}
                currency={cardData.currency}
                // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                daysUntilNextAction={cardData.next_quarter_estimate.days_until_announcement ?? 0}
                companyName={cardData.company_name ?? cardData.name}
                variant={isPremium ? 'premium' : 'standard'}
            />
        </div>
    );
}

export function AnalyticsCardGrid({ rankings, className }: AnalyticsCardGridProps): React.ReactElement {
    // Sort all cards by rank (backend already filters by access level)
    const sortedCards = useMemo(() => {
        return [...rankings].sort((a, b) => a.rank - b.rank);
    }, [rankings]);

    // If no rankings data
    if (rankings.length === 0) {
        return (
            <div className="py-12 text-center">
                <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-slate-800/50">
                    <Sparkles className="h-8 w-8 text-slate-400" />
                </div>
                <p className="text-slate-400">No rankings data available</p>
            </div>
        );
    }

    return (
        <div className={cn('space-y-6', className)}>
            {/* All cards - backend already filters by user access level */}
            <div className="grid grid-cols-1 justify-items-center gap-4 px-2 sm:grid-cols-2 sm:gap-6 sm:px-0 lg:grid-cols-3 xl:grid-cols-3 2xl:grid-cols-5">
                {sortedCards.map((cardData, index) => (
                    <div
                        key={cardData.symbol}
                        className="w-full max-w-[400px]"
                    >
                        <StockCard cardData={cardData} delay={Math.min(index * 50, 500)} />
                    </div>
                ))}
            </div>
        </div>
    );
}

export default AnalyticsCardGrid;
