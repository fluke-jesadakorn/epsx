'use client';

import { useState, useEffect } from 'react';
import { useChainId, useSwitchChain, useAccount } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';
import { ChevronDown, Wifi, WifiOff, AlertCircle, Link } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Button,
} from '@/components/ui';

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
        className="w-48 p-2 bg-white/95 backdrop-blur-xl border border-orange-100/50 dark:bg-slate-900/95 dark:border-slate-700/50"
      >
        <div className="px-2 py-1 mb-2">
          <span className="text-xs font-medium text-slate-600 dark:text-slate-400">
            Select Network
          </span>
        </div>
        
        {supportedChains.map((chain) => (
          <DropdownMenuItem
            key={chain.id}
            onClick={() => handleChainSwitch(chain.id)}
            disabled={isSwitching || chain.id === chainId}
            className={`
              flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer
              hover:bg-orange-50/80 dark:hover:bg-slate-800/40
              ${chain.id === chainId ? 'bg-orange-50/80 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300' : ''}
              disabled:opacity-50 disabled:cursor-not-allowed
            `}
          >
            <Link className="h-4 w-4 text-orange-500" />
            <div className="flex-1">
              <div className="text-sm font-medium">{chain.displayName}</div>
              <div className="text-xs text-slate-500 dark:text-slate-400">
                {chain.testnet ? 'Testnet' : 'Mainnet'}
              </div>
            </div>
            {chain.id === chainId && (
              <Wifi className="h-4 w-4 text-orange-500" />
            )}
          </DropdownMenuItem>
        ))}

        {!isConnected && (
          <div className="px-3 py-2 mt-2 text-xs text-slate-500 dark:text-slate-400 border-t border-slate-200 dark:border-slate-700">
            Connect your wallet to switch networks
          </div>
        )}

        {isConnected && !isSupported && (
          <div className="px-3 py-2 mt-2 text-xs text-red-500 border-t border-slate-200 dark:border-slate-700">
            Current network not supported
          </div>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}