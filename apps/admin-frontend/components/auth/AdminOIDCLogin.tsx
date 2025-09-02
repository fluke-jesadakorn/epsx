'use client'

/**
 * Admin OIDC Login Component
 * Integrates Firebase authentication with OIDC token exchange for admin access
 * Validates admin permissions before granting access
 */

import { useState, useTransition } from 'react'
import { useRouter } from 'next/navigation'
import { 
  signInWithGoogle, 
  signInWithEmail, 
  createAccount, 
  completeFirebaseOIDCFlow 
} from '@/lib/firebase-admin' // Admin-specific Firebase client
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { _Card, _CardContent, _CardDescription, _CardHeader, _CardTitle } from '@/components/ui/card'
import { useToast } from '@/components/ui/toast'
import { AlertTriangle, Shield, Users, Loader2, CheckCircle } from 'lucide-react'

interface AdminOIDCLoginProps {
  redirectTo?: string
}

/**
 * Complete admin authentication flow:
 * Firebase Auth → Firebase ID Token → Backend OIDC Exchange → Admin Permission Check → OIDC Cookies
 */
export function AdminOIDCLogin({ redirectTo = '/dashboard' }: AdminOIDCLoginProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [loginStep, setLoginStep] = useState<'firebase' | 'oidc' | 'permissions' | 'complete'>('firebase')
  const [isPending, startTransition] = useTransition()
  
  const router = useRouter()
  const { addToast } = useToast()

  /**
   * Complete admin OIDC authentication flow
   */
  const completeAdminAuthentication = async (firebaseIdToken: string) => {
    try {
      setLoginStep('oidc')
      
      // Step 1: Exchange Firebase ID token for OIDC tokens
      console.log('🔄 Starting admin Firebase → OIDC token exchange...')
      
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
      const response = await fetch(`${backendUrl}/api/v1/oidc/token/exchange`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          firebase_id_token: firebaseIdToken,
          grant_type: 'firebase_token',
          scope: 'openid profile email admin:*:*' // Request admin scope
        })
      })
      
      if (!response.ok) {
        const errorData = await response.text()
        throw new Error(`OIDC token exchange failed: ${response.status} - ${errorData}`)
      }
      
      const oidcTokens = await response.json()
      console.log('✅ OIDC tokens received successfully')
      
      setLoginStep('permissions')
      
      // Step 2: Validate admin permissions before storing cookies
      console.log('🔄 Validating admin permissions...')
      
      const userInfoResponse = await fetch(`${backendUrl}/oauth/userinfo`, {
        headers: {
          'Authorization': `Bearer ${oidcTokens.access_token}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!userInfoResponse.ok) {
        throw new Error('Failed to validate user permissions')
      }
      
      const userInfo = await userInfoResponse.json()
      
      // Check for admin permissions
      const hasAdminAccess = userInfo.permissions?.some((p: string) => 
        p === 'admin:*:*' || p.startsWith('admin:')
      ) || false
      
      if (!hasAdminAccess) {
        throw new Error(
          'Insufficient permissions for admin access. Required: admin:* permissions. ' +
          `Current permissions: ${userInfo.permissions?.join(', ') || 'none'}`
        )
      }
      
      console.log('✅ Admin permissions validated:', {
        user: userInfo.email,
        adminPermissions: userInfo.permissions.filter((p: string) => p.startsWith('admin:'))
      })
      
      // Step 3: Store OIDC tokens in HttpOnly cookies
      console.log('🔄 Storing admin OIDC tokens in cookies...')
      
      const sessionResponse = await fetch('/api/auth/session', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          accessToken: oidcTokens.access_token,
          idToken: oidcTokens.id_token,
          refreshToken: oidcTokens.refresh_token
        }),
        credentials: 'include'
      })
      
      if (!sessionResponse.ok) {
        const errorData = await sessionResponse.json()
        throw new Error(`Failed to store admin session: ${errorData.error}`)
      }
      
      setLoginStep('complete')
      
      console.log('✅ Admin OIDC authentication flow completed successfully')
      
      addToast({
        type: 'success',
        title: 'Admin Login Successful',
        description: `Welcome back, ${userInfo.email}! Admin access granted.`
      })
      
      // Navigate to admin dashboard
      startTransition(() => {
        router.push(redirectTo)
      })
      
    } catch (error: any) {
      console.error('❌ Admin OIDC authentication failed:', error)
      
      addToast({
        type: 'error',
        title: 'Admin Login Failed', 
        description: error.message || 'Failed to complete admin authentication'
      })
      
      setLoginStep('firebase')
    }
  }

  /**
   * Handle Google Sign-In for admin access
   */
  const handleGoogleSignIn = async () => {
    if (isLoading) return
    
    setIsLoading(true)
    
    try {
      console.log('🔄 Starting admin Google authentication...')
      
      const { user, idToken } = await signInWithGoogle()
      
      console.log('✅ Firebase Google authentication successful:', {
        uid: user.uid,
        email: user.email,
        displayName: user.displayName
      })
      
      // Complete admin OIDC flow
      await completeAdminAuthentication(idToken)
      
    } catch (error: any) {
      console.error('❌ Admin Google sign-in failed:', error)
      
      addToast({
        type: 'error',
        title: 'Google Sign-In Failed',
        description: error.message || 'Failed to authenticate with Google'
      })
      
      setLoginStep('firebase')
    } finally {
      setIsLoading(false)
    }
  }

  /**
   * Handle Email/Password Sign-In for admin access  
   */
  const handleEmailSignIn = async (e: React.FormEvent) => {
    e.preventDefault()
    if (isLoading || !email || !password) return
    
    setIsLoading(true)
    
    try {
      console.log('🔄 Starting admin email authentication...')
      
      const { user, idToken } = await signInWithEmail(email, password)
      
      console.log('✅ Firebase email authentication successful:', {
        uid: user.uid,
        email: user.email
      })
      
      // Complete admin OIDC flow
      await completeAdminAuthentication(idToken)
      
    } catch (error: any) {
      console.error('❌ Admin email sign-in failed:', error)
      
      addToast({
        type: 'error',
        title: 'Email Sign-In Failed',
        description: error.message || 'Invalid email or password'
      })
      
      setLoginStep('firebase')
    } finally {
      setIsLoading(false)
    }
  }

  const getStepIcon = () => {
    switch (loginStep) {
      case 'firebase': return <Shield className="h-5 w-5" />
      case 'oidc': return <Loader2 className="h-5 w-5 animate-spin" />
      case 'permissions': return <Users className="h-5 w-5" />
      case 'complete': return <CheckCircle className="h-5 w-5 text-green-600" />
    }
  }

  const getStepDescription = () => {
    switch (loginStep) {
      case 'firebase': return 'Authenticate with Firebase'
      case 'oidc': return 'Exchanging Firebase token for OIDC tokens...'
      case 'permissions': return 'Validating admin permissions...'
      case 'complete': return 'Authentication complete! Redirecting...'
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
      <div className="max-w-md w-full space-y-6">
        {/* Header */}
        <div className="text-center">
          <h1 className="text-3xl font-bold text-gray-900">Admin Login</h1>
          <p className="text-gray-600 mt-2">
            Sign in to access the EPSX admin dashboard
          </p>
        </div>

        {/* Authentication Flow Status */}
        <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <div className="flex items-center gap-3">
            {getStepIcon()}
            <div>
              <h3 className="font-medium text-blue-900">Authentication Status</h3>
              <p className="text-sm text-blue-700">{getStepDescription()}</p>
            </div>
          </div>
          
          {loginStep !== 'firebase' && (
            <div className="mt-3 text-xs text-blue-600">
              <p>✅ Firebase Authentication</p>
              {loginStep !== 'oidc' && <p>✅ OIDC Token Exchange</p>}
              {loginStep === 'complete' && <p>✅ Admin Permissions Validated</p>}
            </div>
          )}
        </div>

        {/* Login Form */}
        <_Card>
          <_CardHeader>
            <_CardTitle>Admin Access Required</_CardTitle>
            <_CardDescription>
              You need admin permissions to access this interface
            </_CardDescription>
          </_CardHeader>
          <_CardContent className="space-y-4">
            {/* Google Sign-In */}
            <Button
              onClick={handleGoogleSignIn}
              disabled={isLoading || isPending}
              className="w-full"
              variant="outline"
            >
              {isLoading && loginStep === 'firebase' ? (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              ) : (
                <svg className="h-4 w-4 mr-2" viewBox="0 0 24 24">
                  <path fill="currentColor" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                  <path fill="currentColor" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                  <path fill="currentColor" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                  <path fill="currentColor" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                </svg>
              )}
              Continue with Google
            </Button>

            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <span className="w-full border-t" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-white px-2 text-gray-500">Or continue with email</span>
              </div>
            </div>

            {/* Email Sign-In Form */}
            <form onSubmit={handleEmailSignIn} className="space-y-4">
              <div>
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="admin@example.com"
                  required
                  disabled={isLoading || isPending}
                />
              </div>
              
              <div>
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter your password"
                  required
                  disabled={isLoading || isPending}
                />
              </div>
              
              <Button 
                type="submit" 
                className="w-full"
                disabled={isLoading || isPending || !email || !password}
              >
                {isLoading && loginStep === 'firebase' ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Signing In...
                  </>
                ) : (
                  'Sign In'
                )}
              </Button>
            </form>
          </_CardContent>
        </_Card>

        {/* Admin Requirements Notice */}
        <div className="p-4 bg-amber-50 border border-amber-200 rounded-lg">
          <div className="flex gap-3">
            <AlertTriangle className="h-5 w-5 text-amber-600 flex-shrink-0 mt-0.5" />
            <div className="text-sm">
              <h4 className="font-medium text-amber-800">Admin Access Required</h4>
              <p className="text-amber-700 mt-1">
                You must have <code className="bg-amber-100 px-1 rounded">admin:*</code> permissions 
                to access this dashboard. Contact your system administrator if you need access.
              </p>
            </div>
          </div>
        </div>

        {/* OIDC Migration Notice */}
        <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
          <div className="flex items-center gap-2 text-green-800">
            <CheckCircle className="h-4 w-4" />
            <span className="font-medium text-sm">OIDC Authentication Enabled</span>
          </div>
          <p className="text-xs text-green-600 mt-1">
            Using secure OIDC-compliant authentication with structured permissions
          </p>
        </div>
      </div>
    </div>
  )
}

export default AdminOIDCLogin