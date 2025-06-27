'use client';

import { useMemo, useState } from 'react';

import { supportedAssets } from '@/app/constants/assets';

interface AssetSelectionProps {
  selectedAsset: string;
  onSelectAction: (currency: string) => void;
}

export default function AssetSelection({
  selectedAsset,
  onSelectAction,
}: AssetSelectionProps): React.ReactElement {
  const [searchQuery, setSearchQuery] = useState('');
  const [chainFilter, setChainFilter] = useState<string>('');

  const chains = useMemo(() => {
    const uniqueChains = new Set(supportedAssets.map((asset) => asset.chain || ''));
    return Array.from(uniqueChains).filter(Boolean);
  }, []);

  const filteredAssets = useMemo(() => {
    return supportedAssets.filter((asset) => {
      const matchesSearch = asset.currency
        .toLowerCase()
        .includes(searchQuery.toLowerCase());
      const matchesChain = !chainFilter || (asset.chain || '') === chainFilter;
      return matchesSearch && matchesChain;
    });
  }, [searchQuery, chainFilter]);

  return (
    <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
      <div className="mb-6 space-y-4">
        <h2 className="text-xl font-semibold">Select Asset</h2>

        <div className="flex space-x-4">
          <input
            type="text"
            placeholder="Search assets..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 p-2 border rounded-md dark:bg-gray-700 dark:border-gray-600"
          />

          <select
            value={chainFilter}
            onChange={(e) => setChainFilter(e.target.value)}
            className="p-2 border rounded-md dark:bg-gray-700 dark:border-gray-600"
          >
            <option value="">All Chains</option>
            {chains.map((chain) => (
              <option key={chain} value={chain}>
                {chain}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {filteredAssets.map((asset) => (
          <button
            key={asset.currency}
            onClick={() => onSelectAction(asset.currency)}
            className={`p-4 border rounded-lg text-left transition-colors ${
              selectedAsset === asset.currency
                ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                : 'hover:border-gray-300 dark:hover:border-gray-500'
            }`}
          >
            <div className="font-medium">{asset.currency}</div>
            <div className="text-sm text-gray-600 dark:text-gray-400">
              Chain: {asset.chain || 'N/A'}
            </div>
            <div className="text-xs text-gray-500 dark:text-gray-400">
              Min Deposit: {asset.depositThreshold}
            </div>
          </button>
        ))}
      </div>

      {filteredAssets.length === 0 && (
        <div className="text-center text-gray-500 dark:text-gray-400 mt-4">
          No assets found matching your criteria
        </div>
      )}
    </div>
  );
}
