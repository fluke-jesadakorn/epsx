
import { WalletSection } from '@/components/wallet/wallet-section';
import { fetchWalletsAction } from '../actions';

export default async function WalletsPage() {
    const response = await fetchWalletsAction({
        platform: 'all',
        status: 'all',
        sortBy: 'created_at',
        sortOrder: 'desc',
        search: ''
    }, 1, 9).catch(() => undefined) as any;

    const initialWalletsData = response?.success ? { wallets: response.wallets, pagination: response.pagination } : undefined;

    return <WalletSection initialData={initialWalletsData} />;
}
