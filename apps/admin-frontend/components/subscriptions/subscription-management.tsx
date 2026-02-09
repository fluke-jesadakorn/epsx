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
        <div className="text-center mb-12">
          <div className="relative inline-block mb-6">
            <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
              📋 Subscription Management
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary/20 rounded-full animate-ping" />
          </div>
          <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
            Manage user subscriptions, track usage, and handle billing for all plans
          </p>
        </div>

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

        <div className="bg-card/90 backdrop-blur-sm border border-border/30 overflow-hidden rounded-3xl shadow-sm">
          <div className="p-8">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                Subscriptions
              </h2>
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