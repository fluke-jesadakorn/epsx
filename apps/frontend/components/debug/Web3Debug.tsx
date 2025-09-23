'use client';

import { useConnect } from 'wagmi';
import { useWeb3Context } from '@/components/providers/Web3ProviderClient';
import { useEffect, useState } from 'react';

export function Web3Debug() {
  const [mounted, setMounted] = useState(false);
  const { isInitialized, forceRecreateConnectors } = useWeb3Context();
  const { connectors, isPending } = useConnect();

  useEffect(() => {
    setMounted(true);
    console.log('🔍 Web3Debug mounted');
  }, []);

  useEffect(() => {
    if (mounted) {
      console.log('🔍 Web3Debug state update:', {
        isInitialized,
        connectorsCount: connectors?.length || 0,
        connectorsNames: connectors?.map(c => c.name) || [],
        isPending,
        timestamp: new Date().toISOString()
      });
    }
  }, [mounted, isInitialized, connectors, isPending]);

  if (!mounted) {
    return null;
  }

  return (
    <div className="fixed top-20 right-4 p-4 bg-slate-900 text-white rounded-lg text-xs z-50 max-w-xs">
      <div className="font-bold mb-2">Web3 Debug</div>
      <div className="space-y-1">
        <div>Initialized: {isInitialized ? '✅' : '❌'}</div>
        <div>Connectors: {connectors?.length || 0}</div>
        <div>Pending: {isPending ? '⏳' : '✅'}</div>
        {connectors && connectors.length > 0 && (
          <div>
            <div className="font-semibold mt-2">Available:</div>
            {connectors.map((c, i) => (
              <div key={i} className="ml-2">• {c.name}</div>
            ))}
          </div>
        )}
        <button 
          onClick={forceRecreateConnectors}
          className="mt-2 px-2 py-1 bg-orange-500 rounded text-xs hover:bg-orange-600"
        >
          Recreate
        </button>
      </div>
    </div>
  );
}