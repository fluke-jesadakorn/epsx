'use client';

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui';
import { isProduction } from '@/shared/utils';
import { AlertCircle, Link, Wifi, WifiOff } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useAccount, useChainId, useSwitchChain } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

interface ChainSelectorProps {
  className?: string;
  compact?: boolean;
}

interface ChainInfo {
  id: number;
  name: string;
  displayName: string;
  icon: string;
  testnet: boolean;
}

const supportedChains: ChainInfo[] = [
  {
    id: bsc.id,
    name: bsc.name,
    displayName: 'BSC Mainnet',
    icon: '🟡',
    testnet: false,
  },
  {
    id: bscTestnet.id,
    name: bscTestnet.name,
    displayName: 'BSC Testnet',
    icon: '🟠',
    testnet: true,
  },
  {
    id: 31337,
    name: 'Hardhat Local',
    displayName: 'Hardhat Local',
    icon: '🔧',
    testnet: true,
  },
];

export function ChainSelector({ className = '', compact = false }: ChainSelectorProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const chainId = useChainId();
  const { switchChain, isPending: isSwitching } = useSwitchChain();
  const { isConnected } = useAccount();

  useEffect(() => {
    setIsHydrated(true);
  }, []);

  // Hide chain selector in production - keep code for future multi-chain support
  if (isProduction) {
    return null;
  }

  if (!isHydrated) {
    return (
      <Button
        variant="ghost"
        size="icon"
        className={`h-10 w-10 rounded-full bg-orange-50 text-orange-500 hover:bg-orange-100 dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700 ${className}`}
        disabled
      >
        <Wifi className="h-4 w-4 text-orange-500" />
      </Button>
    );
  }

  const currentChain = supportedChains.find(chain => chain.id === chainId);
  const isSupported = !!currentChain;

  const handleChainSwitch = async (targetChainId: number) => {
    if (!isConnected || isSwitching || targetChainId === chainId) return;

    try {
      console.log(`🔄 ChainSelector: Switching to chain ${targetChainId}...`);
      await switchChain({ chainId: targetChainId });
      setIsOpen(false);
      console.log(`✅ ChainSelector: Successfully switched to chain ${targetChainId}`);
    } catch (error: any) {
      console.error('❌ ChainSelector: Failed to switch chain:', error);

      // Handle specific error cases
      if (error?.code === 4902) {
        console.log('🔧 Chain not added to wallet, user needs to add it manually');
      } else if (error?.code === -32002) {
        console.log('⏳ Chain switch request pending, user needs to approve in wallet');
      } else if (error?.code === 4001) {
        console.log('🚫 User rejected the chain switch request');
      }

      // Keep dropdown open on error so user can try again
      // setIsOpen(false); - commented out to keep dropdown open on error
    }
  };

  const getStatusIcon = () => {
    if (!isConnected) return <WifiOff className="h-4 w-4" />;
    if (!isSupported) return <AlertCircle className="h-4 w-4" />;
    return <Wifi className="h-4 w-4" />;
  };

  const getStatusColor = () => {
    if (!isConnected) return 'text-slate-400';
    if (!isSupported) return 'text-red-500';
    return 'text-green-500';
  };

  const getTriggerContent = () => {
    if (compact) {
      return (
        <div className="flex items-center gap-2">
          <Link className="h-4 w-4 text-orange-500" />
          <span className="text-xs font-medium">
            {currentChain?.displayName || 'Unknown'}
          </span>
        </div>
      );
    }

    return (
      <div className="flex items-center gap-2">
        <Link className="h-5 w-5 text-orange-500" />
        <span className="text-sm font-medium">
          {currentChain?.displayName || 'Unknown'}
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
          disabled={isSwitching || !isConnected}
          className={`
            ${compact ? 'h-8 px-3 text-xs' : 'h-10 px-4 text-sm'} 
            rounded-full border-slate-200 bg-white hover:bg-slate-50
            dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-slate-700
            disabled:opacity-50 disabled:cursor-not-allowed
            ${className}
          `}
        >
          {getTriggerContent()}
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent
        align="end"
        style={{ zIndex: 99999 }}
        className="w-56 p-2 bg-white border border-slate-200 shadow-xl dark:bg-slate-900 dark:border-slate-700"
      >
        <div className="px-2 py-1.5 mb-1">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
            Select Network
          </span>
        </div>

        {supportedChains.map((chain) => (
          <DropdownMenuItem
            key={chain.id}
            onClick={() => handleChainSwitch(chain.id)}
            disabled={isSwitching || chain.id === chainId}
            className={`
              flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-colors
              ${chain.id === chainId
                ? 'bg-orange-100 text-orange-800 dark:bg-orange-500/20 dark:text-orange-300'
                : 'text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-800'
              }
              disabled:opacity-50 disabled:cursor-not-allowed
            `}
          >
            <div className={`w-6 h-6 rounded-full flex items-center justify-center shadow-sm ${chain.testnet
              ? chain.id === 31337
                ? 'bg-gradient-to-br from-slate-400 to-slate-600'
                : 'bg-gradient-to-br from-purple-400 to-purple-600'
              : 'bg-gradient-to-br from-yellow-400 to-yellow-600'
              }`}>
              <span className="text-xs text-white">{chain.icon}</span>
            </div>
            <div className="flex-1">
              <div className="text-sm font-medium">{chain.displayName}</div>
              <div className="text-xs text-slate-500 dark:text-slate-400">
                {chain.testnet ? 'Testnet' : 'Mainnet'}
              </div>
            </div>
            {chain.id === chainId && (
              <div className="w-5 h-5 rounded-full bg-green-500 flex items-center justify-center">
                <span className="text-white text-xs">✓</span>
              </div>
            )}
          </DropdownMenuItem>
        ))}

        {!isConnected && (
          <>
            <div className="my-2 border-t border-slate-200 dark:border-slate-700" />
            <div className="px-3 py-2 text-xs text-slate-500 dark:text-slate-400 italic">
              Connect your wallet to switch networks
            </div>
          </>
        )}

        {isConnected && !isSupported && (
          <>
            <div className="my-2 border-t border-slate-200 dark:border-slate-700" />
            <div className="px-3 py-2 text-xs text-red-600 dark:text-red-400">
              Current network not supported
            </div>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}