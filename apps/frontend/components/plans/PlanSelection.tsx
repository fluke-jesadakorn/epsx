'use client'

import { Button, Card, CardContent, CardHeader, CardTitle } from '@/components/ui'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { API_ROUTES } from '@/shared/config/route-constants'
import { AlertCircle, CheckCircle, Crown, Shield, Star, TrendingUp, Zap } from 'lucide-react'
import Link from 'next/link'
import { useEffect, useState } from 'react'

interface PlanFeature {
  id: number
  context_name: string
  feature_key: string
  feature_config: Record<string, any>
  resource_cost: number
  is_active: boolean
}

interface Plan {
  id: number | string // Support both number and UUID string
  name: string
  description?: string
  plan_type: string
  current_price: number | string // Support both number and decimal string
  currency: string
  target_audience: string
  billing_model: string
  plan_category: string
  is_active: boolean
  features: PlanFeature[]
  permissions?: string[] // Add permissions support from backend
  subscriber_count: number
  revenue_last_30_days: number | string // Support both number and decimal string
}

interface UserSubscription {
  id: string
  plan_id: number | string // Support both number and UUID string
  plan_name: string
  status: string
}

interface PlanSelectionProps {
  currentUser: any
}

export function PlanSelection({ currentUser }: PlanSelectionProps) {
  const [plans, setPlans] = useState<Plan[]>([])
  const [userSubscriptions, setUserSubscriptions] = useState<UserSubscription[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedCategory, setSelectedCategory] = useState<'all' | 'standard' | 'api' | 'enterprise'>('all')

  useEffect(() => {
    loadPlans()
    if (currentUser) {
      loadUserSubscriptions()
    }
  }, [currentUser])

  const loadPlans = async () => {
    try {
      setLoading(true)
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
      const response = await fetch(`${backendUrl}${API_ROUTES.PUBLIC.PLANS}`)

      if (response.ok) {
        const data = await response.json()
        if (data.success && Array.isArray(data.data)) {
          // Data is already array of plans
          setPlans(data.data.filter((plan: Plan) => plan.is_active))
        } else {
          throw new Error('Invalid API response structure')
        }
      } else {
        throw new Error('Failed to load plans')
      }
    } catch (error) {
      console.error('Error loading plans:', error)
      setError('Failed to load subscription plans. Please try again later.')
      setPlans([])
    } finally {
      setLoading(false)
    }
  }

  const loadUserSubscriptions = async () => {
    try {
      const response = await fetch('/api/user/subscriptions')

      if (response.ok) {
        const data = await response.json()
        setUserSubscriptions(data.subscriptions || [])
      }
    } catch (error) {
      console.error('Error loading user subscriptions:', error)
    }
  }

  const getPlanIcon = (category: string) => {
    switch (category) {
      case 'enterprise':
        return Crown
      case 'api':
        return Shield
      case 'standard':
        return TrendingUp
      default:
        return Zap
    }
  }

  const getPlanColor = (category: string) => {
    switch (category) {
      case 'enterprise':
        return 'from-purple-400 to-pink-500'
      case 'api':
        return 'from-orange-400 to-red-500'
      case 'standard':
        return 'from-blue-400 to-indigo-500'
      default:
        return 'from-emerald-400 to-green-500'
    }
  }

  const isUserSubscribed = (planId: number | string) => {
    return userSubscriptions.some(sub => sub.plan_id === planId && sub.status === 'active')
  }

  const getUserCurrentPlan = (planId: number | string) => {
    return userSubscriptions.find(sub => sub.plan_id === planId && sub.status === 'active')
  }

  const handleSubscribe = async (plan: Plan) => {
    if (!currentUser) {
      alert('Please connect your wallet to subscribe to a plan');
      return
    }

    // This would integrate with your subscription creation API
    try {
      const response = await fetch('/api/user/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          plan_id: plan.id,
          access_context: 'internal' // Default for web users
        })
      })

      if (response.ok) {
        await loadUserSubscriptions()
        // Show success message or redirect
      } else {
        throw new Error('Failed to subscribe to plan')
      }
    } catch (error) {
      console.error('Subscription error:', error)
      setError('Failed to subscribe to plan. Please try again.')
    }
  }

  const filteredPlans = plans.filter(plan => {
    if (selectedCategory === 'all') return true
    return plan.plan_category === selectedCategory
  })

  const groupedFeatures = (features: PlanFeature[]) => {
    const groups = features.reduce((acc, feature) => {
      if (!acc[feature.context_name]) {
        acc[feature.context_name] = []
      }
      acc[feature.context_name].push(feature)
      return acc
    }, {} as Record<string, PlanFeature[]>)

    return groups
  }

  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {Array.from({ length: 6 }).map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader>
              <div className="h-6 bg-gray-200 rounded"></div>
              <div className="h-4 bg-gray-200 rounded w-3/4"></div>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="h-8 bg-gray-200 rounded"></div>
                <div className="space-y-2">
                  {Array.from({ length: 5 }).map((_, j) => (
                    <div key={j} className="h-4 bg-gray-200 rounded"></div>
                  ))}
                </div>
                <div className="h-10 bg-gray-200 rounded"></div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    )
  }

  return (
    <div className="space-y-8">
      {/* Plan Category Filter */}
      <div className="flex flex-wrap justify-center gap-4">
        {[
          { key: 'all', label: 'All Plans', count: plans.length },
          { key: 'standard', label: 'Standard', count: plans.filter(p => p.plan_category === 'standard').length },
          { key: 'api', label: 'API Plans', count: plans.filter(p => p.plan_category === 'api').length },
          { key: 'enterprise', label: 'Enterprise', count: plans.filter(p => p.plan_category === 'enterprise').length }
        ].map(category => (
          <button
            key={category.key}
            onClick={() => setSelectedCategory(category.key as any)}
            className={`px-6 py-3 rounded-xl font-semibold transition-all ${selectedCategory === category.key
                ? 'bg-gradient-to-r from-emerald-400 to-green-500 text-white shadow-lg'
                : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-600'
              }`}
          >
            {category.label} ({category.count})
          </button>
        ))}
      </div>

      {/* Plans Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {filteredPlans.map(plan => {
          const PlanIcon = getPlanIcon(plan.plan_category)
          const isSubscribed = isUserSubscribed(plan.id)
          const subscription = getUserCurrentPlan(plan.id)
          const isPopular = plan.subscriber_count > 100 // Example popularity threshold

          return (
            <Card
              key={plan.id}
              className={`relative overflow-hidden border-2 transition-all hover:shadow-xl ${isSubscribed
                  ? 'border-emerald-500 bg-emerald-50 dark:bg-emerald-900/20'
                  : 'border-gray-200 dark:border-gray-700 hover:border-emerald-300'
                } ${isPopular ? 'ring-2 ring-yellow-400/50' : ''}`}
            >
              {/* Popular Badge */}
              {isPopular && (
                <div className="absolute top-4 right-4">
                  <Badge className="bg-yellow-400 text-yellow-900">
                    <Star className="h-3 w-3 mr-1" />
                    Popular
                  </Badge>
                </div>
              )}

              {/* Current Plan Badge */}
              {isSubscribed && (
                <div className="absolute top-4 left-4">
                  <Badge className="bg-emerald-500 text-white">
                    <CheckCircle className="h-3 w-3 mr-1" />
                    Current Plan
                  </Badge>
                </div>
              )}

              <CardHeader className="space-y-4">
                <div className="flex items-start gap-3">
                  <div className={`p-2 rounded-xl bg-gradient-to-r ${getPlanColor(plan.plan_category)} flex-shrink-0`}>
                    <PlanIcon className="h-6 w-6 text-white" />
                  </div>
                  <div className="flex-1">
                    <CardTitle className="text-lg leading-tight whitespace-nowrap">{plan.name}</CardTitle>
                    <Badge variant="outline" className="text-xs mt-1">
                      {plan.plan_category.charAt(0).toUpperCase() + plan.plan_category.slice(1)}
                    </Badge>
                  </div>
                </div>

                {plan.description && (
                  <p className="text-gray-600 dark:text-gray-400 text-sm">
                    {plan.description}
                  </p>
                )}

                <div className="flex items-baseline gap-2">
                  <span className="text-3xl font-bold text-gray-900 dark:text-white">
                    ${typeof plan.current_price === 'string' ? parseFloat(plan.current_price).toFixed(2) : plan.current_price}
                  </span>
                  <span className="text-gray-500 dark:text-gray-400">
                    /use
                  </span>
                </div>
              </CardHeader>

              <CardContent>
                <div className="space-y-6">
                  {/* Features by Context */}
                  {Object.entries(groupedFeatures(plan.features)).map(([context, features]) => (
                    <div key={context}>
                      <h4 className="font-semibold text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
                        {context === 'web_app' && '🖥️'}
                        {context === 'api_access' && '🔧'}
                        {context === 'admin_interface' && '⚙️'}
                        {context.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                      </h4>
                      <div className="space-y-2">
                        {features.slice(0, 3).map(feature => (
                          <div key={feature.id} className="flex items-center gap-3">
                            <CheckCircle className="h-4 w-4 text-emerald-500 flex-shrink-0" />
                            <span className="text-sm text-gray-600 dark:text-gray-400">
                              {feature.feature_key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
                              {Object.keys(feature.feature_config).length > 0 && (
                                <span className="text-xs text-gray-500 ml-2">
                                  ({Object.entries(feature.feature_config).map(([k, v]) => `${k}: ${v}`).join(', ')})
                                </span>
                              )}
                            </span>
                          </div>
                        ))}
                        {features.length > 3 && (
                          <div className="text-xs text-gray-500 mt-2">
                            +{features.length - 3} more features
                          </div>
                        )}
                      </div>
                    </div>
                  ))}

                  {/* Action Button */}
                  <div className="pt-4">
                    {!currentUser ? (
                      <Button
                        className="w-full bg-gradient-to-r from-emerald-400 to-green-500 text-white"
                        onClick={() => alert('Please connect your wallet using the navigation menu to subscribe')}
                      >
                        Connect Wallet to Subscribe
                      </Button>
                    ) : isSubscribed ? (
                      <div className="space-y-2">
                        <Button disabled className="w-full bg-emerald-100 text-emerald-800 cursor-not-allowed">
                          <CheckCircle className="h-4 w-4 mr-2" />
                          Subscribed
                        </Button>
                        <Link href="/settings">
                          <Button variant="outline" className="w-full">
                            Manage Subscription
                          </Button>
                        </Link>
                      </div>
                    ) : (
                      <Button
                        onClick={() => handleSubscribe(plan)}
                        className={`w-full bg-gradient-to-r ${getPlanColor(plan.plan_category)} text-white`}
                      >
                        Choose {plan.name}
                      </Button>
                    )}
                  </div>

                  {/* Plan Stats */}
                  <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
                    <div className="flex justify-between text-xs text-gray-500">
                      <span>{plan.subscriber_count} subscribers</span>
                      <span>{plan.target_audience.replace('_', ' ')}</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )
        })}
      </div>

      {filteredPlans.length === 0 && (
        <div className="text-center py-12">
          <div className="text-6xl mb-4">🔍</div>
          <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
            No plans found
          </h3>
          <p className="text-gray-500 dark:text-gray-500">
            Try adjusting your filter or check back later for new plans.
          </p>
        </div>
      )}
    </div>
  )
}