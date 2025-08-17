import { handleSignOut } from '@/lib/actions/auth';
import { LogOut } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface LogoutFormProps {
  className?: string;
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
}

export function LogoutForm({ className, variant = 'outline' }: LogoutFormProps) {
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