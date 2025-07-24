import { redirect } from 'next/navigation';
import { loginAction } from '@/app/actions/auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle , Button , Input , Label } from '@/packages/ui';

interface LoginFormServerProps {
  redirectTo?: string;
  error?: string;
}

export async function LoginFormServer({ redirectTo, error }: LoginFormServerProps) {
  async function handleLogin(formData: FormData) {
    'use server';
    
    const email = formData.get('email') as string;
    const password = formData.get('password') as string;

    if (!email || !password) {
      redirect(`/login?error=${encodeURIComponent('Email and password are required')}`);
    }

    const result = await loginAction(email, password);
    
    if (!result.success) {
      redirect(`/login?error=${encodeURIComponent(result.error || 'Login failed')}`);
    }
    
    redirect(redirectTo || '/dashboard');
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>Sign In</CardTitle>
        <CardDescription>
          Enter your credentials to access your account
        </CardDescription>
      </CardHeader>
      <form action={handleLogin}>
        <CardContent className="space-y-4">
          {error && (
            <div className="rounded-md bg-red-50 p-4 border border-red-200">
              <div className="text-sm text-red-700">{error}</div>
            </div>
          )}
          
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              name="email"
              type="email"
              placeholder="Enter your email"
              required
            />
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              name="password"
              type="password"
              placeholder="Enter your password"
              required
            />
          </div>

          <div className="pt-4">
            <Button type="submit" className="w-full">
              Sign In
            </Button>
          </div>
        </CardContent>
      </form>
    </Card>
  );
}