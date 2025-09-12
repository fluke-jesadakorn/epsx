'use client';

import { LogOut } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { logger, devLog, safeError } from '@/lib/logger';

interface LogoutFormProps {
  className?: string;
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
}

export function LogoutForm({ className, variant = 'outline' }: LogoutFormProps) {
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const router = useRouter();

  const handleLogout = async () => {
    try {
      setIsLoggingOut(true);
      
      // Call the logout API endpoint
      const response = await fetch('/api/auth/logout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (response.ok) {
        // Clear any client-side auth state and redirect to home
        router.push('/');
        router.refresh();
      } else {
        logger.error('Logout failed', { status: response.status, statusText: response.statusText });
        // Still redirect even if logout API fails
        router.push('/');
        router.refresh();
      }
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