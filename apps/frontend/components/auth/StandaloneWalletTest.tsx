'use client';

import { useState, useEffect } from 'react';
import { useConnect, useAccount, useDisconnect } from 'wagmi';
import { Button } from '@/components/ui/button';

export function StandaloneWalletTest() {
  const [mounted, setMounted] = useState(false);
  const { connect, connectors, isPending, error } = useConnect();
  const { address, isConnected } = useAccount();
  const { disconnect } = useDisconnect();

  useEffect(() => {
    setMounted(true);
  }, []);

  // Removed debug logging for production use

  if (!mounted) {
    return <div>Loading wallet test...</div>;
  }

  const handleConnect = (connector: any) => {
    connect({ connector });
  };

  const handleDisconnect = () => {
    disconnect();
  };

  return (
    <div className="p-4 border rounded-lg bg-gray-100 dark:bg-gray-800">
      <h3 className="text-lg font-bold mb-4">Standalone Wallet Test</h3>
      
      <div className="mb-4">
        <p><strong>Status:</strong> {isConnected ? 'Connected' : 'Not Connected'}</p>
        {address && <p><strong>Address:</strong> {address}</p>}
        {isPending && <p><strong>Status:</strong> Connecting...</p>}
      </div>

      {error && (
        <div className="mb-4 p-2 bg-red-100 border border-red-300 rounded text-red-700">
          <strong>Error:</strong> {error.message}
        </div>
      )}

      <div className="space-y-2">
        {isConnected ? (
          <Button onClick={handleDisconnect} variant="outline">
            Disconnect Wallet
          </Button>
        ) : (
          <div className="space-y-2">
            <p className="font-medium">Available Connectors ({connectors.length}):</p>
            {connectors.map((connector) => (
              <Button
                key={connector.uid}
                onClick={() => handleConnect(connector)}
                disabled={isPending}
                variant="outline"
                className="w-full text-left"
              >
                {connector.name} - Connected: {String(connector.connected)} - Ready: {String(connector.ready)}
              </Button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}