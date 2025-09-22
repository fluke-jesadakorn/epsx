'use client';

import { LogOut } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useWeb3Auth } from '@/lib/auth/web3';
import { logger } from '@/lib/utils/logging';

interface LogoutFormProps {
  className?: string;
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
}

export function LogoutForm({ className, variant = 'outline' }: LogoutFormProps) {
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const router = useRouter();
  const { disconnect, isAuthenticated } = useWeb3Auth();

  const handleLogout = async () => {
    try {
      setIsLoggingOut(true);
      
      if (isAuthenticated) {
        // Use Web3 disconnect for authenticated Web3 users
        await disconnect();
      } else {
        // Clear any legacy session markers for non-Web3 users
        if (typeof window !== 'undefined') {
          try {
            window.localStorage.removeItem('oidc_session');
            document.cookie = 'oidc_session=; Max-Age=0; path=/; SameSite=Lax';
          } catch {}
        }
      }
      
      // Redirect to home
      router.push('/');
      router.refresh();
    } catch (error) {
      logger.error('Logout error', error);
      // Still redirect even if there's an error
      router.push('/');
      router.refresh();
    } finally {
      setIsLoggingOut(false);
    }
  };

  return (
    <Button
      onClick={handleLogout}
      disabled={isLoggingOut}
      variant={variant}
      className={className}
      size="sm"
    >
      <LogOut className="h-4 w-4 text-orange-500" />
      <span>{isLoggingOut ? 'Signing Out...' : 'Sign Out'}</span>
    </Button>
  );
}

export default LogoutForm;