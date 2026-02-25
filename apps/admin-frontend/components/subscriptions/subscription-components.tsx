'use client';

import type { FilterContext, FilterStatus } from '@/hooks/use-subscription-management';
import type { SubscriptionResponse } from '@/shared/api/plans';
import { useRouter } from 'next/navigation';

interface SubscriptionStatsProps {
    total: number;
    active: number;
    expired: number;
    revenue: number;
}

export function SubscriptionStats({ total, active, expired, revenue }: SubscriptionStatsProps) {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div className="bg-card/80 rounded-3xl p-6 border border-primary/20 shadow-sm">
                <div className="text-secondary font-semibold mb-2">Total Subscriptions</div>
                <div className="text-3xl font-bold text-foreground mb-1">{total}</div>
                <div className="text-sm text-muted-foreground">All statuses</div>
            </div>

            <div className="bg-card/80 rounded-3xl p-6 border border-primary/20 shadow-sm">
                <div className="text-primary font-semibold mb-2">Active</div>
                <div className="text-3xl font-bold text-foreground mb-1">{active}</div>
                <div className="text-sm text-muted-foreground">Currently active</div>
            </div>

            <div className="bg-card/80 rounded-3xl p-6 border border-primary/20 shadow-sm">
                <div className="text-warning font-semibold mb-2">Expired</div>
                <div className="text-3xl font-bold text-foreground mb-1">{expired}</div>
                <div className="text-sm text-muted-foreground">Need renewal</div>
            </div>

            <div className="bg-card/80 rounded-3xl p-6 border border-primary/20 shadow-sm">
                <div className="text-success font-semibold mb-2">Est. Revenue</div>
                <div className="text-3xl font-bold text-foreground mb-1">${revenue}</div>
                <div className="text-sm text-muted-foreground">Monthly estimate</div>
            </div>
        </div>
    );
}

interface SubscriptionActionsProps {
    onRefresh: () => void;
    isLoading: boolean;
}

export function SubscriptionActions({ onRefresh }: SubscriptionActionsProps) {
    const router = useRouter();

    return (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
            <div
                className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-success/10 p-0.5 cursor-pointer"
                onClick={() => router.push('/subscriptions/new')}
            >
                <div className="relative bg-success text-success-foreground rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
                    <div className="p-8">
                        <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                            <span className="text-2xl">➕</span>
                        </div>
                        <h3 className="text-2xl font-bold mb-4">Create Subscription</h3>
                        <p className="text-success-foreground/80 mb-6">Assign plans to users with custom configurations</p>
                        <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                            New Subscription
                        </div>
                    </div>
                </div>
            </div>

            <div
                className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-secondary/10 p-0.5 cursor-pointer"
                onClick={onRefresh}
            >
                <div className="relative bg-secondary text-secondary-foreground rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
                    <div className="p-8">
                        <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                            <span className="text-2xl">🔄</span>
                        </div>
                        <h3 className="text-2xl font-bold mb-4">Refresh Data</h3>
                        <p className="text-secondary-foreground/80 mb-6">Reload subscription data from server</p>
                        <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                            Refresh
                        </div>
                    </div>
                </div>
            </div>

            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5">
                <div className="relative bg-primary text-primary-foreground rounded-2xl sm:rounded-3xl">
                    <div className="p-8">
                        <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                            <span className="text-2xl">📊</span>
                        </div>
                        <h3 className="text-2xl font-bold mb-4">Usage Analytics</h3>
                        <p className="text-primary-foreground/80 mb-6">View detailed usage and billing analytics</p>
                        <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                            Coming Soon
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

interface SubscriptionCardProps {
    subscription: SubscriptionResponse;
    onCancel: (id: string) => Promise<void>;
    onView: (id: string) => void;
}

export function SubscriptionCard({ subscription, onCancel, onView }: SubscriptionCardProps) {
    return (
        <div
            className="flex items-center justify-between p-6 bg-muted/20 border border-border/50 rounded-2xl hover:bg-muted/40 transition-all cursor-pointer"
            onClick={() => onView(subscription.id)}
        >
            <div className="flex items-center gap-6 flex-1">
                <div className={`h-12 w-12 rounded-2xl flex items-center justify-center text-2xl ${subscription.access_context === 'internal'
                        ? 'bg-secondary text-secondary-foreground'
                        : subscription.access_context === 'external'
                            ? 'bg-primary text-primary-foreground'
                            : 'bg-info text-info-foreground'
                    }`}>
                    {subscription.access_context === 'internal' ? '🖥️' :
                        subscription.access_context === 'external' ? '🔧' : '🔄'}
                </div>

                <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                        <h3 className="text-lg font-bold text-foreground">
                            {subscription.plan_name}
                        </h3>
                        <span className={`px-3 py-1 text-xs rounded-full font-semibold ${subscription.status === 'active'
                                ? 'bg-success/10 text-success border border-success/20'
                                : subscription.status === 'expired'
                                    ? 'bg-warning/10 text-warning border border-warning/20'
                                    : 'bg-destructive/10 text-destructive border border-destructive/20'
                            }`}>
                            {subscription.status.toUpperCase()}
                        </span>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span className="font-semibold text-foreground/80">User: {subscription.user_id}</span>
                        <span>•</span>
                        <span>{subscription.access_context}</span>
                        {subscription.api_key_name && (
                            <>
                                <span>•</span>
                                <span>API Key: {subscription.api_key_name}</span>
                            </>
                        )}
                        {subscription.expires_at && (
                            <>
                                <span>•</span>
                                <span>Expires: {new Date(subscription.expires_at).toLocaleDateString()}</span>
                            </>
                        )}
                    </div>
                </div>
            </div>

            <div className="flex items-center gap-4">
                {subscription.status === 'active' && (
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            void onCancel(subscription.id);
                        }}
                        className="px-4 py-2 rounded-xl font-semibold bg-destructive/10 text-destructive hover:bg-destructive/20 transition-colors border border-destructive/20"
                    >
                        Cancel
                    </button>
                )}

                <button
                    className="px-4 py-2 rounded-xl font-semibold bg-secondary text-secondary-foreground hover:opacity-90 transition-opacity border border-secondary/20 shadow-sm"
                >
                    View Details
                </button>
            </div>
        </div>
    );
}

interface SubscriptionFiltersProps {
    searchTerm: string;
    setSearchTerm: (term: string) => void;
    filterStatus: FilterStatus;
    setFilterStatus: (status: FilterStatus) => void;
    filterContext: FilterContext;
    setFilterContext: (context: FilterContext) => void;
    onApply: () => void;
}

export function SubscriptionFilters({
    searchTerm,
    setSearchTerm,
    filterStatus,
    setFilterStatus,
    filterContext,
    setFilterContext,
    onApply
}: SubscriptionFiltersProps) {
    return (
        <div className="bg-card/80 rounded-3xl p-6 border border-border/50 mb-8 shadow-sm">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div>
                    <label className="block text-sm font-semibold text-muted-foreground mb-2">
                        Search
                    </label>
                    <input
                        type="text"
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                        placeholder="Search by plan, user..."
                        className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                    />
                </div>

                <div>
                    <label className="block text-sm font-semibold text-muted-foreground mb-2">
                        Status
                    </label>
                    <select
                        value={filterStatus}
                        onChange={(e) => setFilterStatus(e.target.value as FilterStatus)}
                        className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                    >
                        <option value="all">All Status</option>
                        <option value="active">Active</option>
                        <option value="expired">Expired</option>
                        <option value="cancelled">Cancelled</option>
                    </select>
                </div>

                <div>
                    <label className="block text-sm font-semibold text-muted-foreground mb-2">
                        Access Context
                    </label>
                    <select
                        value={filterContext}
                        onChange={(e) => setFilterContext(e.target.value as FilterContext)}
                        className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                    >
                        <option value="all">All Context</option>
                        <option value="internal">Internal</option>
                        <option value="external">External</option>
                        <option value="both">Both</option>
                    </select>
                </div>

                <div className="flex items-end">
                    <button
                        onClick={onApply}
                        className="w-full px-6 py-3 bg-primary text-primary-foreground rounded-xl font-semibold hover:opacity-90 transition-opacity border border-border/50 shadow-sm"
                    >
                        Apply Filters
                    </button>
                </div>
            </div>
        </div>
    );
}

export function SubscriptionLoading() {
  return (
    <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
      <div className="text-center mb-12">
        <div className="h-16 bg-muted rounded-2xl w-96 mx-auto mb-6" />
        <div className="h-6 bg-muted/60 rounded-full w-64 mx-auto" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={`skeleton-action-${i}`} className="bg-muted rounded-3xl h-64" />
        ))}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={`skeleton-stat-${i}`} className="bg-muted rounded-3xl h-32" />
        ))}
      </div>
    </div>
  );
}

interface SubscriptionEmptyStateProps {
  hasSubscriptions: boolean;
}

export function SubscriptionEmptyState({ hasSubscriptions }: SubscriptionEmptyStateProps) {
  return (
    <div className="text-center py-12">
      <div className="h-20 w-20 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
        <span className="text-4xl text-primary">📋</span>
      </div>
      <h3 className="text-xl font-semibold text-foreground mb-2">
        No subscriptions found
      </h3>
      <p className="text-muted-foreground">
        {hasSubscriptions
          ? 'Try adjusting your filters or search terms'
          : 'Start by creating your first subscription'
        }
      </p>
    </div>
  );
}
