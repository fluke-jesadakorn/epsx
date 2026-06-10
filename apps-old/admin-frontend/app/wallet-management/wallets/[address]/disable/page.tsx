'use client';

import { DisableWalletPage } from '@/components/wallet/disable-wallet-page';
import { useParams } from 'next/navigation';

export default function DisableWalletRoute() {
    const params = useParams();
    const address = decodeURIComponent(params['address'] as string);

    return <DisableWalletPage walletAddress={address} />;
}
