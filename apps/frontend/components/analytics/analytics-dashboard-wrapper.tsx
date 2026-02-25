'use client';

import type { AnalyticsPagination, SymbolCardData } from '@/shared/types/analytics';
import { useAnalyticsTransition } from '@/shared/components/analytics/analytics-transition-provider';
import { AnalyticsCardGrid } from './analytics-card-grid';
import ServerPagination from './server-pagination';
import { Loader2 } from 'lucide-react';

interface AnalyticsDashboardWrapperProps {
    rankings: SymbolCardData[];
    pagination: AnalyticsPagination | null;
    currentParams: string;
}

export function AnalyticsDashboardWrapper({
    rankings,
    pagination,
    currentParams,
}: AnalyticsDashboardWrapperProps): React.ReactElement {
    const { pending } = useAnalyticsTransition();

    return (
        <div className="space-y-6">
            {/* Card grid with loading overlay */}
            <div className="relative">
                {pending && (
                    <div className="absolute inset-0 z-10 flex items-center justify-center rounded-2xl bg-white/60 dark:bg-slate-950/60 backdrop-blur-[2px]">
                        <div className="flex items-center gap-2 rounded-xl bg-white dark:bg-slate-800 px-4 py-2 shadow-lg border border-gray-200 dark:border-slate-700">
                            <Loader2 className="h-4 w-4 animate-spin text-purple-500" />
                            <span className="text-sm font-medium text-slate-600 dark:text-slate-300">Loading...</span>
                        </div>
                    </div>
                )}
                <div className={pending ? 'pointer-events-none' : ''}>
                    <AnalyticsCardGrid
                        rankings={rankings}
                        pagination={pagination ?? undefined}
                    />
                </div>
            </div>

            {/* Pagination */}
            {pagination && pagination.totalPages > 1 && (
                <div className="mt-8">
                    <ServerPagination
                        pagination={pagination}
                        currentParams={currentParams}
                    />
                </div>
            )}
        </div>
    );
}

export default AnalyticsDashboardWrapper;
