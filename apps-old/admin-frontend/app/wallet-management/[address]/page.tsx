'use client';

import { useParams, useRouter } from 'next/navigation';

import { WalletDetailView } from '@/components/wallet/wallet-detail-view';
import { useWalletAccess } from '@/hooks/use-wallet-access';
import { useSubscriptionData, useWalletData } from '@/hooks/use-wallet-detail';

export default function WalletDetailPage() {
    const router = useRouter();
    const params = useParams();
    const walletAddress = decodeURIComponent(params['address'] as string);

    const walletData = useWalletData({ walletAddress, router });
    const subscriptionData = useSubscriptionData(walletAddress);
    const accessData = useWalletAccess(walletAddress);

    return (
        <WalletDetailView
            walletAddress={walletAddress}
            authLoading={false}
            walletData={walletData}
            subscriptionData={subscriptionData}
            access={accessData}
        />
    );
}
