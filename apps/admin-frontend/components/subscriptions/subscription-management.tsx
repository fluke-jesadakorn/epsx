'use client';

import { PageHeader } from '@/components/shared';
import { useSubscriptionManagement } from '@/hooks/use-subscription-management';
import {
  SubscriptionActions,
  SubscriptionCard,
  SubscriptionEmptyState,
  SubscriptionFilters,
  SubscriptionLoading,
  SubscriptionStats
} from './subscription-components';

interface SubscriptionManagementProps {
  currentUser?: unknown;
}

/**
 * Subscription Management Component
 */
export function SubscriptionManagement({ currentUser: _currentUser }: SubscriptionManagementProps) {
  const {
    filteredSubscriptions,
    subscriptions,
    loading,
    filterStatus,
    setFilterStatus,
    filterContext,
    setFilterContext,
    searchTerm,
    setSearchTerm,
    activeSubscriptions,
    expiredSubscriptions,
    totalRevenue,
    loadSubscriptions,
    handleCancelSubscription,
    router
  } = useSubscriptionManagement();

  if (loading) {
    return <SubscriptionLoading />;
  }

  return (
    <>
      <PageHeader
        title="Subscriptions"
        subtitle="Manage user subscriptions and billing across the platform"
        icon="CreditCard"
        gradient="purple"
      />
      <div className="max-w-7xl mx-auto space-y-8">
        <SubscriptionActions
          onRefresh={() => { void loadSubscriptions(); }}
          isLoading={loading}
        />

        <SubscriptionStats
          total={subscriptions.length}
          active={activeSubscriptions.length}
          expired={expiredSubscriptions.length}
          revenue={totalRevenue}
        />

        <SubscriptionFilters
          searchTerm={searchTerm}
          setSearchTerm={setSearchTerm}
          filterStatus={filterStatus}
          setFilterStatus={setFilterStatus}
          filterContext={filterContext}
          setFilterContext={setFilterContext}
          onApply={() => { void loadSubscriptions(); }}
        />

        <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
          <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" />
          <div className="flex items-center justify-between px-4 py-3 border-b border-border/20">
            <h2 className="text-xs font-bold text-[#7645d9] uppercase tracking-[0.2em]">SUBSCRIPTIONS</h2>
          </div>
          <div className="p-8">
            <div className="flex items-center justify-between mb-6">
              <div className="text-sm text-muted-foreground">
                {filteredSubscriptions.length} subscriptions
              </div>
            </div>

            <div className="space-y-4">
              {filteredSubscriptions.map(subscription => (
                <SubscriptionCard
                  key={subscription.id}
                  subscription={subscription}
                  onCancel={handleCancelSubscription}
                  onView={(id) => router.push(`/subscriptions/${id}`)}
                />
              ))}
            </div>

            {filteredSubscriptions.length === 0 && (
              <SubscriptionEmptyState hasSubscriptions={subscriptions.length > 0} />
            )}
          </div>
        </div>
      </div>
    </>
  );
}