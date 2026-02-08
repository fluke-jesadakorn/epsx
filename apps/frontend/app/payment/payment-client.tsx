'use client';

/**
 * PaymentClient Component
 * 
 * Client-side wrapper for the payment flow.
 * Required because next/dynamic with ssr: false must be in a client component.
 */

import dynamic from 'next/dynamic';
import { Suspense } from 'react';

// Dynamically import the payment component to avoid SSR issues with wallet
const UnifiedPaymentFlow = dynamic(
    () => import('@/components/features/payment/unified-payment-flow'),
    {
        ssr: false,
        loading: () => (
            <div className="flex justify-center items-center min-h-[400px]">
                <div className="text-center">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600 mx-auto mb-4" />
                    <p className="text-gray-600 dark:text-gray-400">Loading payment options...</p>
                </div>
            </div>
        ),
    }
);

type PaymentType = 'plan' | 'access-plan' | 'permission';

interface PaymentClientProps {
    paymentType: PaymentType;
    preselectedId?: string;
    title?: string;
    description?: string;
    initialPlans?: any[];
}

export function PaymentClient({
    paymentType,
    preselectedId,
    title,
    description,
    initialPlans = [],
}: PaymentClientProps) {
    return (
        <Suspense
            fallback={
                <div className="flex justify-center items-center min-h-[400px]">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600" />
                </div>
            }
        >
            <UnifiedPaymentFlow
                paymentType={paymentType}
                preselectedId={preselectedId}
                title={title}
                description={description}
                initialPlans={initialPlans}
            />
        </Suspense>
    );
}

export default PaymentClient;
