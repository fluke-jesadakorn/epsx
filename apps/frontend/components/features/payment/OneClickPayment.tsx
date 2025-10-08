'use client'

import { useState, useEffect } from 'react'
import { cn } from '@/lib/utils'
import { 
  Check, 
  ChevronLeft, 
  ChevronRight, 
  Star,
  Shield,
  Zap,
  Users,
  ArrowLeft,
  Lock,
  Smartphone,
  CheckCircle2,
  AlertCircle
} from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { JustInTimeAuth } from '@/components/auth/JustInTimeAuth'
import MetaMaskPayment from './MetaMaskPayment'

interface OneClickPaymentProps {
  className?: string
  preselectedPackage?: string
}

// Raw API response interface (backend data)
interface ApiPaymentPlan {
  id: number
  name: string
  plan_type: string
  base_price: number
  current_price: number
  currency: string
  features: string[] | string // Can be JSON string from API
  affiliate_commission_rate?: number
  display_order?: number
  is_active: boolean
  is_highlighted: boolean
  created_at: string
  updated_at: string
  // Promotion fields
  promotional_badge?: string
  promotional_message?: string
  discount_type?: string
  discount_value?: number
  max_discount_amount?: number
}

// UI-enhanced payment package interface
interface PaymentPackage extends Omit<ApiPaymentPlan, 'features'> {
  features: string[] // Always array in UI
  // UI fields (derived)
  icon?: string
  description?: string
  popular?: boolean
}

type PaymentStep = 'package' | 'payment' | 'confirmation'

// API helper function to fetch plans
const fetchPlans = async (): Promise<PaymentPackage[]> => {
  try {
    const response = await fetch('/api/public/plans', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
    })
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
    
    const plans: ApiPaymentPlan[] = await response.json()
    
    // Transform API response to include UI-specific fields
    return plans.map((plan: ApiPaymentPlan): PaymentPackage => ({
      ...plan,
      // Add UI-specific fields based on plan type or other criteria
      icon: getIconForPlan(plan.plan_type),
      description: getDescriptionForPlan(plan.plan_type),
      popular: plan.is_highlighted || plan.plan_type === 'professional',
      features: Array.isArray(plan.features) ? plan.features : 
                typeof plan.features === 'string' ? JSON.parse(plan.features) :
                getDefaultFeaturesForPlan(plan.plan_type)
    }))
  } catch (error) {
    console.error('Failed to fetch plans:', error)
    // Return fallback data in case of API failure
    return getFallbackPlans()
  }
}

// Helper functions for UI data
const getIconForPlan = (planType: string): string => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic': return '🚀'
    case 'professional':
    case 'pro': return '⭐'
    case 'enterprise':
    case 'premium': return '👑'
    default: return '📊'
  }
}

const getDescriptionForPlan = (planType: string): string => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic': return 'Perfect for beginners'
    case 'professional':
    case 'pro': return 'Most popular choice'
    case 'enterprise':
    case 'premium': return 'For serious traders'
    default: return 'Trading plan'
  }
}

const getDefaultFeaturesForPlan = (planType: string): string[] => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic': return [
      '5 API calls per day',
      'Basic analytics dashboard', 
      'Email support',
      'Mobile app access',
      'Basic stock alerts'
    ]
    case 'professional':
    case 'pro': return [
      'Everything in Starter',
      '50 API calls per day',
      'Advanced analytics & charts', 
      'Priority email support',
      'Real-time data streaming',
      'Portfolio tracking',
      'Custom alerts & notifications'
    ]
    case 'enterprise':
    case 'premium': return [
      'Everything in Professional',
      'Unlimited API calls',
      'Premium analytics suite',
      '24/7 phone & chat support',
      'AI-powered insights',
      'Advanced portfolio management',
      'Custom integrations',
      'Dedicated account manager'
    ]
    default: return ['Standard features included']
  }
}

const getFallbackPlans = (): PaymentPackage[] => [
  {
    id: 1,
    name: 'Starter',
    plan_type: 'starter',
    base_price: 49,
    current_price: 29,
    currency: 'USD',
    is_active: true,
    is_highlighted: false,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    icon: '🚀',
    description: 'Perfect for beginners',
    features: [
      '5 API calls per day',
      'Basic analytics dashboard', 
      'Email support',
      'Mobile app access',
      'Basic stock alerts'
    ]
  },
  {
    id: 2,
    name: 'Professional',
    plan_type: 'professional', 
    base_price: 99,
    current_price: 59,
    currency: 'USD',
    is_active: true,
    is_highlighted: true,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    icon: '⭐',
    description: 'Most popular choice',
    popular: true,
    features: [
      'Everything in Starter',
      '50 API calls per day',
      'Advanced analytics & charts', 
      'Priority email support',
      'Real-time data streaming',
      'Portfolio tracking',
      'Custom alerts & notifications'
    ]
  },
  {
    id: 3,
    name: 'Enterprise',
    plan_type: 'enterprise',
    base_price: 149,
    current_price: 99,
    currency: 'USD',
    is_active: true,
    is_highlighted: false,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    icon: '👑',
    description: 'For serious traders',
    features: [
      'Everything in Professional',
      'Unlimited API calls',
      'Premium analytics suite',
      '24/7 phone & chat support',
      'AI-powered insights',
      'Advanced portfolio management',
      'Custom integrations',
      'Dedicated account manager'
    ]
  }
]

const PAYMENT_METHODS = [
  { id: 'metamask', name: 'MetaMask (Instant)', icon: Zap, description: 'Pay directly with USDT/USDC via MetaMask' },
]

export default function OneClickPayment({ 
  className, 
  preselectedPackage 
}: OneClickPaymentProps) {
  const [packages, setPackages] = useState<PaymentPackage[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedPackage, setSelectedPackage] = useState<number | null>(null)
  const [currentStep, setCurrentStep] = useState<PaymentStep>('package')
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState('metamask')
  const [isProcessing, setIsProcessing] = useState(false)
  const [transactionHash, setTransactionHash] = useState<string | null>(null)

  // Load plans from API
  useEffect(() => {
    const loadPlans = async () => {
      try {
        setLoading(true)
        setError(null)
        const plans = await fetchPlans()
        setPackages(plans)
        
        // Set default selected package
        if (preselectedPackage) {
          const selectedPlan = plans.find(p => 
            p.plan_type.toLowerCase() === preselectedPackage.toLowerCase() ||
            p.name.toLowerCase() === preselectedPackage.toLowerCase()
          )
          setSelectedPackage(selectedPlan?.id || plans[0]?.id || null)
        } else {
          // Default to professional plan or first plan
          const defaultPlan = plans.find(p => p.popular) || plans[0]
          setSelectedPackage(defaultPlan?.id || null)
        }
      } catch (err) {
        console.error('Error loading plans:', err)
        setError('Failed to load plans. Please try again.')
        // Use fallback data
        const fallbackPlans = getFallbackPlans()
        setPackages(fallbackPlans)
        setSelectedPackage(fallbackPlans[1]?.id || fallbackPlans[0]?.id || null)
      } finally {
        setLoading(false)
      }
    }

    loadPlans()
  }, [preselectedPackage])

  const handlePayment = async () => {
    if (selectedPaymentMethod === 'metamask') {
      // MetaMask payment is handled by the MetaMaskPayment component
      return
    }
    
    setIsProcessing(true)
    
    try {
      console.log('Processing payment for package:', selectedPackage)
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      setCurrentStep('confirmation')
    } catch (error) {
      console.error('Payment failed:', error)
      alert('Payment failed. Please try again.')
    } finally {
      setIsProcessing(false)
    }
  }

  const handleMetaMaskSuccess = async (txHash: string) => {
    setTransactionHash(txHash)
    console.log('MetaMask payment successful:', txHash)
    
    // Call backend to confirm payment and activate subscription
    if (selectedPkg) {
      try {
        const response = await fetch('/api/payments/confirm', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          credentials: 'include',
          body: JSON.stringify({
            plan_id: selectedPkg.id,
            transaction_hash: txHash,
            amount: selectedPkg.current_price,
            currency: selectedPkg.currency,
            network: 'ethereum', // Default to Ethereum for now
          }),
        })

        const result = await response.json()
        
        if (result.success) {
          console.log('Subscription activated successfully:', result)
          setCurrentStep('confirmation')
        } else {
          console.error('Failed to activate subscription:', result.message)
          alert(`Payment confirmed but subscription activation failed: ${result.message}`)
          setCurrentStep('confirmation') // Still show confirmation since payment succeeded
        }
      } catch (error) {
        console.error('Error confirming payment:', error)
        alert('Payment succeeded but there was an error activating your subscription. Please contact support.')
        setCurrentStep('confirmation') // Still show confirmation since payment succeeded
      }
    }
  }

  const handleMetaMaskError = (error: string) => {
    console.error('MetaMask payment error:', error)
    alert(`Payment failed: ${error}`)
  }

  const selectedPkg = packages.find(pkg => pkg.id === selectedPackage)

  // Loading state
  if (loading) {
    return (
      <div className={cn('max-w-5xl mx-auto p-4 sm:p-6', className)}>
        <div className="text-center space-y-6">
          <div className="w-16 h-16 border-4 border-primary border-t-transparent rounded-full animate-spin mx-auto" />
          <h2 className="text-xl font-semibold">Loading plans...</h2>
          <p className="text-muted-foreground">Please wait while we fetch the latest pricing</p>
        </div>
      </div>
    )
  }

  // Error state
  if (error && packages.length === 0) {
    return (
      <div className={cn('max-w-5xl mx-auto p-4 sm:p-6', className)}>
        <div className="text-center space-y-6">
          <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto">
            <AlertCircle className="w-8 h-8 text-red-600" />
          </div>
          <div>
            <h2 className="text-xl font-semibold text-red-600">Failed to Load Plans</h2>
            <p className="text-muted-foreground mt-2">{error}</p>
          </div>
          <Button onClick={() => window.location.reload()}>
            Try Again
          </Button>
        </div>
      </div>
    )
  }

  const StepIndicator = () => (
    <div className="flex items-center justify-center mb-6 sm:mb-8">
      <div className="flex items-center space-x-2 sm:space-x-4">
        {(['package', 'payment', 'confirmation'] as PaymentStep[]).map((step, index) => (
          <div key={step} className="flex items-center">
            <div className={cn(
              'w-8 h-8 sm:w-10 sm:h-10 rounded-full flex items-center justify-center text-xs sm:text-sm font-bold transition-all',
              currentStep === step ? 'bg-primary text-primary-foreground scale-110' :
              index < (['package', 'payment', 'confirmation'].indexOf(currentStep)) ? 'bg-green-500 text-white' :
              'bg-muted text-muted-foreground'
            )}>
              {index < (['package', 'payment', 'confirmation'].indexOf(currentStep)) ? 
                <Check className="w-4 h-4" /> : 
                index + 1
              }
            </div>
            {index < 2 && (
              <div className={cn(
                'w-6 sm:w-12 h-0.5 transition-colors',
                index < (['package', 'payment', 'confirmation'].indexOf(currentStep)) ? 'bg-green-500' : 'bg-muted'
              )} />
            )}
          </div>
        ))}
      </div>
    </div>
  )

  if (currentStep === 'confirmation') {
    return (
      <div className={cn('max-w-2xl mx-auto p-4 sm:p-6', className)}>
        <div className="text-center space-y-6">
          <div className="w-20 h-20 sm:w-24 sm:h-24 bg-green-100 dark:bg-green-900 rounded-full flex items-center justify-center mx-auto">
            <CheckCircle2 className="w-10 h-10 sm:w-12 sm:h-12 text-green-600 dark:text-green-400" />
          </div>
          
          <div>
            <h2 className="text-2xl sm:text-3xl font-bold mb-2">🎉 Payment Successful!</h2>
            <p className="text-muted-foreground">
              Welcome to the {selectedPkg?.name} plan. Your account has been upgraded!
            </p>
          </div>

          <Card className="text-left">
            <CardContent className="p-4 sm:p-6">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  <span className="text-2xl">{selectedPkg?.icon}</span>
                  <div>
                    <h3 className="font-semibold">{selectedPkg?.name} Plan</h3>
                    <p className="text-sm text-muted-foreground">Monthly subscription</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-xl font-bold">${selectedPkg?.current_price}</p>
                  <p className="text-xs text-muted-foreground">per month</p>
                </div>
              </div>
              
              <div className="pt-4 border-t">
                <p className="text-sm text-muted-foreground mb-2">What's included:</p>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-1">
                  {selectedPkg?.features.slice(0, 4).map((feature, index) => (
                    <div key={index} className="flex items-center text-sm">
                      <Check className="w-3 h-3 text-green-500 mr-2 flex-shrink-0" />
                      {feature}
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="flex flex-col sm:flex-row gap-3">
            <Button className="flex-1" onClick={() => window.location.href = '/dashboard'}>
              <Smartphone className="w-4 h-4 mr-2" />
              Go to Dashboard
            </Button>
            <Button variant="outline" className="flex-1" onClick={() => setCurrentStep('package')}>
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Plans
            </Button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={cn('max-w-5xl mx-auto p-4 sm:p-6', className)}>
      <StepIndicator />

      {currentStep === 'package' && (
        <>
          <div className="text-center mb-6 sm:mb-8">
            <h2 className="text-2xl sm:text-3xl lg:text-4xl font-bold mb-2 sm:mb-4">
              💎 Choose Your Trading Plan
            </h2>
            <p className="text-muted-foreground text-sm sm:text-base max-w-2xl mx-auto">
              Unlock powerful analytics and take your trading to the next level
            </p>
          </div>

          {/* Mobile: Swipeable Cards */}
          <div className="block md:hidden mb-6">
            <div className="overflow-x-auto pb-4">
              <div className="flex gap-4 px-2">
                {packages.map((pkg) => (
                  <Card
                    key={pkg.id}
                    className={cn(
                      'w-80 flex-shrink-0 cursor-pointer transition-all duration-300',
                      selectedPackage === pkg.id
                        ? 'ring-2 ring-primary shadow-lg scale-105'
                        : 'hover:shadow-md',
                      pkg.popular && 'border-primary'
                    )}
                    onClick={() => setSelectedPackage(pkg.id)}
                  >
                    {pkg.popular && (
                      <div className="absolute -top-3 left-1/2 transform -translate-x-1/2">
                        <Badge className="bg-primary text-primary-foreground px-3 py-1">
                          <Star className="w-3 h-3 mr-1" />
                          Most Popular
                        </Badge>
                      </div>
                    )}
                    
                    <CardHeader className="text-center pb-4">
                      <div className="text-4xl mb-2">{pkg.icon}</div>
                      <CardTitle className="text-xl">{pkg.name}</CardTitle>
                      <p className="text-sm text-muted-foreground">{pkg.description}</p>
                      
                      <div className="pt-2">
                        {pkg.base_price > pkg.current_price && (
                          <p className="text-sm text-muted-foreground line-through">
                            ${pkg.base_price}/month
                          </p>
                        )}
                        <div className="flex items-baseline justify-center gap-1">
                          <span className="text-3xl font-bold">${pkg.current_price}</span>
                          <span className="text-muted-foreground">/month</span>
                        </div>
                        {pkg.base_price > pkg.current_price && (
                          <Badge variant="destructive" className="mt-2">
                            Save ${pkg.base_price - pkg.current_price}/month
                          </Badge>
                        )}
                      </div>
                    </CardHeader>
                    
                    <CardContent className="pt-0">
                      <ul className="space-y-2">
                        {pkg.features.map((feature, index) => (
                          <li key={index} className="flex items-start text-sm">
                            <Check className="w-4 h-4 text-green-500 mr-2 mt-0.5 flex-shrink-0" />
                            {feature}
                          </li>
                        ))}
                      </ul>
                    </CardContent>
                  </Card>
                ))}
              </div>
              <div className="flex justify-center mt-4">
                <p className="text-xs text-muted-foreground">
                  👆 Swipe to explore all plans
                </p>
              </div>
            </div>
          </div>

          {/* Desktop: Grid */}
          <div className="hidden md:grid md:grid-cols-3 gap-6 mb-8">
            {packages.map((pkg) => (
              <Card
                key={pkg.id}
                className={cn(
                  'cursor-pointer transition-all duration-300 relative',
                  selectedPackage === pkg.id
                    ? 'ring-2 ring-primary shadow-xl scale-105'
                    : 'hover:shadow-lg hover:scale-102',
                  pkg.popular && 'border-primary'
                )}
                onClick={() => setSelectedPackage(pkg.id)}
              >
                {pkg.popular && (
                  <div className="absolute -top-3 left-1/2 transform -translate-x-1/2 z-10">
                    <Badge className="bg-primary text-primary-foreground px-3 py-1">
                      <Star className="w-3 h-3 mr-1" />
                      Most Popular
                    </Badge>
                  </div>
                )}
                
                <CardHeader className="text-center">
                  <div className="text-4xl mb-2">{pkg.icon}</div>
                  <CardTitle className="text-xl">{pkg.name}</CardTitle>
                  <p className="text-sm text-muted-foreground">{pkg.description}</p>
                  
                  <div className="pt-4">
                    {pkg.base_price > pkg.current_price && (
                      <p className="text-sm text-muted-foreground line-through">
                        ${pkg.base_price}/month
                      </p>
                    )}
                    <div className="flex items-baseline justify-center gap-1">
                      <span className="text-3xl font-bold">${pkg.current_price}</span>
                      <span className="text-muted-foreground">/month</span>
                    </div>
                    {pkg.base_price > pkg.current_price && (
                      <Badge variant="destructive" className="mt-2">
                        Save ${pkg.base_price - pkg.current_price}/month
                      </Badge>
                    )}
                  </div>
                </CardHeader>
                
                <CardContent>
                  <ul className="space-y-2">
                    {pkg.features.map((feature, index) => (
                      <li key={index} className="flex items-start text-sm">
                        <Check className="w-4 h-4 text-green-500 mr-2 mt-0.5 flex-shrink-0" />
                        {feature}
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            ))}
          </div>

          <div className="text-center">
            <Button 
              onClick={() => setCurrentStep('payment')}
              size="lg"
              className="w-full sm:w-auto px-8 py-3 text-lg font-semibold"
            >
              Continue with {selectedPkg?.name} Plan
              <ChevronRight className="w-5 h-5 ml-2" />
            </Button>
          </div>
        </>
      )}

      {currentStep === 'payment' && selectedPkg && (
        <>
          <div className="flex items-center gap-4 mb-6">
            <Button 
              variant="ghost" 
              size="sm"
              onClick={() => setCurrentStep('package')}
              className="p-2"
            >
              <ChevronLeft className="w-4 h-4" />
            </Button>
            <h2 className="text-xl sm:text-2xl font-bold">Complete Payment</h2>
          </div>

          <div className="grid lg:grid-cols-2 gap-6 sm:gap-8">
            {/* Payment Form */}
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Zap className="w-5 h-5" />
                    Payment Method
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {PAYMENT_METHODS.map((method) => (
                    <div
                      key={method.id}
                      className={cn(
                        'p-4 border rounded-lg cursor-pointer transition-all',
                        selectedPaymentMethod === method.id
                          ? 'border-primary bg-primary/5'
                          : 'border-muted hover:border-primary/50'
                      )}
                      onClick={() => setSelectedPaymentMethod(method.id)}
                    >
                      <div className="flex items-center gap-3">
                        <method.icon className="w-5 h-5" />
                        <div className="flex-1">
                          <p className="font-medium">{method.name}</p>
                          <p className="text-sm text-muted-foreground">{method.description}</p>
                        </div>
                        {selectedPaymentMethod === method.id && (
                          <Check className="w-5 h-5 text-primary" />
                        )}
                      </div>
                    </div>
                  ))}
                </CardContent>
              </Card>

              {/* MetaMask Payment Component */}
              {selectedPaymentMethod === 'metamask' && selectedPkg && (
                <MetaMaskPayment
                  planId={selectedPkg.id}
                  planName={selectedPkg.name}
                  amount={selectedPkg.current_price}
                  currency={selectedPkg.currency}
                  onSuccess={handleMetaMaskSuccess}
                  onError={handleMetaMaskError}
                />
              )}

              {/* Security Info */}
              <Card className="border-green-200 bg-green-50 dark:bg-green-950 dark:border-green-800">
                <CardContent className="p-4">
                  <div className="flex items-center gap-2 text-green-700 dark:text-green-300">
                    <Shield className="w-5 h-5" />
                    <span className="font-medium">Secure Payment</span>
                  </div>
                  <p className="text-sm text-green-600 dark:text-green-400 mt-1">
                    Your MetaMask transaction is processed directly on the blockchain. We never have access to your private keys.
                  </p>
                </CardContent>
              </Card>
            </div>

            {/* Order Summary */}
            <div className="space-y-6">
              <Card className="sticky top-4">
                <CardHeader>
                  <CardTitle>Order Summary</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center gap-3">
                    <span className="text-2xl">{selectedPkg.icon}</span>
                    <div className="flex-1">
                      <h3 className="font-semibold">{selectedPkg.name} Plan</h3>
                      <p className="text-sm text-muted-foreground">Monthly subscription</p>
                    </div>
                  </div>

                  <div className="border-t pt-4 space-y-2">
                    <div className="flex justify-between">
                      <span>Subtotal</span>
                      <span>${selectedPkg.current_price}</span>
                    </div>
                    {selectedPkg.base_price > selectedPkg.current_price && (
                      <div className="flex justify-between text-green-600">
                        <span>Discount</span>
                        <span>-${selectedPkg.base_price - selectedPkg.current_price}</span>
                      </div>
                    )}
                    <div className="flex justify-between font-bold text-lg border-t pt-2">
                      <span>Total</span>
                      <span>${selectedPkg.current_price}/month</span>
                    </div>
                  </div>

                  <JustInTimeAuth
                    onAuthenticated={handlePayment}
                  >
                    <Button 
                      disabled={isProcessing}
                      className="w-full"
                      size="lg"
                    >
                      {isProcessing ? (
                        <>
                          <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin mr-2" />
                          Processing...
                        </>
                      ) : (
                        <>
                          <Lock className="w-4 h-4 mr-2" />
                          Pay ${selectedPkg.current_price} Now
                        </>
                      )}
                    </Button>
                  </JustInTimeAuth>

                  <p className="text-xs text-muted-foreground text-center">
                    By continuing, you agree to our Terms of Service and Privacy Policy
                  </p>
                </CardContent>
              </Card>
            </div>
          </div>
        </>
      )}
    </div>
  )
}