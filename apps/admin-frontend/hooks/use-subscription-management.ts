'use client';

import { toast } from '@/hooks/use-toast';
import { createPlansClient, isApiSuccess, type SubscriptionResponse } from '@/shared/api/plans';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

export type FilterStatus = 'all' | 'active' | 'expired' | 'cancelled';
export type FilterContext = 'all' | 'internal' | 'external' | 'both';

export function useSubscriptionManagement() {
    const router = useRouter();
    const [subscriptions, setSubscriptions] = useState<SubscriptionResponse[]>([]);
    const [loading, setLoading] = useState(true);
    const [filterStatus, setFilterStatus] = useState<FilterStatus>('all');
    const [filterContext, setFilterContext] = useState<FilterContext>('all');
    const [searchTerm, setSearchTerm] = useState('');

    const loadSubscriptions = useCallback(async () => {
        const adminClient = createPlansClient(createAdminApiClient());
        try {
            setLoading(true);
            const response = await adminClient.getSubscriptions({
                limit: 100,
                status: filterStatus === 'all' ? undefined : filterStatus,
                access_context: filterContext === 'all' ? undefined : filterContext,
            });

            if (isApiSuccess(response)) {
                const data = response.data as Record<string, SubscriptionResponse[] | undefined>;
                setSubscriptions(data?.subscriptions ?? (Array.isArray(data) ? data : []));
            } else {
                toast({
                    title: "Error",
                    description: response.error ?? "Failed to load subscriptions",
                    variant: "destructive"
                });
            }
        } catch (_error) {
            toast({
                title: "Error",
                description: "Failed to load subscriptions",
                variant: "destructive"
            });
        } finally {
            setLoading(false);
        }
    }, [filterStatus, filterContext]);

    useEffect(() => {
        void loadSubscriptions();
    }, [loadSubscriptions]);

    const handleCancelSubscription = async (subscriptionId: string) => {
        // TODO: Replace with proper modal confirmation
        // eslint-disable-next-line no-alert
        if (!confirm('Are you sure you want to cancel this subscription?')) {
            return;
        }

        const adminClient = createPlansClient(createAdminApiClient());
        try {
            const response = await adminClient.cancelSubscription(subscriptionId);

            if (isApiSuccess(response)) {
                void loadSubscriptions();
                toast({
                    title: "Success",
                    description: "Subscription cancelled successfully",
                });
            } else {
                toast({
                    title: "Error",
                    description: response.error ?? "Failed to cancel subscription",
                    variant: "destructive"
                });
            }
        } catch {
            toast({
                title: "Error",
                description: "Failed to cancel subscription",
                variant: "destructive"
            });
        }
    };

    const filteredSubscriptions = subscriptions.filter(sub => {
        const statusMatch = filterStatus === 'all' || sub.status === filterStatus;
        const contextMatch = filterContext === 'all' || sub.access_context === filterContext;
        const searchMatch = searchTerm === '' ||
            sub.plan_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            sub.user_id.toLowerCase().includes(searchTerm.toLowerCase()) ||
            (sub.api_key_name?.toLowerCase().includes(searchTerm.toLowerCase()) ?? false);

        return statusMatch && contextMatch && searchMatch;
    });

    const activeSubscriptions = subscriptions.filter(s => s.status === 'active');
    const expiredSubscriptions = subscriptions.filter(s => s.status === 'expired');
    const cancelledSubscriptions = subscriptions.filter(s => s.status === 'cancelled');
    const totalRevenue = subscriptions.reduce((sum, sub) => {
        // Estimate monthly revenue - this could be enhanced with actual billing data
        return sum + (sub.status === 'active' ? 99 : 0); // placeholder calculation
    }, 0);

    return {
        subscriptions,
        filteredSubscriptions,
        loading,
        filterStatus,
        setFilterStatus,
        filterContext,
        setFilterContext,
        searchTerm,
        setSearchTerm,
        activeSubscriptions,
        expiredSubscriptions,
        cancelledSubscriptions,
        totalRevenue,
        loadSubscriptions,
        handleCancelSubscription,
        router
    };
}
