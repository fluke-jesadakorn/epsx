import { signIn } from '@/lib/auth';
import { Button } from '@epsx/ui';
import { ExternalLink } from 'lucide-react';

interface LoginFormProps {
  redirectTo?: string;
}

export async function LoginForm({ redirectTo = '/dashboard' }: LoginFormProps) {
  async function handleSignIn() {
    'use server';
    
    // Use NextAuth.js signIn with backend OIDC provider
    await signIn('epsx-backend', { 
      redirectTo,
      // Let NextAuth.js handle the full OIDC flow
    });
  }

  return (
    <div className="space-y-4">
      {/* Main login form using Server Action */}
      <form action={handleSignIn}>
        <Button
          type="submit"
          className="w-full h-12 text-lg font-semibold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-0 shadow-lg"
          size="lg"
        >
          <ExternalLink className="mr-2 h-5 w-5" />
          Continue with EPSX
        </Button>
      </form>

      {/* Security notice */}
      <div className="text-center">
        <p className="text-xs text-muted-foreground">
          🔒 Secure authentication powered by EPSX backend
        </p>
      </div>
    </div>
  );
}

export default LoginForm;