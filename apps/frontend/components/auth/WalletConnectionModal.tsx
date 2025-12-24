'use client';

import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';
import { useUnifiedWeb3 } from '@/shared/components/providers/UnifiedWeb3Provider';
import { AlertCircle, ChevronDown, Loader2, RefreshCw, Wallet } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { useConnect } from 'wagmi';

interface WalletConnectionModalProps {
  children?: React.ReactNode;
  className?: string;
}

export function WalletConnectionModal({ children, className }: WalletConnectionModalProps) {
  const [lastConnectionError, setLastConnectionError] = useState<string | null>(null);

  const { connect, connectors, isPending, error } = useConnect();
  const { forceReset, forceRecreateConnectors, isInitialized } = useUnifiedWeb3();

  const walletIcons = {
    'MetaMask': '🦊',
    'WalletConnect': '🔗',
    'Injected': '💼',
  };

  const getWalletIcon = (name: string) => {
    return walletIcons[name as keyof typeof walletIcons] || '💼';
  };

  const getWalletDescription = (name: string) => {
    switch (name) {
      case 'MetaMask':
        return 'Recent';
      case 'WalletConnect':
        return 'Scan QR code with mobile wallet';
      case 'Injected':
        return 'Browser wallet';
      default:
        return 'Connect using this wallet';
    }
  };

  // Connection timeout handling
  useEffect(() => {
    if (isPending) {
      const timeout = setTimeout(() => {
        if (isPending) {
          setLastConnectionError('Connection timeout - please try again');
        }
      }, 15000);

      return () => clearTimeout(timeout);
    }
  }, [isPending]);

  const handleConnect = async (connector: any, retryCount = 0) => {
    const maxRetries = 3;

    try {
      setLastConnectionError(null);

      // Handle undefined connector states with retry logic
      if (connector.connected === undefined || connector.ready === undefined) {
        if (retryCount < maxRetries) {
          const waitTime = (retryCount + 1) * 500;
          await new Promise(resolve => setTimeout(resolve, waitTime));

          if ((connector.connected === undefined || connector.ready === undefined) && retryCount === maxRetries - 1) {
            if (forceRecreateConnectors) {
              toast.info('Initializing wallet connectors...');
              forceRecreateConnectors();
              await new Promise(resolve => setTimeout(resolve, 2000));
              return;
            }
          }

          return handleConnect(connector, retryCount + 1);
        } else {
          throw new Error('Wallet connector not ready. Please refresh the page and try again.');
        }
      }

      // Force connector ready state if needed
      if (connector.ready === false) {
        if (typeof connector.prepare === 'function') {
          await connector.prepare();
        }
        await new Promise(resolve => setTimeout(resolve, 300));
      }

      // Reset connector if already connected
      if (connector.connected === true) {
        try {
          if (typeof connector.disconnect === 'function') {
            await connector.disconnect();
          }

          if (typeof connector.reset === 'function') {
            await connector.reset();
          }

          try {
            if (connector.connected) {
              connector.connected = false;
            }
          } catch (propError) {
            // Property might be readonly
          }

          await new Promise(resolve => setTimeout(resolve, 200));

        } catch (resetError) {
          if (retryCount < maxRetries) {
            return handleConnect(connector, retryCount + 1);
          }
        }
      }

      // Final state validation
      if (connector.connected && retryCount >= maxRetries) {
        if (forceRecreateConnectors) {
          toast.info('Recreating wallet connectors...');
          forceRecreateConnectors();
          setTimeout(() => {
            handleConnect(connector, 0);
          }, 1000);
          return;
        } else if (forceReset) {
          toast.info('Resetting wallet connection state...');
          forceReset();
          return;
        } else {
          throw new Error('Connector state corrupted. Please refresh the page.');
        }
      }

      // Attempt connection
      const result = connect({ connector });

    } catch (err: any) {
      let errorMsg = err?.message || 'Connection failed';

      // Handle specific error types with recovery
      if (errorMsg.includes('Connector already connected')) {
        if (retryCount < maxRetries) {
          await new Promise(resolve => setTimeout(resolve, 500));
          return handleConnect(connector, retryCount + 1);
        } else {
          errorMsg = 'Wallet connection state corrupted. Click "Reset" below or refresh the page.';
        }
      } else if (errorMsg.includes('User rejected')) {
        errorMsg = 'Connection was cancelled by user';
      } else if (errorMsg.includes('No provider found')) {
        errorMsg = 'Wallet not found. Please install MetaMask or your preferred wallet.';
      } else if (errorMsg.includes('Failed to connect')) {
        if (retryCount < maxRetries) {
          await new Promise(resolve => setTimeout(resolve, 300));
          return handleConnect(connector, retryCount + 1);
        }
      }

      setLastConnectionError(errorMsg);
      toast.error(errorMsg);
    }
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        {children || (
          <Button
            variant="outline"
            className={cn(
              "flex items-center gap-2 px-4 py-2",
              "bg-slate-800 hover:bg-slate-700 border-slate-600",
              "text-white rounded-lg",
              className
            )}
          >
            <Wallet className="h-4 w-4 text-orange-500" />
            <span className="font-medium">Connect Wallet</span>
            <ChevronDown className="h-4 w-4 text-slate-400" />
          </Button>
        )}
      </DropdownMenuTrigger>

      <DropdownMenuContent
        className={cn(
          "w-80 p-0 bg-slate-800 border-slate-600",
          "rounded-2xl shadow-xl"
        )}
        align="end"
        sideOffset={8}
      >
        {/* Header */}
        <div className="px-4 py-4 border-b border-slate-700">
          <h2 className="text-xl font-bold text-white">Connect a Wallet</h2>
        </div>

        {/* Recommended Section */}
        <div className="px-4 py-2">
          <div className="text-sm text-slate-400 mb-3">Recommended</div>

          {!isInitialized ? (
            // Show loading state while connectors are being initialized
            <div className="flex items-center justify-center py-8">
              <div className="text-center">
                <Loader2 className="h-6 w-6 animate-spin text-orange-500 mx-auto mb-2" />
                <p className="text-sm text-slate-400">Setting up wallet connectors...</p>
                <p className="text-xs text-slate-500 mt-1">Initializing MetaMask, WalletConnect...</p>
              </div>
            </div>
          ) : connectors.length === 0 ? (
            // Show error state if no connectors available
            <div className="flex items-center justify-center py-8">
              <div className="text-center">
                <AlertCircle className="h-6 w-6 text-red-500 mx-auto mb-2" />
                <p className="text-sm text-slate-400 mb-2">No wallet connectors available</p>
                <button
                  onClick={() => {
                    toast.info('Recreating wallet connectors...');
                    forceRecreateConnectors();
                  }}
                  className="px-3 py-1 bg-orange-500 text-white rounded hover:bg-orange-600 text-xs"
                >
                  Try Again
                </button>
              </div>
            </div>
          ) : (
            // Show connectors when ready
            connectors.map((connector) => (
              <DropdownMenuItem
                key={connector.uid}
                className={cn(
                  "flex items-center gap-4 px-0 py-3 cursor-pointer",
                  "hover:bg-slate-700 focus:bg-slate-700 rounded-lg mb-2",
                  "disabled:opacity-50 disabled:cursor-not-allowed"
                )}
                onClick={() => handleConnect(connector)}
                disabled={isPending}
              >
                <div className="flex items-center gap-4 w-full px-3">
                  <div className="w-10 h-10 rounded-lg bg-slate-700 flex items-center justify-center text-2xl">
                    {getWalletIcon(connector.name)}
                  </div>
                  <div className="flex-1">
                    <div className="font-semibold text-white">
                      {connector.name}
                    </div>
                    <div className="text-sm text-slate-400">
                      {getWalletDescription(connector.name)}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    {isPending && (
                      <Loader2 className="h-4 w-4 animate-pulse text-orange-500" />
                    )}
                  </div>
                </div>
              </DropdownMenuItem>
            ))
          )}
        </div>

        {/* Footer */}
        <DropdownMenuSeparator className="bg-slate-700" />
        <div className="px-4 py-3 flex items-center justify-between">
          <span className="text-sm text-slate-400">New to Ethereum wallets?</span>
          <span className="text-sm text-orange-400 hover:text-orange-300 cursor-pointer">
            Learn More
          </span>
        </div>

        {/* Enhanced Error Display with Multiple Recovery Options */}
        {(error || lastConnectionError) && (
          <>
            <DropdownMenuSeparator className="bg-slate-700" />
            <div className="px-4 py-3">
              <div className="flex items-start gap-3">
                <AlertCircle className="h-4 w-4 text-red-400 flex-shrink-0 mt-0.5" />
                <div className="flex-1">
                  <p className="text-sm font-medium text-red-300">
                    Connection Failed
                  </p>
                  <p className="text-sm text-red-400 mt-1">
                    {error?.message || lastConnectionError || 'Failed to connect wallet'}
                  </p>
                </div>
                <div className="flex flex-col gap-1">
                  <button
                    onClick={() => setLastConnectionError(null)}
                    className="text-xs px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
                  >
                    Clear
                  </button>
                  {(lastConnectionError?.includes('already connected') ||
                    lastConnectionError?.includes('corrupted') ||
                    lastConnectionError?.includes('state')) && (
                      <>
                        <button
                          onClick={() => {
                            toast.info('Recreating wallet connectors...');
                            forceRecreateConnectors();
                          }}
                          className="flex items-center gap-1 text-xs px-2 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
                          title="Recreate fresh connector instances"
                        >
                          <RefreshCw className="h-3 w-3" />
                          Recreate
                        </button>
                        <button
                          onClick={() => {
                            toast.info('Resetting connection state...');
                            forceReset();
                          }}
                          className="flex items-center gap-1 text-xs px-2 py-1 bg-orange-500 text-white rounded hover:bg-orange-600"
                          title="Full reset with page reload"
                        >
                          <RefreshCw className="h-3 w-3" />
                          Reset
                        </button>
                      </>
                    )}
                </div>
              </div>
            </div>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}