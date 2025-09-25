'use client';

import { useState, useEffect } from 'react';
import { useAccount, useDisconnect } from 'wagmi';
import { Wallet, ExternalLink, Copy, Check } from 'lucide-react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { formatAddress } from '@/lib/auth/web3-store';
import { useWeb3Context } from '@/components/providers/MinimalWeb3Provider';

interface WalletProviderIconProps {
  className?: string;
  compact?: boolean;
}

interface WalletProviderInfo {
  name: string;
  icon: string;
  color: string;
}

const walletProviders: Record<string, WalletProviderInfo> = {
  metaMask: {
    name: 'MetaMask',
    icon: '🦊',
    color: 'bg-orange-500',
  },
  walletConnect: {
    name: 'WalletConnect',
    icon: '🔗',
    color: 'bg-blue-500',
  },
  injected: {
    name: 'Browser Wallet',
    icon: '🌐',
    color: 'bg-purple-500',
  },
  coinbase: {
    name: 'Coinbase',
    icon: '🔵',
    color: 'bg-blue-600',
  },
  rainbow: {
    name: 'Rainbow',
    icon: '🌈',
    color: 'bg-gradient-to-r from-pink-500 to-violet-500',
  },
};

export function WalletProviderIcon({ className = '', compact = false }: WalletProviderIconProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const { address, isConnected, connector } = useAccount();
  const { disconnect } = useDisconnect();
  const { isInitialized } = useWeb3Context();

  useEffect(() => {
    setIsHydrated(true);
    console.log('🔍 WalletProviderIcon Debug:', {
      isConnected,
      address,
      connector: connector?.id,
      isHydrated: true,
      isInitialized
    });
  }, [isConnected, address, connector, isInitialized]);

  const handleCopyAddress = async () => {
    if (!address) return;
    
    try {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy address:', error);
    }
  };

  // Show loading state during hydration or before RainbowKit is ready
  if (!isHydrated || !isInitialized) {
    return (
      <Button
        variant="ghost"
        size="icon"
        className={`h-10 w-10 rounded-full bg-orange-50 text-orange-500 hover:bg-orange-100 dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700 ${className}`}
        disabled
      >
        <Wallet className="h-4 w-4" />
      </Button>
    );
  }

  // Only render connect button when RainbowKit is fully initialized
  if (!isConnected || !address) {
    return (
      <ConnectButton.Custom>
        {({ openConnectModal, connectModalOpen, mounted }) => {
          return (
            <Button
              onClick={openConnectModal}
              variant="default"
              size={compact ? "sm" : "default"}
              className={`
                ${compact ? 'h-8 px-3 text-xs' : 'h-10 px-4 text-sm'} 
                rounded-full bg-gradient-to-r from-yellow-400 to-orange-500 
                hover:from-yellow-500 hover:to-orange-600 
                text-white font-semibold shadow-lg hover:shadow-xl
                border-0 transition-all duration-200
                ${className}
              `}
            >
              <Wallet className={`${compact ? 'h-3 w-3' : 'h-4 w-4'} mr-2 text-white`} />
              Connect
            </Button>
          );
        }}
      </ConnectButton.Custom>
    );
  }

  // Detect wallet provider
  const connectorId = connector?.id?.toLowerCase() || 'injected';
  const providerInfo = walletProviders[connectorId] || walletProviders.injected;

  const getTriggerContent = () => {
    if (compact) {
      return (
        <div className="flex items-center gap-2">
          <Wallet className="h-4 w-4 text-orange-500" />
          <span className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {formatAddress(address)}
          </span>
        </div>
      );
    }

    return (
      <div className="flex items-center gap-2">
        <Wallet className="h-5 w-5 text-orange-500" />
        <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
          {formatAddress(address)}
        </span>
      </div>
    );
  };

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size={compact ? "sm" : "default"}
          className={`
            ${compact ? 'h-8 px-3 text-xs' : 'h-10 px-4 text-sm'} 
            rounded-full border-slate-200 bg-white hover:bg-slate-50
            dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-slate-700
            ${className}
          `}
        >
          {getTriggerContent()}
        </Button>
      </DropdownMenuTrigger>
      
      <DropdownMenuContent 
        align="end" 
        className="w-64 p-2 bg-white/95 backdrop-blur-xl border border-orange-100/50 dark:bg-slate-900/95 dark:border-slate-700/50"
      >
        {/* Wallet Header */}
        <div className="px-3 py-2 mb-2">
          <div className="flex items-center gap-3">
            <Wallet className="h-5 w-5 text-orange-500" />
            <div>
              <div className="text-sm font-medium text-slate-900 dark:text-slate-100">
                {providerInfo.name}
              </div>
              <div className="text-xs text-slate-500 dark:text-slate-400">
                Connected
              </div>
            </div>
          </div>
        </div>

        <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />

        {/* Wallet Address */}
        <DropdownMenuItem
          onClick={handleCopyAddress}
          className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40"
        >
          <div className="flex items-center gap-2 flex-1">
            {copied ? (
              <Check className="h-4 w-4 text-orange-500" />
            ) : (
              <Copy className="h-4 w-4 text-orange-500" />
            )}
            <div>
              <div className="text-sm font-medium">
                {copied ? 'Copied!' : 'Copy Address'}
              </div>
              <div className="text-xs text-slate-500 dark:text-slate-400 font-mono">
                {formatAddress(address)}
              </div>
            </div>
          </div>
        </DropdownMenuItem>

        {/* View on Explorer */}
        <DropdownMenuItem
          onClick={() => {
            const explorerUrl = `https://bscscan.com/address/${address}`;
            window.open(explorerUrl, '_blank', 'noopener,noreferrer');
            setIsOpen(false);
          }}
          className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-orange-50/80 dark:hover:bg-slate-800/40"
        >
          <ExternalLink className="h-4 w-4 text-orange-500" />
          <div>
            <div className="text-sm font-medium">View on Explorer</div>
            <div className="text-xs text-slate-500 dark:text-slate-400">
              Open in BSCScan
            </div>
          </div>
        </DropdownMenuItem>

        <DropdownMenuSeparator className="bg-orange-100/50 dark:bg-slate-700/50" />

        {/* Disconnect Wallet */}
        <DropdownMenuItem
          onClick={() => {
            disconnect();
            setIsOpen(false);
          }}
          className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-red-50/80 dark:hover:bg-red-900/20 text-red-600 dark:text-red-400"
        >
          <Wallet className="h-4 w-4 text-red-600 dark:text-red-400" />
          <div>
            <div className="text-sm font-medium">Disconnect</div>
            <div className="text-xs opacity-75">
              Disconnect your wallet
            </div>
          </div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}