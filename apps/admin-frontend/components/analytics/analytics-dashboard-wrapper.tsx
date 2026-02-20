'use client';

import type { AnalyticsPagination, SymbolCardData } from '@/shared/types/analytics';
import { AnalyticsCardGrid } from './analytics-card-grid';

import ServerPagination from './server-pagination';

interface AnalyticsDashboardWrapperProps {
    rankings: SymbolCardData[];
    pagination: AnalyticsPagination | null;
    currentParams: string;
}

/**
 * Client-side wrapper for the analytics dashboard
 * Handles plan-aware rendering with the new AnalyticsCardGrid
 */
export function AnalyticsDashboardWrapper({
    rankings,
    pagination,
    currentParams,
}: AnalyticsDashboardWrapperProps): React.ReactElement {
    return (
        <div className="space-y-6">

            {/* Analytics Card Grid with plan-aware locking */}
            <AnalyticsCardGrid
                rankings={rankings}
                pagination={pagination ?? undefined}
            />

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
