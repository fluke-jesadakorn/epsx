'use client';

import { LogOut } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useState } from 'react';
import { useRouter } from 'next/navigation';

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
        console.error('Logout failed:', response.statusText);
        // Still redirect even if logout API fails
        router.push('/');
        router.refresh();
      }
    } catch (error) {
      console.error('Logout error:', error);
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
      <LogOut className="mr-2 h-4 w-4" />
      {isLoggingOut ? 'Signing Out...' : 'Sign Out'}
    </Button>
  );
}

export default LogoutForm;