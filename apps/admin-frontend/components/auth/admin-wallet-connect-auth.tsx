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

function MetaInfoItem({ label, icon, value, color }: { label: string; icon?: React.ReactNode; value: React.ReactNode; color?: string }) {
  return (
    <div>
      <p className="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-0.5">{label}</p>
      <div className="flex items-center gap-1">
        {icon}
        <span className={`text-xs font-medium ${color ?? 'text-gray-700 dark:text-gray-300'}`}>{value}</span>
      </div>
    </div>
  );
}

function MetaInfoGrid({ role, tierLevel, permCount, balance }: { role?: string; tierLevel?: string; permCount?: number; balance?: string }) {
  return (
    <div className="px-3 py-2.5 border-b border-gray-200 dark:border-white/10 grid grid-cols-2 gap-x-4 gap-y-2.5">
      {role !== undefined && role !== '' && (
        <MetaInfoItem
          label="Role"
          icon={<Shield className="h-3 w-3 text-purple-500 dark:text-purple-400 flex-shrink-0" />}
          value={role.replace('_', ' ')}
          color="text-purple-600 dark:text-purple-400 capitalize"
        />
      )}
      {tierLevel !== undefined && tierLevel !== '' && (
        <MetaInfoItem label="Tier" value={tierLevel} color="text-cyan-600 dark:text-cyan-400" />
      )}
      {permCount !== undefined && permCount > 0 && (
        <MetaInfoItem
          label="Permissions"
          icon={<Key className="h-3 w-3 text-gray-400 dark:text-gray-500 flex-shrink-0" />}
          value={permCount}
        />
      )}
      {balance !== undefined && balance !== '' && (
        <MetaInfoItem label="Balance" value={`${balance} BNB`} />
      )}
    </div>
  );
}

function NetworkBadge({ chainId }: { chainId?: number }) {
  const chainName = chainId === 56 ? 'BSC Mainnet' : chainId === 97 ? 'BSC Testnet' : chainId !== undefined ? `Chain ${chainId}` : null;
  const isLive = chainId === 56;
  if (chainName === null) { return null; }
  return (
    <div className="px-3 py-2 border-b border-gray-200 dark:border-white/10">
      <div className="flex items-center gap-1.5">
        <span className={cn('h-1.5 w-1.5 rounded-full flex-shrink-0', isLive ? 'bg-green-500' : 'bg-yellow-400')} />
        <span className="text-xs text-gray-600 dark:text-gray-400">{chainName}</span>
      </div>
    </div>
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
        <MetaInfoGrid role={role} tierLevel={tierLevel} permCount={permCount} balance={balance} />

        {/* Network */}
        <NetworkBadge chainId={chainId} />

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

function ConnectedState({ displayAddress, user, copied, chainId, balanceData, copyToClipboard, openBSCScan, handleDisconnect, className }: {
  displayAddress: string;
  user: { is_admin?: boolean; tier_level?: string; permissions: unknown[] };
  copied: boolean;
  chainId: number;
  balanceData?: { formatted: string };
  copyToClipboard: (text: string) => Promise<void>;
  openBSCScan: () => void;
  handleDisconnect: () => Promise<void>;
  className: string;
}) {
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
      role={user.is_admin === true ? 'admin' : undefined}
      tierLevel={user.tier_level}
      permCount={user.permissions.length}
      chainId={chainId}
      balance={bnbBalance}
    />
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

  if (user !== null && displayAddress !== undefined && displayAddress !== '') {
    return (
      <ConnectedState
        displayAddress={displayAddress}
        user={user}
        copied={copied}
        chainId={chainId}
        balanceData={balanceData}
        copyToClipboard={copyToClipboard}
        openBSCScan={openBSCScan}
        handleDisconnect={handleDisconnect}
        className={className}
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
