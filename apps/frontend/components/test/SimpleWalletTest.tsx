'use client';

import { ConnectButton } from '@rainbow-me/rainbowkit';

export function SimpleWalletTest() {
  return (
    <div className="fixed top-4 right-4 p-4 bg-slate-900 text-white rounded-lg z-50">
      <div className="text-sm mb-2">Simple Wallet Test:</div>
      <ConnectButton />
    </div>
  );
}