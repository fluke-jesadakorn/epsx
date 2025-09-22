'use client';

import { useState } from 'react';
import { formatAddress } from '@/lib/auth/web3';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogClose,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Copy, ExternalLink, X } from 'lucide-react';
import { toast } from 'sonner';

interface WalletDisconnectModalProps {
  isOpen: boolean;
  onClose: () => void;
  onDisconnect: () => Promise<void>;
  walletAddress: string;
  balance?: string;
}

export function WalletDisconnectModal({
  isOpen,
  onClose,
  onDisconnect,
  walletAddress,
  balance = '0 ETH'
}: WalletDisconnectModalProps) {
  const [isDisconnecting, setIsDisconnecting] = useState(false);

  const handleCopyAddress = async () => {
    try {
      await navigator.clipboard.writeText(walletAddress);
      toast.success('Address copied to clipboard');
    } catch (error) {
      toast.error('Failed to copy address');
    }
  };

  const handleDisconnect = async () => {
    try {
      setIsDisconnecting(true);
      await onDisconnect();
      onClose();
    } catch (error) {
      console.error('Disconnect error:', error);
    } finally {
      setIsDisconnecting(false);
    }
  };

  const handleViewOnExplorer = () => {
    const isMainnet = process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK === 'mainnet';
    const explorerUrl = isMainnet 
      ? `https://bscscan.com/address/${walletAddress}`
      : `https://testnet.bscscan.com/address/${walletAddress}`;
    window.open(explorerUrl, '_blank');
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader className="relative">
          <div className="flex items-center gap-2 text-orange-500">
            <div className="w-8 h-8 bg-gradient-to-r from-orange-500 to-purple-600 rounded-full flex items-center justify-center">
              <svg 
                width="16" 
                height="16" 
                viewBox="0 0 24 24" 
                fill="currentColor"
                className="text-white"
              >
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
              </svg>
            </div>
            <DialogTitle className="text-xl font-bold">
              🌐 Web3 Admin
            </DialogTitle>
          </div>
          <DialogClose className="absolute right-0 top-0">
            <X className="h-4 w-4" />
            <span className="sr-only">Close</span>
          </DialogClose>
        </DialogHeader>

        <div className="space-y-6">
          {/* Wallet Info Section */}
          <div className="flex flex-col items-center space-y-4 p-6 bg-gradient-to-br from-orange-50 to-purple-50 dark:from-orange-900/10 dark:to-purple-900/10 rounded-lg">
            {/* Avatar */}
            <div className="w-16 h-16 bg-gradient-to-r from-red-400 to-orange-500 rounded-full flex items-center justify-center">
              <span className="text-2xl">👨‍💼</span>
            </div>

            {/* Address */}
            <div className="text-center">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {formatAddress(walletAddress)}
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                {balance}
              </p>
            </div>

            {/* Action Buttons */}
            <div className="flex gap-3 w-full">
              <Button
                variant="outline"
                onClick={handleCopyAddress}
                className="flex-1 flex items-center gap-2"
              >
                <Copy className="h-4 w-4" />
                คัดลอกที่อยู่
              </Button>
              <Button
                variant="destructive"
                onClick={handleDisconnect}
                disabled={isDisconnecting}
                className="flex-1 flex items-center gap-2"
              >
                {isDisconnecting ? 'กำลังตัด...' : 'ตัดการเชื่อมต่อ'}
              </Button>
            </div>
          </div>

          {/* EPSX Admin Info */}
          <div className="text-center space-y-2">
            <p className="text-sm text-gray-600 dark:text-gray-400">
              <span className="font-medium text-gray-900 dark:text-gray-100">EPSX Admin</span>{' '}
              รายการจะปรากฏที่นี่...
            </p>
          </div>

          {/* Explorer Link */}
          <Button
            variant="ghost"
            onClick={handleViewOnExplorer}
            className="w-full flex items-center gap-2 text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
          >
            <span>ดูเพิ่มเติมบน</span>
            <span className="font-medium">explorer</span>
            <ExternalLink className="h-4 w-4" />
            <span className="text-gray-400">❓</span>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}