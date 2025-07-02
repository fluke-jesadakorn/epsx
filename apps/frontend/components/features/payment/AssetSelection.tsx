'use client';

import { Card } from '@/components/ui/card';

interface Asset {
  id: string;
  name: string;
  network: string;
  logo: string;
  description: string;
}

interface AssetSelectionProps {
  onSelect: (asset: string) => void;
  selectedAsset?: string;
}

const assets: Asset[] = [
  {
    id: 'USDT_TRC20',
    name: 'USDT',
    network: 'TRC20',
    logo: '/QRPayment/USDT_TRX.png',
    description: 'USDT on TRON Network (TRC20)',
  },
  {
    id: 'USDT_BEP20',
    name: 'USDT',
    network: 'BEP20',
    logo: '/QRPayment/USDT_BNB.png',
    description: 'USDT on BNB Smart Chain (BSC)',
  },
  {
    id: 'USDT_ERC20',
    name: 'USDT',
    network: 'ERC20',
    logo: '/QRPayment/USDT_ETH.png',
    description: 'USDT on Ethereum Network (ERC20)',
  },
  {
    id: 'USDT_ARB',
    name: 'USDT',
    network: 'Arbitrum',
    logo: '/QRPayment/USDT_ARB.png',
    description: 'USDT on Arbitrum Network',
  },
  {
    id: 'TON',
    name: 'TON',
    network: 'TON',
    logo: '/QRPayment/USDT_TON.png',
    description: 'TON Native Token',
  },
];

export default function AssetSelection({
  onSelect,
  selectedAsset,
}: AssetSelectionProps) {
  return (
    <div className="grid gap-4">
      {assets.map((asset) => (
        <Card
          key={asset.id}
          className={`p-4 cursor-pointer hover:border-primary transition-colors ${
            selectedAsset === asset.id ? 'border-primary' : ''
          }`}
          onClick={() => onSelect(asset.id)}
        >
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-full overflow-hidden bg-background flex items-center justify-center">
              <img
                src={asset.logo}
                alt={`${asset.name} logo`}
                className="w-8 h-8 object-contain"
              />
            </div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold">
                {asset.name}{' '}
                <span className="text-muted-foreground">({asset.network})</span>
              </h3>
              <p className="text-sm text-muted-foreground">
                {asset.description}
              </p>
            </div>
            <div className="flex items-center justify-center w-6 h-6 rounded-full border-2 border-muted">
              {selectedAsset === asset.id && (
                <div className="w-3 h-3 rounded-full bg-primary" />
              )}
            </div>
          </div>
        </Card>
      ))}
    </div>
  );
}
