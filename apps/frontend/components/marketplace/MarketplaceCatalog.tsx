/**
 * Marketplace Catalog Placeholder
 * Backend doesn't implement marketplace endpoints yet
 */
'use client';

import { useWeb3Auth } from '@/lib/auth/Web3AuthProvider';

interface MarketplaceCatalogProps {
  initialCategory?: string;
}

export function MarketplaceCatalog({ initialCategory = 'all' }: MarketplaceCatalogProps) {
  const { getWalletAddress } = useWeb3Auth();
  const walletAddress = getWalletAddress();
  
  return (
    <div className="max-w-6xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow p-8 text-center">
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">Marketplace</h1>
          <p className="text-gray-600">
            Backend marketplace endpoints are not implemented yet
          </p>
        </div>
        
        {walletAddress && (
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-6">
            <div className="text-left">
              <h3 className="font-medium text-gray-900 mb-2">Current Wallet</h3>
              <p className="text-sm text-gray-600 font-mono">
                {walletAddress.slice(0, 8)}...{walletAddress.slice(-4)}
              </p>
              <p className="text-sm text-gray-600 mt-1">
                Access managed by backend
              </p>
            </div>
          </div>
        )}
        
        <div className="text-left bg-yellow-50 border border-yellow-200 rounded-lg p-6">
          <h3 className="font-medium text-gray-900 mb-2">Development Note</h3>
          <p className="text-sm text-gray-600">
            The backend currently only implements Web3 wallet authentication and permission endpoints. 
            Marketplace features are not yet available.
          </p>
          <ul className="text-sm text-gray-600 mt-2 list-disc list-inside space-y-1">
            <li>Product catalog</li>
            <li>Integration services</li>
            <li>Professional services</li>
            <li>Shopping cart</li>
          </ul>
        </div>
      </div>
    </div>
  );
}