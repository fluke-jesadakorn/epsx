import { signOut } from '@/lib/auth';
import { Button } from '@epsx/ui';
import { LogOut } from 'lucide-react';

interface LogoutFormProps {
  className?: string;
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
}

export function LogoutForm({ className, variant = 'outline' }: LogoutFormProps) {
  async function handleSignOut() {
    'use server';
    
    // Use NextAuth.js signOut with proper cleanup
    await signOut({
      redirectTo: '/login'
    });
  }

  return (
    <form action={handleSignOut}>
      <Button
        type="submit"
        variant={variant}
        className={className}
        size="sm"
      >
        <LogOut className="mr-2 h-4 w-4" />
        Sign Out
      </Button>
    </form>
  );
}

export default LogoutForm;