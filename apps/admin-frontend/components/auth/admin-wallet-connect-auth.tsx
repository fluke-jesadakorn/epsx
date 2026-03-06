'use client';

import { AlertCircle, ChevronDown, Loader2, Wallet, Check, Copy, ExternalLink, LogOut, Shield, Key } from 'lucide-react';
import { useState } from 'react';
import { useDisconnect, useChainId, useBalance } from 'wagmi';
import { AuthModal, useSharedAuth } from '@/shared/components/auth';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';
import { getExplorerAddressLink } from '@/shared/config/constants';
import { formatAddress } from '@/shared/auth/utils';

interface AdminWalletConnectAuthProps {
  className?: string;
}

function LoadingButton({ message, className = '' }: {
  message: string;
  className?: string;
}) {
  return (
    <button
      disabled
      className={cn('flex items-center gap-2 text-primary-foreground', className)}
    >
      <Loader2 className="h-4 w-4 animate-pulse" />
      {message}
    </button>
  );
}

function ConnectedDropdown({ displayAddress, copied, onCopy, onExplorer, onDisconnect, role, tierLevel, permCount, chainId, balance }: {
  displayAddress: string;
  copied: boolean;
  onCopy: () => void;
  onExplorer: () => void;
  onDisconnect: () => void;
  className?: string;
  role?: string;
  tierLevel?: string;
  permCount?: number;
  chainId?: number;
  balance?: string;
}) {
  const chainName = chainId === 56 ? 'BSC Mainnet' : chainId === 97 ? 'BSC Testnet' : chainId !== undefined ? `Chain ${chainId}` : null;
  const isLive = chainId === 56;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button className="flex items-center gap-1.5 sm:gap-2 text-primary-foreground">
          <Wallet className="h-3.5 w-3.5 sm:h-4 sm:w-4 flex-shrink-0" />
          <span className="hidden lg:inline">{formatAddress(displayAddress)}</span>
          <ChevronDown className="h-3 w-3 flex-shrink-0" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="w-64 bg-white dark:bg-[#1a1d2e] border border-gray-200 dark:border-white/10 p-0 overflow-hidden"
        style={{ zIndex: 99999 }}
      >
        {/* Header — orange accent bar */}
        <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#f97316]" />

        {/* Address */}
        <div className="px-3 py-2.5 border-b border-gray-200 dark:border-white/10">
          <div className="flex items-center gap-1.5 mb-1">
            <Wallet className="h-3 w-3 text-[#ffb237] flex-shrink-0" />
            <p className="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">Wallet</p>
          </div>
          <p className="text-xs text-gray-800 dark:text-gray-100 font-mono break-all leading-relaxed">{displayAddress}</p>
        </div>

        {/* Meta info */}
        <div className="px-3 py-2.5 border-b border-gray-200 dark:border-white/10 grid grid-cols-2 gap-x-4 gap-y-2.5">
          {role !== undefined && role !== '' && (
            <div>
              <p className="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-0.5">Role</p>
              <div className="flex items-center gap-1">
                <Shield className="h-3 w-3 text-purple-500 dark:text-purple-400 flex-shrink-0" />
                <span className="text-xs font-medium text-purple-600 dark:text-purple-400 capitalize">{role.replace('_', ' ')}</span>
              </div>
            </div>
          )}
          {tierLevel !== undefined && tierLevel !== '' && (
            <div>
              <p className="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-0.5">Tier</p>
              <span className="text-xs font-medium text-cyan-600 dark:text-cyan-400">{tierLevel}</span>
            </div>
          )}
          {permCount !== undefined && permCount > 0 && (
            <div>
              <p className="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-0.5">Permissions</p>
              <div className="flex items-center gap-1">
                <Key className="h-3 w-3 text-gray-400 dark:text-gray-500 flex-shrink-0" />
                <span className="text-xs text-gray-700 dark:text-gray-300">{permCount}</span>
              </div>
            </div>
          )}
          {balance !== undefined && balance !== '' && (
            <div>
              <p className="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-0.5">Balance</p>
              <span className="text-xs text-gray-700 dark:text-gray-300">{balance} BNB</span>
            </div>
          )}
        </div>

        {/* Network */}
        {chainName !== null && (
          <div className="px-3 py-2 border-b border-gray-200 dark:border-white/10">
            <div className="flex items-center gap-1.5">
              <span className={cn('h-1.5 w-1.5 rounded-full flex-shrink-0', isLive ? 'bg-green-500' : 'bg-yellow-400')} />
              <span className="text-xs text-gray-600 dark:text-gray-400">{chainName}</span>
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="p-1">
          <DropdownMenuItem
            onClick={onCopy}
            className="flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/10"
          >
            {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
            {copied ? 'Copied!' : 'Copy Address'}
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={onExplorer}
            className="flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/10"
          >
            <ExternalLink className="h-4 w-4" />
            View on Explorer
          </DropdownMenuItem>
          <DropdownMenuSeparator className="my-1 bg-gray-200 dark:bg-white/10" />
          <DropdownMenuItem
            onClick={onDisconnect}
            className="flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-500/20"
          >
            <LogOut className="h-4 w-4" />
            Disconnect
          </DropdownMenuItem>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function AdminWalletConnectAuth({ className = '' }: AdminWalletConnectAuthProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  const { disconnect } = useDisconnect();
  const { user, isLoading, error, logout } = useSharedAuth();
  const chainId = useChainId();
  const { data: balanceData } = useBalance({
    address: user?.wallet_address as `0x${string}` | undefined,
  });

  const isAuthenticated = Boolean(user);

  if (isLoading) {
    return <LoadingButton message="Loading..." className={className} />;
  }

  const displayAddress = user?.wallet_address;

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Silently fail
    }
  };

  const openBSCScan = () => {
    if (displayAddress !== undefined && displayAddress !== '') {
      window.open(getExplorerAddressLink(displayAddress), '_blank');
    }
  };

  const handleDisconnect = async () => {
    try {
      await logout();
      disconnect();
    } catch {
      // Silently fail
    } finally {
      window.location.href = '/?logout=1';
    }
  };

  if (isAuthenticated && displayAddress !== undefined && displayAddress !== '') {
    const bnbBalance = balanceData !== undefined
      ? parseFloat(balanceData.formatted).toFixed(4)
      : undefined;

    return (
      <ConnectedDropdown
        displayAddress={displayAddress}
        copied={copied}
        onCopy={() => { void copyToClipboard(displayAddress); }}
        onExplorer={openBSCScan}
        onDisconnect={() => { void handleDisconnect(); }}
        className={className}
        role={user?.is_admin === true ? 'admin' : undefined}
        tierLevel={user?.tier_level}
        permCount={user?.permissions.length}
        chainId={chainId}
        balance={bnbBalance}
      />
    );
  }

  return (
    <div className="flex items-center gap-2">
      <button
        onClick={() => setIsOpen(true)}
        className={cn('flex items-center gap-1.5 sm:gap-2 text-primary-foreground', className)}
      >
        <Wallet className="h-3.5 w-3.5 sm:h-4 sm:w-4 flex-shrink-0" />
        <span className="hidden lg:inline">Connect Wallet</span>
      </button>

      {error !== null && error !== '' && (
        <div className="flex items-center gap-1 text-xs text-red-500" title={error}>
          <AlertCircle className="h-3 w-3" />
        </div>
      )}

      <AuthModal
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        variant="admin"
        onSuccess={() => {
          setIsOpen(false);
          window.location.reload();
        }}
        onError={() => {}}
      />
    </div>
  );
}
