'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { useAuth } from '@/lib/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Loader2 } from 'lucide-react'
import { getBackendUrl } from '../../../shared/utils/url-resolver'

export default function Login() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const { login, isLoading } = useAuth()
  const router = useRouter()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (!email || !password) {
      setError('Email and password are required')
      return
    }

    try {
      // POST directly to our backend OIDC endpoint
      const backendUrl = getBackendUrl('client');
      if (!backendUrl) {
        throw new Error('NEXT_PUBLIC_BACKEND_URL environment variable is required');
      }
      const response = await fetch(`${backendUrl}/oauth/login-post`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          email,
          password,
        }),
        credentials: 'include',
      });

      if (response.ok) {
        const result = await response.json();
        console.log('✅ Admin login successful:', result);
        
        // Store access token in session API for proper JWT handling
        await fetch('/api/auth/session', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ accessToken: result.access_token }),
          credentials: 'include'
        });
        
        // Redirect to admin dashboard
        router.push('/');
      } else {
        setError('Invalid admin credentials. Please try again.');
      }
    } catch (err) {
      console.error('❌ Admin login failed:', err);
      setError('Login failed. Please check your connection and try again.');
    }
  }

  const handleOIDCLogin = async () => {
    try {
      // Redirect to backend OIDC login form for admin
      const backendUrl = getBackendUrl('client');
      if (!backendUrl) {
        throw new Error('NEXT_PUBLIC_BACKEND_URL environment variable is required');
      }
      window.location.href = `${backendUrl}/oauth/login`;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'OIDC login failed')
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl font-bold">Admin Login</CardTitle>
          <CardDescription>
            Sign in to access the admin dashboard
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="admin@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                disabled={isLoading}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                placeholder="Enter your password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={isLoading}
                required
              />
            </div>

            <Button 
              type="submit" 
              className="w-full" 
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Signing in...
                </>
              ) : (
                'Sign In'
              )}
            </Button>
          </form>

          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <span className="w-full border-t" />
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-background px-2 text-muted-foreground">
                Or continue with
              </span>
            </div>
          </div>

          <Button
            variant="outline"
            onClick={handleOIDCLogin}
            className="w-full"
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Connecting...
              </>
            ) : (
              'EPSX Backend Login'
            )}
          </Button>

          <p className="text-center text-sm text-muted-foreground">
            Use your EPSX admin credentials to access the dashboard
          </p>
        </CardContent>
      </Card>
    </div>
  )
}

// Simplified login button for existing layouts
export function LoginButton({ 
  children = 'Login',
  className = '',
  variant = 'default' as const
}: {
  children?: React.ReactNode
  className?: string
  variant?: 'default' | 'outline' | 'ghost'
}) {
  const { login, isLoading } = useAuth()

  const handleClick = async () => {
    try {
      // Redirect to backend OIDC login form
      const backendUrl = getBackendUrl('client');
      if (!backendUrl) {
        throw new Error('NEXT_PUBLIC_BACKEND_URL environment variable is required');
      }
      window.location.href = `${backendUrl}/oauth/login`;
    } catch (error) {
      console.error('Login failed:', error)
    }
  }

  return (
    <Button
      onClick={handleClick}
      disabled={isLoading}
      className={className}
      variant={variant}
    >
      {isLoading ? (
        <>
          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          Connecting...
        </>
      ) : (
        children
      )}
    </Button>
  )
}

// Logout button component
export function LogoutButton({
  children = 'Logout',
  className = '',
  variant = 'ghost' as const
}: {
  children?: React.ReactNode
  className?: string
  variant?: 'default' | 'outline' | 'ghost'
}) {
  const { logout, isLoading } = useAuth()

  const handleClick = async () => {
    try {
      await logout()
    } catch (error) {
      console.error('Logout failed:', error)
    }
  }

  return (
    <Button
      onClick={handleClick}
      disabled={isLoading}
      className={className}
      variant={variant}
    >
      {isLoading ? (
        <>
          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          Signing out...
        </>
      ) : (
        children
      )}
    </Button>
  )
}