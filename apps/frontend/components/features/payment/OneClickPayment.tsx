'use client'

import { useState } from 'react'
import { cn } from '@/lib/utils'
import { 
  CreditCard, 
  Check, 
  ChevronLeft, 
  ChevronRight, 
  Star,
  Shield,
  Zap,
  Users,
  Globe,
  ArrowLeft,
  Lock,
  Smartphone,
  CheckCircle2
} from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'

interface OneClickPaymentProps {
  className?: string
  preselectedPackage?: string
}

interface PaymentPackage {
  id: string
  name: string
  price: number
  originalPrice?: number
  features: string[]
  popular?: boolean
  recommended?: boolean
  icon: string
  description: string
  highlight?: string
}

type PaymentStep = 'package' | 'payment' | 'confirmation'

const MOCK_PACKAGES: PaymentPackage[] = [
  {
    id: 'basic',
    name: 'Starter',
    price: 29,
    originalPrice: 49,
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
    id: 'pro',
    name: 'Professional', 
    price: 59,
    originalPrice: 99,
    icon: '⭐',
    description: 'Most popular choice',
    popular: true,
    recommended: true,
    highlight: 'Best Value',
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
    id: 'enterprise',
    name: 'Enterprise',
    price: 99,
    originalPrice: 149,
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
  { id: 'card', name: 'Credit/Debit Card', icon: CreditCard, description: 'Visa, Mastercard, Amex' },
  { id: 'paypal', name: 'PayPal', icon: Globe, description: 'Fast & secure' },
  { id: 'crypto', name: 'Cryptocurrency', icon: Zap, description: 'Bitcoin, Ethereum' },
]

export default function OneClickPayment({ 
  className, 
  preselectedPackage 
}: OneClickPaymentProps) {
  const [selectedPackage, setSelectedPackage] = useState(
    preselectedPackage || 'pro'
  )
  const [currentStep, setCurrentStep] = useState<PaymentStep>('package')
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState('card')
  const [isProcessing, setIsProcessing] = useState(false)

  const handlePayment = async () => {
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

  const selectedPkg = MOCK_PACKAGES.find(pkg => pkg.id === selectedPackage)

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
                  <p className="text-xl font-bold">${selectedPkg?.price}</p>
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
                {MOCK_PACKAGES.map((pkg) => (
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
                        {pkg.originalPrice && (
                          <p className="text-sm text-muted-foreground line-through">
                            ${pkg.originalPrice}/month
                          </p>
                        )}
                        <div className="flex items-baseline justify-center gap-1">
                          <span className="text-3xl font-bold">${pkg.price}</span>
                          <span className="text-muted-foreground">/month</span>
                        </div>
                        {pkg.originalPrice && (
                          <Badge variant="destructive" className="mt-2">
                            Save ${pkg.originalPrice - pkg.price}/month
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
            {MOCK_PACKAGES.map((pkg) => (
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
                    {pkg.originalPrice && (
                      <p className="text-sm text-muted-foreground line-through">
                        ${pkg.originalPrice}/month
                      </p>
                    )}
                    <div className="flex items-baseline justify-center gap-1">
                      <span className="text-3xl font-bold">${pkg.price}</span>
                      <span className="text-muted-foreground">/month</span>
                    </div>
                    {pkg.originalPrice && (
                      <Badge variant="destructive" className="mt-2">
                        Save ${pkg.originalPrice - pkg.price}/month
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
                    <CreditCard className="w-5 h-5" />
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

              {/* Security Info */}
              <Card className="border-green-200 bg-green-50 dark:bg-green-950 dark:border-green-800">
                <CardContent className="p-4">
                  <div className="flex items-center gap-2 text-green-700 dark:text-green-300">
                    <Shield className="w-5 h-5" />
                    <span className="font-medium">Secure Payment</span>
                  </div>
                  <p className="text-sm text-green-600 dark:text-green-400 mt-1">
                    Your payment information is encrypted and secure. We never store your card details.
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
                      <span>${selectedPkg.price}</span>
                    </div>
                    {selectedPkg.originalPrice && (
                      <div className="flex justify-between text-green-600">
                        <span>Discount</span>
                        <span>-${selectedPkg.originalPrice - selectedPkg.price}</span>
                      </div>
                    )}
                    <div className="flex justify-between font-bold text-lg border-t pt-2">
                      <span>Total</span>
                      <span>${selectedPkg.price}/month</span>
                    </div>
                  </div>

                  <Button 
                    onClick={handlePayment}
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
                        Pay ${selectedPkg.price} Now
                      </>
                    )}
                  </Button>

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