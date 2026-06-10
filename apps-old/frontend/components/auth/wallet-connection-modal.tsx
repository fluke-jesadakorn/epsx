'use client';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';
import { ChevronDown, Wallet } from 'lucide-react';

interface WalletConnectionModalProps {
  children?: React.ReactNode;
  className?: string;
}

export function WalletConnectionModal({ children, className }: WalletConnectionModalProps) {
  const { openSignInModal } = useSharedAuth();

  if (children) {
    return (
      <div
        onClick={openSignInModal}
        className={cn("cursor-pointer", className)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            openSignInModal();
          }
        }}
      >
        {children}
      </div>
    );
  }

  return (
    <Button
      variant="outline"
      className={cn(
        "flex items-center gap-2 px-6 py-3 sm:px-4 sm:py-2",
        "bg-gradient-to-r from-orange-500 to-purple-600 hover:from-orange-600 hover:to-purple-700",
        "text-white rounded-xl sm:rounded-lg border-0 transition-all",
        "shadow-lg hover:shadow-xl active:scale-[0.98]",
        "text-base sm:text-sm font-bold sm:font-medium",
        "min-h-[48px] sm:min-h-0",
        className
      )}
      onClick={openSignInModal}
    >
      <Wallet className="h-5 w-5 sm:h-4 sm:w-4" />
      <span>Connect Wallet</span>
      <ChevronDown className="h-5 w-5 sm:h-4 sm:w-4" />
    </Button>
  );
}
