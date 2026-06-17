
import { WalletSection } from '@/components/wallet/wallet-section';
import { fetchWalletsAction } from '../actions';

export default async function WalletsPage() {
    const response = await fetchWalletsAction({
        platform: 'all',
        status: 'all',
        sortBy: 'created_at',
        sortOrder: 'desc',
        search: ''
    }, 1, 10).catch(() => undefined);

    const initialWalletsData = response?.success === true
        ? { wallets: response.wallets, pagination: response.pagination }
        : undefined;

    return <WalletSection initialData={initialWalletsData} />;
}
