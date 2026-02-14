'use client';

import { Edit, Eye, MoreHorizontal, Star } from 'lucide-react';

import type { WalletData } from './types';
import { getPlanDisplay } from './wallet-card-sections';
import { WalletLabelBadge } from './wallet-label-badge';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';

interface WalletTableRowProps {
    wallet: WalletData;
    isSelected: boolean;
    onSelectWallet: (address: string, selected: boolean) => void;
    onView: (wallet: WalletData) => void;
    onManage: (wallet: WalletData) => void;
    onDisable: (wallet: WalletData) => void;
    onEnable: (wallet: WalletData) => void;
    onEdit: (wallet: WalletData) => void;
}

export function WalletTableRow({
    wallet,
    isSelected,
    onSelectWallet,
    onView,
    onManage,
    onDisable,
    onEnable,
    onEdit,
}: WalletTableRowProps) {
    const { name: plan } = getPlanDisplay(wallet);
    const isDisabled = wallet.status === 'disabled';

    return (
        <tr
            className={cn(
                "group hover:bg-muted/30 transition-colors",
                isSelected && "bg-primary/5",
                isDisabled && "opacity-70"
            )}
        >
            <td className="p-4">
                <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={(e) => onSelectWallet(wallet.walletAddress, e.target.checked)}
                    className="h-4 w-4 rounded border-border text-primary focus:ring-primary cursor-pointer"
                />
            </td>
            <td className="p-4">
                <div className="flex flex-col min-w-0">
                    <div className="flex items-center gap-2">
                        <span className="font-mono text-xs font-medium text-foreground truncate max-w-[120px] md:max-w-none">
                            {wallet.walletAddress}
                        </span>
                        {wallet.label ? (
                            <WalletLabelBadge label={wallet.label} size="sm" className="hidden sm:inline-flex" />
                        ) : null}
                    </div>
                    {wallet.label ? (
                        <span className="text-[10px] text-muted-foreground mt-0.5 sm:hidden truncate">
                            {wallet.label}
                        </span>
                    ) : null}
                </div>
            </td>
            <td className="p-4">
                <div className="flex items-center gap-2">
                    <Badge variant="outline" className="bg-primary/5 text-primary border-primary/10 text-[10px] py-0 px-2 uppercase font-bold tracking-wider">
                        {plan}
                    </Badge>
                </div>
            </td>
            <td className="p-4">
                <div className="flex items-center gap-2">
                    <div className={cn(
                        "h-1.5 w-1.5 rounded-full",
                        wallet.status === 'active' ? "bg-success" : "bg-warning"
                    )} />
                    <span className={cn(
                        "capitalize text-xs font-medium",
                        wallet.status === 'active' ? "text-success" : "text-warning"
                    )}>
                        {wallet.status}
                    </span>
                </div>
            </td>

            <td className="p-4 text-right">
                <div className="flex items-center justify-end gap-1">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onView(wallet)}
                        className="h-8 w-8 text-muted-foreground hover:text-primary transition-colors"
                        title="View Details"
                    >
                        <Eye className="h-4 w-4" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onEdit(wallet)}
                        className="h-8 w-8 text-muted-foreground hover:text-primary transition-colors"
                        title="Edit Metadata"
                    >
                        <Edit className="h-4 w-4" />
                    </Button>
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button
                                variant="ghost"
                                size="icon"
                                className="h-8 w-8 text-muted-foreground hover:text-primary transition-colors"
                            >
                                <MoreHorizontal className="h-4 w-4" />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end" className="w-48">
                            <DropdownMenuItem onClick={() => onManage(wallet)} className="gap-2">
                                <Star className="h-4 w-4" />
                                Manage Access
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            {isDisabled ? (
                                <DropdownMenuItem onClick={() => onEnable(wallet)} className="text-success gap-2">
                                    <span>🔓</span>
                                    Re-enable
                                </DropdownMenuItem>
                            ) : (
                                <DropdownMenuItem onClick={() => onDisable(wallet)} className="text-warning gap-2">
                                    <span>⚠️</span>
                                    Disable Wallet
                                </DropdownMenuItem>
                            )}
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            </td>
        </tr>
    );
}
