'use client'

import { useState } from 'react'
import { cn } from '@/lib/utils'

interface OneClickPaymentProps {
  className?: string
  preselectedPackage?: string
}

interface PaymentPackage {
  id: string
  name: string
  price: number
  features: string[]
}

const MOCK_PACKAGES: PaymentPackage[] = [
  {
    id: 'basic',
    name: 'Basic',
    price: 29,
    features: ['Feature 1', 'Feature 2', 'Feature 3']
  },
  {
    id: 'pro',
    name: 'Pro', 
    price: 59,
    features: ['All Basic features', 'Feature 4', 'Feature 5', 'Feature 6']
  },
  {
    id: 'enterprise',
    name: 'Enterprise',
    price: 99,
    features: ['All Pro features', 'Feature 7', 'Feature 8', 'Priority support']
  }
]

export default function OneClickPayment({ 
  className, 
  preselectedPackage 
}: OneClickPaymentProps) {
  const [selectedPackage, setSelectedPackage] = useState(
    preselectedPackage || 'basic'
  )
  const [isProcessing, setIsProcessing] = useState(false)

  const handlePayment = async () => {
    setIsProcessing(true)
    
    try {
      // TODO: Implement actual payment processing
      console.log('Processing payment for package:', selectedPackage)
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      alert('Payment processed successfully!')
    } catch (error) {
      console.error('Payment failed:', error)
      alert('Payment failed. Please try again.')
    } finally {
      setIsProcessing(false)
    }
  }

  const selectedPkg = MOCK_PACKAGES.find(pkg => pkg.id === selectedPackage)

  return (
    <div className={cn('max-w-4xl mx-auto p-6', className)}>
      <div className="text-center mb-8">
        <h2 className="text-3xl font-bold mb-4">Choose Your Package</h2>
        <p className="text-gray-600 dark:text-gray-400">
          Select the perfect package for your trading needs
        </p>
      </div>

      <div className="grid md:grid-cols-3 gap-6 mb-8">
        {MOCK_PACKAGES.map((pkg) => (
          <div
            key={pkg.id}
            className={cn(
              'border rounded-lg p-6 cursor-pointer transition-all',
              selectedPackage === pkg.id
                ? 'border-blue-500 bg-blue-50 dark:bg-blue-950'
                : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'
            )}
            onClick={() => setSelectedPackage(pkg.id)}
          >
            <h3 className="text-xl font-semibold mb-2">{pkg.name}</h3>
            <div className="text-3xl font-bold text-blue-600 mb-4">
              ${pkg.price}
              <span className="text-sm text-gray-500">/month</span>
            </div>
            <ul className="space-y-2">
              {pkg.features.map((feature, index) => (
                <li key={index} className="flex items-center text-sm">
                  <svg 
                    className="w-4 h-4 text-green-500 mr-2" 
                    fill="currentColor" 
                    viewBox="0 0 20 20"
                  >
                    <path 
                      fillRule="evenodd" 
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" 
                      clipRule="evenodd" 
                    />
                  </svg>
                  {feature}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>

      {selectedPkg && (
        <div className="text-center">
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6 mb-6">
            <h3 className="text-lg font-semibold mb-2">
              Selected: {selectedPkg.name} Package
            </h3>
            <p className="text-2xl font-bold text-blue-600">
              ${selectedPkg.price}/month
            </p>
          </div>

          <button
            onClick={handlePayment}
            disabled={isProcessing}
            className={cn(
              'px-8 py-3 rounded-lg font-semibold text-white transition-colors',
              isProcessing
                ? 'bg-gray-400 cursor-not-allowed'
                : 'bg-blue-600 hover:bg-blue-700'
            )}
          >
            {isProcessing ? 'Processing...' : 'Complete Payment'}
          </button>
        </div>
      )}
    </div>
  )
}