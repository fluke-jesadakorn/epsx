'use client'

import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Crown, Zap, Shield, TrendingUp, CheckCircle, XCircle, AlertCircle } from 'lucide-react'

interface PlanFeature {
  id: number
  context_name: string
  feature_key: string
  feature_config: Record<string, any>
  resource_cost: number
  is_active: boolean
}

interface UserSubscription {
  id: string
  plan_id: number
  plan_name: string
  access_context: string
  status: string
  current_usage: Record<string, any>
  quota_limits: Record<string, any>
  started_at: string
  expires_at?: string
  auto_renew: boolean
  features: PlanFeature[]
}

interface UserPlanDisplayProps {
  userId: string
}

export function UserPlanDisplay({ userId }: UserPlanDisplayProps) {
  const [subscriptions, setSubscriptions] = useState<UserSubscription[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadUserSubscriptions()
  }, [userId])

  const loadUserSubscriptions = async () => {
    try {
      setLoading(true)
      // This would be a call to your backend API to get user's subscriptions
      const response = await fetch(`/api/v1/user/subscriptions`)
      
      if (response.ok) {
        const data = await response.json()
        setSubscriptions(data.subscriptions || [])
      } else if (response.status === 404) {
        // No subscriptions found - user might be on free tier
        setSubscriptions([])
      } else {
        throw new Error('Failed to load subscription data')
      }
    } catch (error) {
      console.error('Error loading subscriptions:', error)
      setError('Unable to load subscription information')
    } finally {
      setLoading(false)
    }
  }

  const calculateUsagePercentage = (used: number, limit: number) => {
    if (limit === 0) return 0
    return Math.min((used / limit) * 100, 100)
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-600 dark:bg-green-900/20 dark:text-green-400'
      case 'expired':
        return 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/20 dark:text-yellow-400'
      case 'cancelled':
        return 'bg-red-100 text-red-600 dark:bg-red-900/20 dark:text-red-400'
      default:
        return 'bg-gray-100 text-gray-600 dark:bg-gray-900/20 dark:text-gray-400'
    }
  }

  const getPlanIcon = (planName: string) => {
    if (planName.toLowerCase().includes('enterprise')) return Crown
    if (planName.toLowerCase().includes('pro') || planName.toLowerCase().includes('premium')) return Zap
    if (planName.toLowerCase().includes('api')) return Shield
    return TrendingUp
  }

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Current Plan
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="h-4 bg-gray-200 rounded animate-pulse"></div>
            <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4"></div>
            <div className="h-4 bg-gray-200 rounded animate-pulse w-1/2"></div>
          </div>
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertCircle className="h-5 w-5" />
            Plan Information
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    )
  }

  if (subscriptions.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Get Started with EPSX
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center space-y-4">
            <div className="text-6xl mb-4">🚀</div>
            <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400">
              No Active Plans
            </h3>
            <p className="text-gray-500 dark:text-gray-500 mb-6">
              Choose a plan to unlock powerful analytics features and API access
            </p>
            <Button className="bg-gradient-to-r from-emerald-400 to-green-500 text-white">
              View Available Plans
            </Button>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-6">
      {subscriptions.map(subscription => {
        const PlanIcon = getPlanIcon(subscription.plan_name)
        const isActive = subscription.status === 'active'
        
        return (
          <Card key={subscription.id}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-3">
                  <PlanIcon className="h-6 w-6 text-emerald-600" />
                  {subscription.plan_name}
                </CardTitle>
                <Badge className={getStatusColor(subscription.status)}>
                  {subscription.status.toUpperCase()}
                </Badge>
              </div>
              <div className="flex items-center gap-4 text-sm text-gray-500">
                <span>Access: {subscription.access_context}</span>
                {subscription.expires_at && (
                  <span>
                    Expires: {new Date(subscription.expires_at).toLocaleDateString()}
                  </span>
                )}
                <span>
                  Auto-renew: {subscription.auto_renew ? 'Yes' : 'No'}
                </span>
              </div>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Usage Statistics */}
              {isActive && Object.keys(subscription.current_usage).length > 0 && (
                <div>
                  <h4 className="font-semibold text-gray-900 dark:text-white mb-4">Current Usage</h4>
                  <div className="space-y-4">
                    {Object.entries(subscription.current_usage).map(([key, used]) => {
                      const limit = subscription.quota_limits[key] || 0
                      const percentage = calculateUsagePercentage(Number(used), Number(limit))
                      const isOverLimit = percentage >= 100
                      
                      return (
                        <div key={key} className="space-y-2">
                          <div className="flex justify-between items-center text-sm">
                            <span className="font-medium capitalize">
                              {key.replace(/_/g, ' ')}
                            </span>
                            <span className={`font-semibold ${isOverLimit ? 'text-red-600' : 'text-gray-600'}`}>
                              {used} / {limit === 0 ? '∞' : limit}
                            </span>
                          </div>
                          <Progress 
                            value={percentage} 
                            className={`h-2 ${isOverLimit ? 'bg-red-100' : 'bg-gray-100'}`}
                          />
                          {isOverLimit && (
                            <div className="text-xs text-red-600 flex items-center gap-1">
                              <AlertCircle className="h-3 w-3" />
                              Usage limit exceeded
                            </div>
                          )}
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}

              {/* Plan Features */}
              <div>
                <h4 className="font-semibold text-gray-900 dark:text-white mb-4">Plan Features</h4>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {subscription.features.map(feature => (
                    <div key={feature.id} className="flex items-start gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                      {feature.is_active ? (
                        <CheckCircle className="h-5 w-5 text-green-500 mt-0.5 flex-shrink-0" />
                      ) : (
                        <XCircle className="h-5 w-5 text-gray-400 mt-0.5 flex-shrink-0" />
                      )}
                      <div className="min-w-0 flex-1">
                        <div className="font-medium text-sm">
                          {feature.feature_key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
                        </div>
                        <div className="text-xs text-gray-500 mt-1">
                          <Badge variant="outline" className="text-xs">
                            {feature.context_name}
                          </Badge>
                          {feature.resource_cost > 0 && (
                            <span className="ml-2">Cost: {feature.resource_cost}</span>
                          )}
                        </div>
                        {Object.keys(feature.feature_config).length > 0 && (
                          <div className="text-xs text-gray-400 mt-1">
                            {Object.entries(feature.feature_config).map(([key, value]) => (
                              <span key={key} className="mr-2">
                                {key}: {String(value)}
                              </span>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Action Buttons */}
              <div className="flex gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                {isActive ? (
                  <>
                    <Button variant="outline" className="flex-1">
                      Manage Plan
                    </Button>
                    <Button variant="outline">
                      View Usage Details
                    </Button>
                  </>
                ) : (
                  <>
                    <Button className="flex-1 bg-gradient-to-r from-emerald-400 to-green-500 text-white">
                      Renew Plan
                    </Button>
                    <Button variant="outline">
                      View History
                    </Button>
                  </>
                )}
              </div>

              {/* Expiry Warning */}
              {isActive && subscription.expires_at && (
                (() => {
                  const daysUntilExpiry = Math.ceil(
                    (new Date(subscription.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)
                  )
                  
                  if (daysUntilExpiry <= 7 && daysUntilExpiry > 0) {
                    return (
                      <Alert>
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription>
                          Your plan expires in {daysUntilExpiry} day{daysUntilExpiry !== 1 ? 's' : ''}. 
                          {!subscription.auto_renew && ' Consider enabling auto-renewal.'}
                        </AlertDescription>
                      </Alert>
                    )
                  }
                  
                  if (daysUntilExpiry <= 0) {
                    return (
                      <Alert variant="destructive">
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription>
                          Your plan has expired. Renew now to continue using premium features.
                        </AlertDescription>
                      </Alert>
                    )
                  }
                  
                  return null
                })()
              )}
            </CardContent>
          </Card>
        )
      })}
    </div>
  )
}