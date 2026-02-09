'use client';

import { cn } from '@/lib/utils';
import type { WalletData } from './types';
import { WalletTableRow } from './wallet-table-row';

interface WalletTableProps {
    wallets: WalletData[];
    selectedAddresses: Set<string>;
    onSelectWallet: (address: string, selected: boolean) => void;
    onView: (wallet: WalletData) => void;
    onManage: (wallet: WalletData) => void;
    onDisable: (wallet: WalletData) => void;
    onEnable: (wallet: WalletData) => void;
    onEdit: (wallet: WalletData) => void;
    className?: string;
}

export function WalletTable({
    wallets,
    selectedAddresses,
    onSelectWallet,
    onView,
    onManage,
    onDisable,
    onEnable,
    onEdit,
    className,
}: WalletTableProps) {
    return (
        <div className={cn("w-full overflow-auto rounded-xl border border-border/60 bg-card", className)}>
            <table className="w-full text-sm text-left border-collapse">
                <thead>
                    <tr className="border-b border-border/60 bg-muted/30">
                        <th className="p-4 w-10">
                            {/* Header checkbox could go here for bulk select */}
                        </th>
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap">Wallet & Label</th>
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap">Plan</th>
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap">Status</th>
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap text-right">Actions</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-border/40">
                    {wallets.map((wallet) => (
                        <WalletTableRow
                            key={wallet.walletAddress}
                            wallet={wallet}
                            isSelected={selectedAddresses.has(wallet.walletAddress)}
                            onSelectWallet={onSelectWallet}
                            onView={onView}
                            onManage={onManage}
                            onDisable={onDisable}
                            onEnable={onEnable}
                            onEdit={onEdit}
                        />
                    ))}
                </tbody>
            </table>
        </div>
    );
}
