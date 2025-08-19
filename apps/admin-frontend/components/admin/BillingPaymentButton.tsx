/**
 * Billing Payment Button - Client Component for payment interactions
 * Handles payment processing with Server Actions
 */

'use client'

import { useState, useTransition } from 'react'
import { CreditCard } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { toast } from 'react-hot-toast'
import { useRouter } from 'next/navigation'
import { processPayment } from '@/lib/actions/billing-actions'

interface BillingPaymentButtonProps {
  billId: string
}

export function BillingPaymentButton({ billId }: BillingPaymentButtonProps) {
  const router = useRouter()
  const [isPending, startTransition] = useTransition()
  const [showPaymentModal, setShowPaymentModal] = useState(false)

  const handlePayment = async (paymentMethod: string) => {
    startTransition(async () => {
      const result = await processPayment(billId, paymentMethod)

      if (result.success) {
        toast.success('Payment processed successfully!')
        setShowPaymentModal(false)
        router.refresh() // Refresh to show updated payment status
      } else {
        toast.error(result.error?.message || 'Payment failed')
      }
    })
  }

  return (
    <>
      <Button
        onClick={() => setShowPaymentModal(true)}
        disabled={isPending}
        className="flex items-center gap-2"
      >
        <CreditCard className="w-4 h-4" />
        {isPending ? 'Processing...' : 'Make Payment'}
      </Button>

      {/* Payment Modal */}
      {showPaymentModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold mb-4">Process Payment</h3>
            
            <div className="space-y-4">
              <p className="text-gray-600">
                Select a payment method to process your bill payment.
              </p>
              
              <div className="space-y-2">
                <Button
                  className="w-full justify-start"
                  variant="outline"
                  onClick={() => handlePayment('credit_card')}
                  disabled={isPending}
                >
                  <CreditCard className="w-4 h-4 mr-2" />
                  Credit Card
                </Button>
                
                <Button
                  className="w-full justify-start"
                  variant="outline"
                  onClick={() => handlePayment('bank_transfer')}
                  disabled={isPending}
                >
                  <CreditCard className="w-4 h-4 mr-2" />
                  Bank Transfer
                </Button>
              </div>
              
              <div className="flex gap-2 pt-4 border-t">
                <Button
                  variant="outline"
                  onClick={() => setShowPaymentModal(false)}
                  disabled={isPending}
                  className="flex-1"
                >
                  Cancel
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  )
}