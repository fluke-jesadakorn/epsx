'use client';

import { Edit, Eye, MoreHorizontal, Shield, Star } from 'lucide-react';

import { WalletLabelBadge } from './WalletLabelBadge';
import type { WalletData } from './types';

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
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap">Access</th>
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap">Status</th>
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap text-center">Perms</th>
                        <th className="p-4 font-semibold text-muted-foreground whitespace-nowrap text-right">Actions</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-border/40">
                    {wallets.map((wallet) => {
                        const isSelected = selectedAddresses.has(wallet.walletAddress);
                        const activePermissions = wallet.permissions.filter(p => p.isActive).length;
                        const plan = wallet.subscriptions[0]?.planName || 'Free';
                        const group = wallet.groups?.[0]?.groupName || 'User';
                        const isDisabled = wallet.status === 'disabled';

                        return (
                            <tr
                                key={wallet.walletAddress}
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
                                            {wallet.label && (
                                                <WalletLabelBadge label={wallet.label} size="sm" className="hidden sm:inline-flex" />
                                            )}
                                        </div>
                                        {wallet.label && (
                                            <span className="text-[10px] text-muted-foreground mt-0.5 sm:hidden truncate">
                                                {wallet.label}
                                            </span>
                                        )}
                                    </div>
                                </td>
                                <td className="p-4">
                                    <div className="flex items-center gap-2">
                                        <Badge variant="outline" className="bg-primary/5 text-primary border-primary/10 text-[10px] py-0 px-2 uppercase font-bold tracking-wider">
                                            {plan}
                                        </Badge>
                                        <div className="flex items-center gap-1 text-muted-foreground text-xs whitespace-nowrap">
                                            <Shield className="h-3 w-3 fill-current opacity-70" />
                                            {group}
                                        </div>
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
                                <td className="p-4 text-center">
                                    <Badge variant="secondary" className="font-mono text-[10px] px-1.5 h-5 min-w-5 justify-center">
                                        {activePermissions}
                                    </Badge>
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
                    })}
                </tbody>
            </table>
        </div>
    );
}
