'use client';

import { signIn } from '@/lib/auth';
import { ExternalLink } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface LoginFormProps {
  redirectTo?: string;
}

export function LoginForm({ redirectTo = '/dashboard' }: LoginFormProps) {
  function handleSignIn() {
    // Use custom OAuth 2.0 sign-in flow
    signIn();
  }

  return (
    <div className="space-y-4">
      {/* Main login form using custom OAuth 2.0 flow */}
      <div>
        <Button
          onClick={handleSignIn}
          type="button"
          className="w-full h-12 text-lg font-semibold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-0 shadow-lg"
          size="lg"
        >
          <ExternalLink className="mr-2 h-5 w-5" />
          Continue with EPSX
        </Button>
      </div>

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