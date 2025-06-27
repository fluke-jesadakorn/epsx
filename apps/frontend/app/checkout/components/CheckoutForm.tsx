'use client'

import { useState, useEffect } from 'react'

import { createPayment, getAssetInfo } from '@/app/actions/payment-server'

import type { CreatePaymentResponse } from '@/types/payment'

interface CheckoutFormProps {
  currency: string
}

type PaymentMethod = 'on_line' | 'on_chain'

type FormErrors = {
  amount?: string;
}

export default function CheckoutForm({ currency }: CheckoutFormProps) {
  const [amount, setAmount] = useState('')
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>('on_line')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [formErrors, setFormErrors] = useState<FormErrors>({})
  const [paymentDetails, setPaymentDetails] = useState<CreatePaymentResponse | null>(null)
  const [assetInfo, setAssetInfo] = useState<Awaited<ReturnType<typeof getAssetInfo>> | null>(null)
  
  useEffect(() => {
    async function fetchAssetInfo() {
      const info = await getAssetInfo(currency)
      setAssetInfo(info)
    }
    fetchAssetInfo()
  }, [currency])
  
  if (!assetInfo) {
    return <div className="animate-pulse">Loading asset information...</div>
  }
  
  if (assetInfo === null) {
    return <div className="text-red-500">Invalid currency selected</div>
  }

  const validateForm = (): boolean => {
    const errors: FormErrors = {}
    const numAmount = parseFloat(amount)

    if (!amount) {
      errors.amount = 'Amount is required'
    } else if (isNaN(numAmount)) {
      errors.amount = 'Amount must be a valid number'
    } else if (assetInfo.depositThreshold !== undefined && numAmount < assetInfo.depositThreshold) {
      errors.amount = `Minimum amount is ${assetInfo.depositThreshold} ${currency}`
    } else if (numAmount <= 0) {
      errors.amount = 'Amount must be greater than 0'
    }

    setFormErrors(errors)
    return Object.keys(errors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!validateForm()) {
      return
    }

    setLoading(true)
    setError('')
    
    try {
      const response = await createPayment({
        currency,
        amount,
        payment_method: paymentMethod,
        product_name: `Payment for ${currency}`,
      })
      
      setPaymentDetails(response)
      // Payment success is shown in the payment details view
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create payment'
      setError(errorMessage)
      // Error is shown in the error state
    } finally {
      setLoading(false)
    }
  }

  if (paymentDetails) {
    return (
      <div className="mt-6 p-6 bg-white dark:bg-gray-800 rounded-lg shadow-md">
        <h3 className="text-lg font-semibold mb-4">Payment Details</h3>
        <div className="text-sm text-gray-500 dark:text-gray-400 mb-4">
          Please complete your payment using the details below
        </div>
        
        <div className="space-y-4">
          <div>
            <div className="text-sm text-gray-600 dark:text-gray-400">Order Number</div>
            <div className="font-medium">{paymentDetails.order_no}</div>
          </div>

          <div className="border-t border-b border-gray-200 dark:border-gray-700 py-4 my-4">
            <div className="text-sm text-gray-600 dark:text-gray-400">Amount</div>
            <div className="font-medium text-lg">{paymentDetails.order_amount} {paymentDetails.currency}</div>
          </div>

          {paymentDetails.receive_address && (
            <div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Deposit Address</div>
              <div className="font-mono bg-gray-50 dark:bg-gray-700 p-3 rounded-lg break-all mt-1 relative group">
                {paymentDetails.receive_address}
                <button
                  onClick={() => {
                    navigator.clipboard.writeText(paymentDetails.receive_address!)
                    // Show copy success visually
                    const button = document.activeElement as HTMLButtonElement;
                    const originalText = button.textContent;
                    button.textContent = 'Copied!';
                    setTimeout(() => {
                      button.textContent = originalText;
                    }, 2000);
                  }}
                  className="absolute right-2 top-2 p-2 opacity-0 group-hover:opacity-100 transition-opacity bg-gray-200 dark:bg-gray-600 rounded-md"
                >
                  Copy
                </button>
              </div>
            </div>
          )}

          {paymentDetails.checkout_url && (
            <div className="mt-6">
              <a 
                href={paymentDetails.checkout_url}
                target="_blank"
                rel="noopener noreferrer"
                className="block w-full text-center px-4 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
              >
                Proceed to Payment Gateway
              </a>
              <div className="text-sm text-gray-500 dark:text-gray-400 mt-2 text-center">
                You will be redirected to our secure payment partner
              </div>
            </div>
          )}
        </div>

        <button
          onClick={() => setPaymentDetails(null)}
          className="mt-6 text-sm text-blue-500 hover:text-blue-600"
        >
          Create Another Payment
        </button>
      </div>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="mt-6 p-6 bg-white dark:bg-gray-800 rounded-lg shadow-md relative">
      {loading && (
        <div className="absolute inset-0 bg-white/50 dark:bg-gray-800/50 flex items-center justify-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      )}
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Amount ({currency})
          </label>
          <input
            type="number"
            step="any"
            min={assetInfo.depositThreshold !== undefined ? assetInfo.depositThreshold : 0}
            required
            value={amount}
            onChange={(e) => {
              setAmount(e.target.value)
              setFormErrors({})
            }}
            className={`mt-1 block w-full p-2 border rounded-md dark:bg-gray-700 
              ${formErrors.amount ? 'border-red-500 dark:border-red-500' : 'border-gray-300 dark:border-gray-600'}
              focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none`}
            placeholder={`Min: ${assetInfo.depositThreshold !== undefined ? assetInfo.depositThreshold : 0}`}
            disabled={loading}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Payment Method
          </label>
          <div className="mt-1 grid grid-cols-2 gap-4">
            {formErrors.amount && (
              <div className="col-span-2 text-sm text-red-500 mt-1">
                {formErrors.amount}
              </div>
            )}
            <button
              type="button"
              onClick={() => setPaymentMethod('on_line')}
              className={`p-3 border rounded-md text-center transition-colors ${
                paymentMethod === 'on_line'
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                  : 'hover:border-gray-300 dark:hover:border-gray-500'
              }`}
              disabled={loading}
            >
              Online Checkout
            </button>
            <button
              type="button"
              onClick={() => setPaymentMethod('on_chain')}
              className={`p-3 border rounded-md text-center transition-colors ${
                paymentMethod === 'on_chain'
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                  : 'hover:border-gray-300 dark:hover:border-gray-500'
              }`}
              disabled={loading}
            >
              On-Chain Transfer
            </button>
          </div>
        </div>

        {error && (
          <div className="text-red-500 text-sm">
            {error}
          </div>
        )}

        <button
          type="submit"
          disabled={loading}
          className={`w-full py-2 px-4 rounded-md text-white transition-colors ${
            loading
              ? 'bg-blue-400 cursor-not-allowed'
              : 'bg-blue-500 hover:bg-blue-600'
          }`}
        >
          {loading ? 'Creating Payment...' : 'Create Payment'}
        </button>
      </div>
    </form>
  )
}
