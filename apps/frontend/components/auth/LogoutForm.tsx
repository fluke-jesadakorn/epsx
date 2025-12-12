'use client';

import { LogOut } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useWeb3Auth } from '@/lib/auth/use-auth';

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
      
      // Use Web3 disconnect for all users (Web3-first approach)
      await disconnect();
      
      // Redirect to home
      router.push('/');
      router.refresh();
    } catch (error) {
      console.error('Logout error', error);
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