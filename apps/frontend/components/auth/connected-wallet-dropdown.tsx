'use client';

import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';
import { getExplorerAddressLink } from '@/shared/config/constants';
import { formatAddress } from '@/shared/auth/utils';
import { copyToClipboard as copyToClipboardUtil } from '@/utils/clipboard';
import { Check, Copy, ExternalLink, LogOut, Wallet } from 'lucide-react';
import { useState } from 'react';
import { useAccount, useDisconnect } from 'wagmi';

interface ConnectedWalletDropdownProps {
  className?: string;
}

export function ConnectedWalletDropdown({ className }: ConnectedWalletDropdownProps) {
  const { address } = useAccount();
  const { disconnect } = useDisconnect();
  const { user, logout } = useSharedAuth();
  const [copied, setCopied] = useState(false);

  const displayAddress = user?.wallet_address ?? address;

  const copyToClipboard = async (text: string) => {
    const success = await copyToClipboardUtil(text);
    if (success) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const openBSCScan = () => {
    if (displayAddress) {
      window.open(getExplorerAddressLink(displayAddress), '_blank');
    }
  };

  if (!displayAddress) {return null;}

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          className={cn(
            "flex items-center gap-2 px-4 py-2 h-auto",
            "bg-gray-100 dark:bg-slate-800 hover:bg-slate-700 border-slate-600",
            "text-white rounded-full",
            className
          )}
        >
          <Wallet className="h-4 w-4 text-orange-500" />
          <span className="font-medium">{formatAddress(displayAddress)}</span>
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent
        className={cn(
          "w-80 p-0 bg-gray-100 dark:bg-slate-800 border-slate-600",
          "rounded-2xl shadow-xl"
        )}
        align="end"
        sideOffset={8}
      >
        {/* Wallet Status Section */}
        <div className="px-4 py-4 border-b border-slate-700">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-orange-500/10">
              <Wallet className="h-5 w-5 text-orange-500" />
            </div>
            <div>
              <div className="text-white font-medium">Browser Wallet</div>
              <div className="text-slate-400 text-sm">Connected</div>
            </div>
          </div>
        </div>

        {/* Copy Address Section */}
        <DropdownMenuItem
          className={cn(
            "flex items-center gap-3 px-4 py-4 cursor-pointer",
            "hover:bg-slate-700 focus:bg-slate-700",
            "border-b border-slate-700"
          )}
          onClick={() => copyToClipboard(displayAddress)}
        >
          <div className="p-2 rounded-lg bg-orange-500/10">
            {copied ? (
              <Check className="h-4 w-4 text-orange-500" />
            ) : (
              <Copy className="h-4 w-4 text-orange-500" />
            )}
          </div>
          <div className="flex-1">
            <div className="text-white font-medium">Copy Address</div>
            <div className="text-slate-400 text-sm font-mono">{formatAddress(displayAddress)}</div>
          </div>
        </DropdownMenuItem>

        {/* View on Explorer Section */}
        <DropdownMenuItem
          className={cn(
            "flex items-center gap-3 px-4 py-4 cursor-pointer",
            "hover:bg-slate-700 focus:bg-slate-700",
            "border-b border-slate-700"
          )}
          onClick={openBSCScan}
        >
          <div className="p-2 rounded-lg bg-orange-500/10">
            <ExternalLink className="h-4 w-4 text-orange-500" />
          </div>
          <div className="flex-1">
            <div className="text-white font-medium">View on Explorer</div>
            <div className="text-slate-400 text-sm">Open in BSCScan</div>
          </div>
        </DropdownMenuItem>

        {/* Disconnect Section */}
        <DropdownMenuItem
          className={cn(
            "flex items-center gap-3 px-4 py-4 cursor-pointer",
            "hover:bg-red-500/10 focus:bg-red-500/10"
          )}
          onClick={() => {
            void logout().then(() => {
              disconnect();
            }).catch(() => {
              disconnect();
            });
          }}
        >
          <div className="p-2 rounded-lg bg-red-500/10">
            <LogOut className="h-4 w-4 text-red-500" />
          </div>
          <div className="flex-1">
            <div className="text-red-500 font-medium">Disconnect</div>
            <div className="text-red-400/70 text-sm">Disconnect your wallet</div>
          </div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}