import { Button } from '@/components/ui/button';
import type { UseWalletAccessReturn } from '@/hooks/use-wallet-access';
import type { useWalletData } from '@/hooks/use-wallet-detail';
import { cn } from '@/lib/utils';
import { ArrowLeft, RefreshCw } from 'lucide-react';
import Link from 'next/link';

interface WalletDetailHeaderProps {
    walletData: ReturnType<typeof useWalletData>;
    accessData: UseWalletAccessReturn;
}

export function WalletDetailHeader({
    walletData,
    accessData
}: WalletDetailHeaderProps) {
    return (
        <div className="flex items-center gap-4">
            <Link
                href="/wallet-management"
                className="p-2 rounded-xl bg-card border border-border/40 hover:bg-muted/30 transition-colors"
            >
                <ArrowLeft className="h-5 w-5" />
            </Link>
            <div className="flex-1">
                <h1 className="text-2xl font-bold flex items-center gap-2">
                    <span>👛</span>
                    Wallet Details
                </h1>
                <p className="text-sm text-muted-foreground">
                    Manage wallet access and plans
                </p>
            </div>
            <Button
                variant="outline"
                onClick={() => { void walletData.loadWallet(); void accessData.refresh(); }}
                disabled={walletData.isRefreshing === true}
                className="gap-2"
            >
                <RefreshCw className={cn('h-4 w-4', walletData.isRefreshing === true && 'animate-spin')} />
                Refresh
            </Button>
        </div>
    );
}
