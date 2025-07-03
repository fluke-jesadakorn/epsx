'use client';

import { Check } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { BLOCKCHAIN_CONFIG } from '@/app/constants/packages';
import type { CurrencyType } from '@/app/constants/packages';

interface AssetSelectionProps {
  selectedAsset: string;
  onSelect: (asset: string) => void;
}

interface NetworkAsset {
  id: CurrencyType;
  name: string;
  network: string;
  icon: string;
  color: string;
}

const NETWORK_ASSETS: NetworkAsset[] = [
  {
    id: 'USDT_TRC20',
    name: 'USDT',
    network: 'TRC20',
    icon: '/icons/trc20.svg',
    color: 'from-red-500/20 to-red-600/20'
  },
  {
    id: 'USDT_BSC',
    name: 'USDT',
    network: BLOCKCHAIN_CONFIG.BSC.name,
    icon: '/icons/bsc.svg',
    color: 'from-yellow-500/20 to-yellow-600/20'
  },
  {
    id: 'USDT_ERC20',
    name: 'USDT',
    network: 'ERC20',
    icon: '/icons/erc20.svg',
    color: 'from-blue-500/20 to-blue-600/20'
  },
  {
    id: 'USDT_ARB',
    name: 'USDT',
    network: 'Arbitrum',
    icon: '/icons/arbitrum.svg',
    color: 'from-purple-500/20 to-purple-600/20'
  }
];

export default function AssetSelection({
  selectedAsset,
  onSelect
}: AssetSelectionProps) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
      {NETWORK_ASSETS.map((asset) => (
        <Card
          key={asset.id}
          className={`relative cursor-pointer transition-all duration-300 hover:scale-[1.02] hover:shadow-lg ${
            selectedAsset === asset.id
              ? 'border-primary shadow-lg'
              : 'hover:border-primary/50'
          }`}
          onClick={() => onSelect(asset.id)}
        >
          <div
            className={`absolute inset-0 bg-gradient-to-br ${asset.color} rounded-lg opacity-50`}
          />
          <CardContent className="relative p-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="h-10 w-10 rounded-full bg-background/80 p-2 backdrop-blur-sm">
                  <img
                    src={asset.icon}
                    alt={asset.name}
                    className="h-full w-full object-contain"
                  />
                </div>
                <div>
                  <h3 className="font-semibold">{asset.name}</h3>
                  <p className="text-sm text-muted-foreground">{asset.network}</p>
                </div>
              </div>
              {selectedAsset === asset.id && (
                <div className="rounded-full bg-primary/10 p-1">
                  <Check className="h-4 w-4 text-primary" />
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      ))}
      
      <div className="sm:col-span-2 mt-4">
        <p className="text-sm text-muted-foreground">
          Select your preferred payment network. Make sure to use the correct network
          when sending your payment to avoid transaction failures.
        </p>
      </div>
    </div>
  );
}
