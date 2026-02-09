'use client';

import { useParams, useRouter } from 'next/navigation';
import { useEffect } from 'react';

import { WalletDetailView } from '@/components/wallet/wallet-detail-view';
import { useWalletAccess } from '@/hooks/use-wallet-access';
import { useSubscriptionData, useWalletData } from '@/hooks/use-wallet-detail';
import { useSharedAuth } from '@/shared/components/auth/Provider';

export default function WalletDetailPage() {
    const router = useRouter();
    const params = useParams();
    const walletAddress = decodeURIComponent(params['address'] as string);

    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // Extract custom hooks
    const walletData = useWalletData({ walletAddress, router });
    const subscriptionData = useSubscriptionData(walletAddress);
    const accessData = useWalletAccess(walletAddress);

    // Load wallet on auth
    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            void walletData.loadWallet();
        }
    }, [isAuthenticated, authLoading, walletData]);

    return (
        <WalletDetailView
            walletAddress={walletAddress}
            authLoading={authLoading}
            walletData={walletData}
            subscriptionData={subscriptionData}
            access={accessData}
        />
    );
}
