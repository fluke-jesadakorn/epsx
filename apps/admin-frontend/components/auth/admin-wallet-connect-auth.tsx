'use client';

import { AlertCircle, ChevronDown, Loader2, Wallet , Check, Copy, ExternalLink, LogOut } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useAccount, useDisconnect } from 'wagmi';
import { AuthModal, useSharedAuth } from '@/shared/components/auth';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
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

function ConnectedDropdown({ displayAddress, copied, onCopy, onExplorer, onDisconnect }: {
  displayAddress: string;
  copied: boolean;
  onCopy: () => void;
  onExplorer: () => void;
  onDisconnect: () => void;
  className?: string;
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
        className="w-52 bg-white dark:bg-[#1a1d2e] border border-border/20 p-1"
        style={{ zIndex: 99999 }}
      >
        <div className="px-3 py-2 text-xs text-muted-foreground border-b border-border mb-1 font-mono break-all">
          {displayAddress}
        </div>
        <DropdownMenuItem
          onClick={onCopy}
          className="flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer"
        >
          {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
          {copied ? 'Copied!' : 'Copy Address'}
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={onExplorer}
          className="flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer"
        >
          <ExternalLink className="h-4 w-4" />
          View on Explorer
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={onDisconnect}
          className="flex items-center gap-2 px-3 py-2 rounded-md text-red-400 hover:bg-red-500/20 cursor-pointer"
        >
          <LogOut className="h-4 w-4" />
          Disconnect
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function AdminWalletConnectAuth({ className = '' }: AdminWalletConnectAuthProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  const { address, isConnected: _isConnected } = useAccount();
  const { disconnect } = useDisconnect();
  const { user, isLoading, error, logout } = useSharedAuth();

  const isAuthenticated = Boolean(user);

  useEffect(() => {
    setIsHydrated(true);
  }, []);

  if (!isHydrated || isLoading) {
    return <LoadingButton message="Loading..." className={className} />;
  }

  const displayAddress = user?.wallet_address ?? address;

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
      window.location.href = '/';
    } catch {
      // Silently fail
    }
  };

  if (isAuthenticated && displayAddress !== undefined && displayAddress !== '') {
    return (
      <ConnectedDropdown
        displayAddress={displayAddress}
        copied={copied}
        onCopy={() => { void copyToClipboard(displayAddress); }}
        onExplorer={openBSCScan}
        onDisconnect={() => { void handleDisconnect(); }}
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
