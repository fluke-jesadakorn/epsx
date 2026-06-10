'use client';

import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { formatDistanceToNow } from 'date-fns';
import { ExternalLink, Hash, RefreshCcw, Wifi, WifiOff } from 'lucide-react';

interface ActivityStreamWallet {
    wallet_address: string;
    is_active: boolean;
    last_auth_at?: string;
    connection_info?: { is_new: boolean };
}

interface RecentWalletsData {
    recent_wallets: ActivityStreamWallet[];
}

interface WalletRowProps {
    wallet: ActivityStreamWallet;
}

function WalletRow({ wallet }: WalletRowProps): React.JSX.Element {
    const isNew = wallet.connection_info?.is_new === true;
    const dateStr = wallet.last_auth_at !== undefined ? new Date(wallet.last_auth_at) : null;
    const addr = `${wallet.wallet_address.slice(0, 6)}...${wallet.wallet_address.slice(-4)}`;
    return (
        <div className="group relative z-10 flex items-start gap-3 p-3 rounded-lg border border-transparent hover:border-border/30 hover:bg-muted/30 transition-all">
            <div className="mt-1">
                {wallet.is_active === true ? (
                    <Wifi className="w-4 h-4 text-success" />
                ) : (
                    <WifiOff className="w-4 h-4 text-muted-foreground/50" />
                )}
            </div>
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                    <Hash className="w-3 h-3 text-muted-foreground" />
                    <span className={`font-bold ${isNew ? 'text-cyan-400' : 'text-foreground'}`}>{addr}</span>
                    {isNew && (
                        <span className="px-1.5 py-0.5 rounded text-[9px] bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 uppercase tracking-widest leading-none">
                            New Auth
                        </span>
                    )}
                    {wallet.is_active === true && (
                        <span className="px-1.5 py-0.5 rounded text-[9px] bg-success/20 text-success border border-success/30 uppercase tracking-widest leading-none">
                            Active
                        </span>
                    )}
                </div>
                <div className="text-[11px] text-muted-foreground opacity-80 flex items-center gap-2">
                    {dateStr !== null ? (
                        <>
                            <span>{formatDistanceToNow(dateStr, { addSuffix: true })}</span>
                            <span className="text-border/40">•</span>
                            <span className="hidden sm:inline">{dateStr.toISOString()}</span>
                        </>
                    ) : (
                        'TIME_UNKNOWN'
                    )}
                </div>
            </div>
            <a
                href={`https://bscscan.com/address/${wallet.wallet_address}`}
                target="_blank"
                rel="noreferrer"
                className="opacity-0 group-hover:opacity-100 p-2 text-muted-foreground hover:text-cyan-400 hover:bg-cyan-400/10 rounded transition-all mt-0.5"
            >
                <ExternalLink className="w-4 h-4" />
            </a>
        </div>
    );
}

async function fetchRecentWallets(): Promise<RecentWalletsData> {
    const client = createAdminApiClient();
    const response = await client.get<RecentWalletsData>('/api/admin/web3/recent-wallets?limit=15&days=30');
    if (response.success === true && response.data !== null) {
        return response.data;
    }
    return { recent_wallets: [] };
}

interface DashboardActivityStreamProps {
    initialData?: RecentWalletsData;
}

export function DashboardActivityStream({ initialData }: DashboardActivityStreamProps) {
    const { data, isLoading, isFetching, refetch } = useQuery<RecentWalletsData>({
        queryKey: ['recent-wallets-stream'],
        queryFn: fetchRecentWallets,
        initialData,
        staleTime: initialData !== undefined ? 30_000 : 0,
        gcTime: 300_000,
        refetchInterval: 120_000,
        refetchOnWindowFocus: false,
    });

    const wallets = data?.recent_wallets ?? [];

    return (
        <div className="rounded-2xl border border-border/20 bg-card shadow-2xl overflow-hidden flex flex-col h-full lg:max-h-[800px]">
            <div className="flex items-center justify-between p-4 border-b border-border/20 bg-muted/20">
                <div className="flex items-center gap-3">
                    <div className="relative flex h-3 w-3">
                        <span className="animate-[ping_2s_ease-in-out_infinite] absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75" />
                        <span className="relative inline-flex rounded-full h-3 w-3 bg-cyan-500" />
                    </div>
                    <h2 className="text-xs font-black uppercase tracking-[0.2em] text-cyan-400 shadow-cyan-500/50 drop-shadow-sm font-mono">
                        Global Event Stream
                    </h2>
                </div>
                <button
                    onClick={() => { void refetch(); }}
                    className={`p-1.5 rounded-md hover:bg-muted transition-all ${isFetching ? 'animate-spin' : ''}`}
                >
                    <RefreshCcw className="w-4 h-4 text-muted-foreground" />
                </button>
            </div>
            <div className="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar bg-background/50 font-mono text-sm relative">
                <div className="absolute inset-0 pointer-events-none bg-[linear-gradient(transparent_50%,rgba(0,0,0,0.1)_50%)] bg-[length:100%_4px]" />
                {isLoading ? (
                    <div className="flex items-center justify-center h-48 opacity-50">
                        <span className="animate-pulse">STREAM.CONNECTING...</span>
                    </div>
                ) : wallets.length === 0 ? (
                    <div className="flex items-center justify-center h-48 text-muted-foreground opacity-50">
                        EMPTY_BUFFER
                    </div>
                ) : (
                    wallets.map((wallet) => <WalletRow key={wallet.wallet_address} wallet={wallet} />)
                )}
            </div>
            <div className="bg-background/80 p-2 border-t border-border/20 text-[10px] text-muted-foreground font-mono uppercase tracking-widest text-center">
                END OF STREAM / {wallets.length} NODES LOGGED
            </div>
        </div>
    );
}
