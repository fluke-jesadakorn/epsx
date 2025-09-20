'use client'

import { useEffect, useState, ReactNode } from 'react'
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Lock, Crown, Zap, Shield, AlertCircle } from 'lucide-react'
import Link from 'next/link'

interface FeatureAccess {
  hasAccess: boolean
  permissionTemplate?: string
  featureKey: string
  context: string
  requiredPermissions?: string[]
  usageLimit?: number
  currentUsage?: number
}

interface FeatureGateProps {
  featureKey: string
  context: 'web_app' | 'api_access' | 'admin_interface'
  children: ReactNode
  fallback?: ReactNode
  showUpgrade?: boolean
  className?: string
}

export function FeatureGate({ 
  featureKey, 
  context, 
  children, 
  fallback, 
  showUpgrade = true,
  className = '' 
}: FeatureGateProps) {
  const [access, setAccess] = useState<FeatureAccess | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    checkFeatureAccess()
  }, [featureKey, context])

  const checkFeatureAccess = async () => {
    try {
      setLoading(true)
      const response = await fetch(`/api/v1/user/features/check`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          feature_key: featureKey,
          context: context
        })
      })

      if (response.ok) {
        const data = await response.json()
        setAccess(data)
      } else {
        // If API fails, assume no access for security
        setAccess({
          hasAccess: false,
          featureKey,
          context
        })
      }
    } catch (error) {
      console.error('Error checking feature access:', error)
      // Default to no access on error
      setAccess({
        hasAccess: false,
        featureKey,
        context
      })
    } finally {
      setLoading(false)
    }
  }

  const getUpgradeIcon = (requiredPermissions?: string[]) => {
    if (!requiredPermissions || requiredPermissions.length === 0) return Lock
    
    const hasEnterprise = requiredPermissions.some(p => p.includes('*:*') || p.includes('enterprise'))
    const hasPremium = requiredPermissions.some(p => p.includes('premium') || p.includes('pro'))
    
    if (hasEnterprise) return Crown
    if (hasPremium) return Zap
    return Shield
  }

  if (loading) {
    return (
      <div className={`animate-pulse bg-gray-200 dark:bg-gray-700 rounded-lg h-20 ${className}`} />
    )
  }

  if (!access) {
    return (
      <Alert variant="destructive" className={className}>
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          Unable to verify feature access. Please try again.
        </AlertDescription>
      </Alert>
    )
  }

  if (access.hasAccess) {
    // Check usage limits if applicable
    if (access.usageLimit && access.currentUsage !== undefined) {
      const usagePercentage = (access.currentUsage / access.usageLimit) * 100
      
      if (usagePercentage >= 100) {
        // Usage limit exceeded
        const UpgradeIcon = getUpgradeIcon(access.requiredPermissions)
        
        return (
          <Card className={`border-red-200 dark:border-red-800 ${className}`}>
            <CardContent className="p-6">
              <div className="flex items-center gap-4">
                <div className="p-3 bg-red-100 dark:bg-red-900/20 rounded-full">
                  <Lock className="h-6 w-6 text-red-600 dark:text-red-400" />
                </div>
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-red-900 dark:text-red-100">
                    Usage Limit Reached
                  </h3>
                  <p className="text-red-700 dark:text-red-300">
                    You've used {access.currentUsage} of {access.usageLimit} for "{featureKey}". 
                    {showUpgrade && access.requiredPermissions && ' Upgrade your permissions to continue using this feature.'}
                  </p>
                </div>
              </div>
              {showUpgrade && access.requiredPermissions && (
                <div className="mt-4 flex gap-2">
                  <Link href="/templates">
                    <Button className="bg-gradient-to-r from-emerald-400 to-green-500 text-white">
                      <UpgradeIcon className="h-4 w-4 mr-2" />
                      Upgrade Access
                    </Button>
                  </Link>
                  <Link href="/settings">
                    <Button variant="outline">
                      View Usage
                    </Button>
                  </Link>
                </div>
              )}
            </CardContent>
          </Card>
        )
      } else if (usagePercentage >= 80) {
        // Usage warning - still show content but with warning
        return (
          <div className={className}>
            <Alert className="mb-4 border-yellow-200 bg-yellow-50 dark:border-yellow-800 dark:bg-yellow-900/20">
              <AlertCircle className="h-4 w-4 text-yellow-600" />
              <AlertDescription className="text-yellow-800 dark:text-yellow-200">
                You've used {access.currentUsage} of {access.usageLimit} for this feature. 
                Consider upgrading your plan to avoid interruption.
              </AlertDescription>
            </Alert>
            {children}
          </div>
        )
      }
    }
    
    // User has access and within limits
    return <div className={className}>{children}</div>
  }

  // User doesn't have access
  if (fallback) {
    return <div className={className}>{fallback}</div>
  }

  if (!showUpgrade) {
    return null
  }

  const UpgradeIcon = getUpgradeIcon(access.requiredPermissions)

  return (
    <Card className={`border-gray-200 dark:border-gray-700 ${className}`}>
      <CardHeader>
        <CardTitle className="flex items-center gap-3 text-gray-700 dark:text-gray-300">
          <div className="p-2 bg-gray-100 dark:bg-gray-800 rounded-full">
            <Lock className="h-5 w-5 text-gray-500" />
          </div>
          Permission Required
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <p className="text-gray-600 dark:text-gray-400">
            {access.requiredPermissions && access.requiredPermissions.length > 0 ? (
              <>This feature requires additional permissions: <strong>{access.requiredPermissions.join(', ')}</strong></>
            ) : (
              <>This feature requires additional permissions to access.</>
            )}
          </p>
          <div className="flex gap-3">
            <Link href="/templates">
              <Button className="bg-gradient-to-r from-emerald-400 to-green-500 text-white">
                <UpgradeIcon className="h-4 w-4 mr-2" />
                View Permission Templates
              </Button>
            </Link>
            <Link href="/settings">
              <Button variant="outline">
                <Shield className="h-4 w-4 mr-2" />
                Current Permissions
              </Button>
            </Link>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

// Hook for programmatic feature checking
export function useFeatureAccess(featureKey: string, context: 'web_app' | 'api_access' | 'admin_interface') {
  const [access, setAccess] = useState<FeatureAccess | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const checkAccess = async () => {
      try {
        setLoading(true)
        const response = await fetch(`/api/v1/user/features/check`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            feature_key: featureKey,
            context: context
          })
        })

        if (response.ok) {
          const data = await response.json()
          setAccess(data)
        } else {
          setAccess({
            hasAccess: false,
            featureKey,
            context
          })
        }
      } catch (error) {
        console.error('Error checking feature access:', error)
        setAccess({
          hasAccess: false,
          featureKey,
          context
        })
      } finally {
        setLoading(false)
      }
    }

    checkAccess()
  }, [featureKey, context])

  return { access, loading, refetch: () => window.location.reload() }
}