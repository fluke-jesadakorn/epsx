'use client'

import { useState, useEffect } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Loader2, AlertTriangle, Shield } from 'lucide-react'

export default function AdminLoginPage() {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')
  const router = useRouter()
  const searchParams = useSearchParams()
  
  useEffect(() => {
    // Check for error parameters
    const errorParam = searchParams.get('error')
    const fresh = searchParams.get('fresh')
    
    if (errorParam) {
      switch (errorParam) {
        case 'state_mismatch':
          setError('OAuth state mismatch detected. Please start the login process again.')
          break
        case 'oidc_callback_error':
          setError('Authentication failed. Please try again.')
          break
        case 'missing_parameters':
          setError('Invalid authentication request. Please try again.')
          break
        default:
          setError('Authentication error. Please try again.')
      }
    }
    
    // If fresh=true, clear any existing error after showing it briefly
    if (fresh === 'true') {
      setTimeout(() => {
        // Clear URL parameters
        router.replace('/login', undefined)
      }, 2000)
    }
  }, [searchParams, router])

  const handleOAuthLogin = async () => {
    setIsLoading(true)
    setError('')
    
    try {
      console.log('🔄 Starting admin OAuth flow...')
      
      // Get the redirect URL from search params
      const redirectTo = searchParams.get('redirectTo') || '/'
      
      // Initiate OAuth flow via API
      const response = await fetch('/api/auth/initiate-oauth', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          redirectTo: redirectTo
        }),
        credentials: 'include'
      })
      
      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || 'Failed to initiate OAuth flow')
      }
      
      const { authorizationUrl, debug } = await response.json()
      
      console.log('✅ OAuth flow initiated, redirecting to authorization URL...')
      
      // Store PKCE parameters in localStorage as backup for development
      if (debug && typeof window !== 'undefined') {
        localStorage.setItem('admin_oauth_verifier_backup', debug.codeVerifier)
        localStorage.setItem('admin_oauth_state_backup', debug.state)
        localStorage.setItem('admin_oauth_callback_backup', debug.callbackUrl)
        console.log('💾 PKCE parameters backed up to localStorage for development')
      }
      
      // Redirect to the OAuth authorization endpoint
      window.location.href = authorizationUrl
      
    } catch (error: any) {
      console.error('❌ Failed to start OAuth flow:', error)
      setError('Failed to start authentication. Please try again.')
      setIsLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
      <div className="max-w-md w-full space-y-6">
        {/* Header */}
        <div className="text-center">
          <Shield className="mx-auto h-12 w-12 text-blue-600" />
          <h1 className="text-3xl font-bold text-gray-900 mt-4">Admin Login</h1>
          <p className="text-gray-600 mt-2">
            Sign in to access the EPSX admin dashboard
          </p>
        </div>

        {/* Error Alert */}
        {error && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Login Card */}
        <Card>
          <CardHeader className="text-center">
            <CardTitle>Admin Access Required</CardTitle>
            <CardDescription>
              You need admin permissions to access this interface
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Button
              onClick={handleOAuthLogin}
              disabled={isLoading}
              className="w-full"
              size="lg"
            >
              {isLoading ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Connecting to OAuth...
                </>
              ) : (
                <>
                  <Shield className="h-4 w-4 mr-2" />
                  Sign In with EPSX OAuth
                </>
              )}
            </Button>
            
            <div className="text-center">
              <p className="text-sm text-gray-600">
                Use your EPSX credentials to authenticate
              </p>
            </div>
          </CardContent>
        </Card>

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

        {/* OAuth Flow Info */}
        <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <div className="flex items-center gap-2 text-blue-800">
            <Shield className="h-4 w-4" />
            <span className="font-medium text-sm">Secure OAuth Authentication</span>
          </div>
          <p className="text-xs text-blue-600 mt-1">
            Using PKCE-secured OAuth flow with admin permission validation
          </p>
        </div>
      </div>
    </div>
  )
}