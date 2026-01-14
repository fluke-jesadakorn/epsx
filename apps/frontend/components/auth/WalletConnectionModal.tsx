'use client';

/**
 * WALLET CONNECTION MODAL
 * Lightweight wrapper around shared AuthModal for frontend use
 */

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { AuthModal } from '@/shared/components/auth';
import { ChevronDown, Wallet } from 'lucide-react';
import { useState } from 'react';

interface WalletConnectionModalProps {
  children?: React.ReactNode;
  className?: string;
}

export function WalletConnectionModal({ children, className }: WalletConnectionModalProps) {
  const [isOpen, setIsOpen] = useState(false);

  const handleSuccess = () => {
    setIsOpen(false);
    // Refresh to update server state
    if (typeof window !== 'undefined') {
      window.location.reload();
    }
  };

  return (
    <>
      {children ? (
        <div onClick={() => setIsOpen(true)} className={className}>
          {children}
        </div>
      ) : (
        <Button
          variant="outline"
          className={cn(
            "flex items-center gap-2 px-4 py-2",
            "bg-slate-800 hover:bg-slate-700 border-slate-600",
            "text-white rounded-lg",
            className
          )}
          onClick={() => setIsOpen(true)}
        >
          <Wallet className="h-4 w-4 text-orange-500" />
          <span className="font-medium">Connect Wallet</span>
          <ChevronDown className="h-4 w-4 text-slate-400" />
        </Button>
      )}

      <AuthModal
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        variant="user"
        onSuccess={handleSuccess}
      />
    </>
  );
}