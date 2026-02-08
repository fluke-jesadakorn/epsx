
import { WalletSection } from '@/components/wallet/wallet-section';
import { fetchWalletsAction } from '../actions';

export default async function WalletsPage() {
    const initialWalletsData = await fetchWalletsAction({
        platform: 'all',
        status: 'all',
        sortBy: 'created_at',
        sortOrder: 'desc',
        search: ''
    }).catch(() => ({
        wallets: [],
        pagination: { page: 1, limit: 20, total: 0, total_pages: 1, has_next_page: false, has_previous_page: false }
    }));

    return <WalletSection initialData={initialWalletsData} />;
}
