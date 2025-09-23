'use client';

import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { useTheme } from 'next-themes';
import { Wallet, ChevronDown, ExternalLink, Copy, LogOut, RefreshCw, AlertCircle, Loader2 } from 'lucide-react';
import { useState, useEffect, useCallback } from 'react';
import { useAccount, useConnect, useDisconnect } from 'wagmi';
import { useWeb3Context } from '@/components/providers/SimpleWeb3Provider';
import { cn } from '@/lib/utils';
import { detectStateCorruption, resetWalletState } from '@/lib/utils/wallet-state-reset';
import { toast } from 'sonner';

interface CustomWalletModalProps {
  children?: React.ReactNode;
  className?: string;
}

export function CustomWalletModal({ children, className }: CustomWalletModalProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [connectionAttempts, setConnectionAttempts] = useState(0);
  const [lastConnectionError, setLastConnectionError] = useState<string | null>(null);
  const [isRecovering, setIsRecovering] = useState(false);
  
  const { resolvedTheme } = useTheme();
  const { address, isConnected, connector } = useAccount();
  const { connectors, connect, isPending, error } = useConnect();
  const { disconnect } = useDisconnect();
  const { isInitialized } = useWeb3Context();

  useEffect(() => {
    setMounted(true);
  }, []);

  // Reset modal state when wallet connects/disconnects
  useEffect(() => {
    if (isConnected) {
      setIsOpen(false);
      setConnectionAttempts(0);
      setLastConnectionError(null);
    }
  }, [isConnected]);

  // Connection timeout and retry logic
  useEffect(() => {
    if (isPending) {
      const timeout = setTimeout(() => {
        if (isPending && !isConnected) {
          console.warn('⚠️ Connection timeout detected');
          setLastConnectionError('Connection timeout - please try again');
          setConnectionAttempts(prev => prev + 1);
        }
      }, 15000); // 15 second timeout

      return () => clearTimeout(timeout);
    }
  }, [isPending, isConnected]);

  // Auto-recovery for persistent connection failures
  const handleConnectionRecovery = useCallback(async () => {
    if (connectionAttempts >= 3 && !isRecovering) {
      console.log('🔧 Connection recovery needed - please refresh page');
      setIsRecovering(true);
      
      try {
        // Simple recovery - clear local state and suggest refresh
        setConnectionAttempts(0);
        setLastConnectionError('Too many failed attempts - please refresh the page');
        toast.error('Connection failed repeatedly - please refresh the page');
      } catch (error) {
        console.error('Recovery failed:', error);
        toast.error('Recovery failed - please refresh the page');
      } finally {
        setIsRecovering(false);
      }
    }
  }, [connectionAttempts, isRecovering]);

  useEffect(() => {
    if (connectionAttempts >= 3) {
      handleConnectionRecovery();
    }
  }, [connectionAttempts, handleConnectionRecovery]);

  const walletIcons = {
    'MetaMask': '🦊',
    'WalletConnect': '🔗', 
    'Injected': '💼',
  };

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  if (!mounted) {
    return (
      <Button variant="outline" disabled className="relative">
        <Wallet className="mr-2 h-4 w-4" />
        Connect Wallet
      </Button>
    );
  }

  if (isConnected && address) {
    return (
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button 
            variant="outline" 
            className={cn(
              "relative overflow-hidden rounded-2xl border-2",
              "bg-gradient-to-r from-orange-50 to-yellow-50 dark:from-orange-900/20 dark:to-yellow-900/20",
              "border-orange-200/50 dark:border-orange-400/30",
              "hover:from-orange-100 hover:to-yellow-100 dark:hover:from-orange-900/30 dark:hover:to-yellow-900/30",
              "shadow-lg hover:shadow-xl backdrop-blur-sm",
              className
            )}
          >
            {/* Background decorations */}
            <div className="absolute -top-2 -right-2 h-6 w-6 rounded-full bg-gradient-to-br from-orange-300/20 to-yellow-300/20 blur-sm" />
            <div className="absolute -bottom-1 -left-1 h-4 w-4 rounded-full bg-gradient-to-br from-blue-300/20 to-cyan-300/20 blur-sm" />
            
            <div className="relative z-10 flex items-center gap-2">
              <div className="h-2 w-2 rounded-full bg-green-400 animate-pulse" />
              <span className="font-medium text-gray-700 dark:text-gray-200">
                {formatAddress(address)}
              </span>
              <ChevronDown className="h-4 w-4 text-gray-500" />
            </div>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent 
          className={cn(
            "w-64 p-2 rounded-2xl border-2",
            "bg-white/95 dark:bg-slate-800/95 backdrop-blur-xl",
            "border-orange-200/50 dark:border-orange-400/30",
            "shadow-2xl"
          )}
        >
          {/* Header */}
          <div className="px-3 py-2 border-b border-orange-200/30 dark:border-orange-400/20">
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <span className="text-lg">{walletIcons[connector?.name as keyof typeof walletIcons] || '💼'}</span>
                <div>
                  <p className="font-semibold text-sm text-gray-800 dark:text-gray-100">
                    {connector?.name || 'Wallet'}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400">Connected</p>
                </div>
              </div>
              <div className="h-2 w-2 rounded-full bg-green-400" />
            </div>
          </div>

          {/* Address Section */}
          <div className="p-3 bg-gradient-to-r from-orange-50/50 to-yellow-50/50 dark:from-orange-900/20 dark:to-yellow-900/20 rounded-xl m-2">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Address</p>
            <div className="flex items-center justify-between">
              <span className="font-mono text-sm text-gray-700 dark:text-gray-200">
                {formatAddress(address)}
              </span>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => copyToClipboard(address)}
                className="h-6 w-6 p-0 hover:bg-orange-100/50 dark:hover:bg-orange-900/30"
              >
                <Copy className="h-3 w-3" />
              </Button>
            </div>
          </div>

          {/* Actions */}
          <DropdownMenuItem 
            onClick={() => window.open(`https://bscscan.com/address/${address}`, '_blank')}
            className="m-1 rounded-xl hover:bg-orange-50 dark:hover:bg-orange-900/30 cursor-pointer"
          >
            <ExternalLink className="mr-3 h-4 w-4" />
            View on BSCScan
          </DropdownMenuItem>

          <DropdownMenuItem 
            onClick={() => disconnect()}
            className="m-1 rounded-xl hover:bg-red-50 dark:hover:bg-red-900/30 text-red-600 dark:text-red-400 cursor-pointer"
          >
            <LogOut className="mr-3 h-4 w-4" />
            Disconnect
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        {children || (
          <Button 
            variant="outline"
            className={cn(
              "relative overflow-hidden rounded-2xl border-2",
              "bg-gradient-to-r from-orange-50 to-yellow-50 dark:from-orange-900/20 dark:to-yellow-900/20",
              "border-orange-200/50 dark:border-orange-400/30",
              "hover:from-orange-100 hover:to-yellow-100 dark:hover:from-orange-900/30 dark:hover:to-yellow-900/30",
              "shadow-lg hover:shadow-xl backdrop-blur-sm",
              className
            )}
          >
            {/* Background decorations matching homepage */}
            <div className="absolute -top-2 -right-2 h-6 w-6 rounded-full bg-gradient-to-br from-orange-300/20 to-yellow-300/20 blur-sm" />
            <div className="absolute -bottom-1 -left-1 h-4 w-4 rounded-full bg-gradient-to-br from-blue-300/20 to-cyan-300/20 blur-sm" />
            
            <div className="relative z-10 flex items-center gap-2">
              <Wallet className="h-4 w-4" />
              <span className="font-semibold">Connect Wallet</span>
            </div>
          </Button>
        )}
      </DialogTrigger>

      <DialogContent 
        className={cn(
          "max-w-md p-0 overflow-hidden rounded-3xl border-2",
          "bg-white/95 dark:bg-slate-800/95 backdrop-blur-xl",
          "border-orange-200/50 dark:border-orange-400/30",
          "shadow-2xl"
        )}
      >
        {/* Background decorations matching homepage */}
        <div className="absolute -top-8 -left-8 h-24 w-24 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-2xl" />
        <div className="absolute -bottom-8 -right-8 h-32 w-32 rounded-full bg-gradient-to-br from-blue-400/15 to-cyan-400/15 blur-2xl" />
        <div className="absolute top-1/2 right-1/4 h-16 w-16 rounded-full bg-gradient-to-br from-purple-400/10 to-pink-400/10 blur-xl" />

        <div className="relative z-10 p-6">
          {/* Header */}
          <div className="text-center mb-6">
            <div className="flex items-center justify-center mb-4">
              <div className="relative">
                <div className="h-16 w-16 rounded-2xl bg-gradient-to-br from-orange-400/20 to-yellow-400/20 flex items-center justify-center backdrop-blur-sm">
                  <Wallet className="h-8 w-8 text-orange-500 dark:text-orange-400" />
                </div>
                <div className="absolute -top-1 -right-1 h-4 w-4 rounded-full bg-gradient-to-br from-blue-400 to-cyan-400" />
              </div>
            </div>
            <h2 className="text-2xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent dark:from-orange-400 dark:to-yellow-400">
              Connect Your Wallet
            </h2>
            <p className="text-sm text-gray-600 dark:text-gray-300 mt-2">
              Choose your preferred wallet to connect to EPSX
            </p>
          </div>

          {/* Wallet Options */}
          <div className="space-y-3">
            {connectors.map((connector) => (
              <button
                key={connector.uid}
                onClick={async () => {
                  try {
                    setLastConnectionError(null);
                    console.log(`🔌 Attempting to connect to ${connector.name}...`);
                    
                    // Check for state corruption before connecting
                    if (detectStateCorruption()) {
                      console.warn('⚠️ State corruption detected before connection attempt');
                      toast.error('Please refresh the page and try again');
                      return;
                    }
                    
                    connect({ connector });
                  } catch (err: any) {
                    console.error('Connection failed:', err);
                    const errorMsg = err?.message || 'Connection failed';
                    setLastConnectionError(errorMsg);
                    setConnectionAttempts(prev => prev + 1);
                    toast.error(`Failed to connect: ${errorMsg}`);
                  }
                }}
                disabled={isPending || isRecovering || isResetting}
                className={cn(
                  "w-full p-4 rounded-2xl border-2 text-left",
                  "bg-gradient-to-r from-white/80 to-orange-50/80 dark:from-slate-700/80 dark:to-orange-900/20",
                  "border-orange-200/50 dark:border-orange-400/30",
                  "hover:from-orange-50 hover:to-yellow-50 dark:hover:from-orange-900/30 dark:hover:to-yellow-900/30",
                  "hover:border-orange-300/70 dark:hover:border-orange-400/50",
                  "shadow-lg hover:shadow-xl backdrop-blur-sm",
                  "disabled:opacity-50 disabled:cursor-not-allowed",
                  "group relative overflow-hidden"
                )}
              >
                {/* Hover effect */}
                <div className="absolute inset-0 bg-gradient-to-r from-orange-400/5 to-yellow-400/5 opacity-0 group-hover:opacity-100" />
                
                <div className="relative z-10 flex items-center gap-4">
                  <div className="text-2xl">
                    {walletIcons[connector.name as keyof typeof walletIcons] || '💼'}
                  </div>
                  <div className="flex-1">
                    <p className="font-semibold text-gray-800 dark:text-gray-100">
                      {connector.name}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {connector.name === 'MetaMask' && 'Browser extension wallet'}
                      {connector.name === 'WalletConnect' && 'Scan QR code with mobile wallet'}
                      {connector.name === 'Injected' && 'Browser wallet'}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    {isPending && (
                      <Loader2 className="h-4 w-4 animate-spin text-blue-500" />
                    )}
                    {isRecovering && (
                      <RefreshCw className="h-4 w-4 animate-spin text-orange-500" />
                    )}
                    <div className="h-2 w-2 rounded-full bg-orange-400 group-hover:bg-yellow-400" />
                  </div>
                </div>
              </button>
            ))}
          </div>

          {/* Enhanced Error Display */}
          {(error || lastConnectionError) && (
            <div className="mt-4 p-3 rounded-xl bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
              <div className="flex items-start gap-3">
                <AlertCircle className="h-5 w-5 text-red-500 flex-shrink-0 mt-0.5" />
                <div className="flex-1">
                  <p className="text-sm font-medium text-red-800 dark:text-red-200">
                    Connection Failed
                  </p>
                  <p className="text-sm text-red-600 dark:text-red-400 mt-1">
                    {error?.message || lastConnectionError || 'Please try again'}
                  </p>
                  {connectionAttempts > 0 && (
                    <p className="text-xs text-red-500 dark:text-red-400 mt-2">
                      Attempts: {connectionAttempts}/3
                      {connectionAttempts >= 3 && ' - Auto-recovery in progress...'}
                    </p>
                  )}
                </div>
                <div className="flex flex-col gap-2">
                  <button
                    onClick={() => {
                      setLastConnectionError(null);
                      setConnectionAttempts(0);
                    }}
                    className="text-xs px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
                    title="Clear error"
                  >
                    Clear
                  </button>
                  {connectionAttempts >= 2 && (
                    <button
                      onClick={async () => {
                        setIsRecovering(true);
                        try {
                          await resetWalletConnection();
                          setConnectionAttempts(0);
                          setLastConnectionError(null);
                          toast.info('Connection reset - please try again');
                        } catch (err) {
                          toast.error('Reset failed');
                        } finally {
                          setIsRecovering(false);
                        }
                      }}
                      disabled={isRecovering || isResetting}
                      className="flex items-center gap-1 text-xs px-2 py-1 bg-orange-500 text-white rounded hover:bg-orange-600 disabled:opacity-50"
                      title="Reset connection state"
                    >
                      <RefreshCw className={cn("h-3 w-3", isRecovering && "animate-spin")} />
                      Reset
                    </button>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Recovery Status Display */}
          {isRecovering && (
            <div className="mt-4 p-3 rounded-xl bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800">
              <div className="flex items-center gap-3">
                <RefreshCw className="h-5 w-5 text-orange-500 animate-spin" />
                <div>
                  <p className="text-sm font-medium text-orange-800 dark:text-orange-200">
                    Recovering Connection State
                  </p>
                  <p className="text-sm text-orange-600 dark:text-orange-400">
                    Please wait while we reset the connection...
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Footer */}
          <div className="mt-6 text-center">
            <p className="text-xs text-gray-500 dark:text-gray-400">
              By connecting, you agree to EPSX{' '}
              <span className="text-orange-500 hover:text-orange-600 cursor-pointer">
                Terms of Service
              </span>
            </p>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}