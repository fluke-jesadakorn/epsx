'use server'

import { z } from 'zod'

import { apiClient } from '@/lib/api-client'

import type { CreatePaymentRequest, CreatePaymentResponse, AssetInfo } from '@/types/payment'

async function getCreatePaymentSchema() {
  return z.object({
  currency: z.string(),
  amount: z.string(),
  payment_method: z.enum(['on_line', 'on_chain']),
  product_name: z.string(),
  notify_url: z.string().optional()
  })
}

export async function createPayment(
  data: CreatePaymentRequest
): Promise<CreatePaymentResponse> {
  try {
    const schema = await getCreatePaymentSchema()
    const validatedData = schema.parse(data)
    
    const result = await apiClient.post('/api/payments', validatedData)
    
    if (!result.success) {
      throw new Error('Failed to create payment')
    }
    
    return result.data as CreatePaymentResponse
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new Error('Invalid payment data: ' + error.message)
    }
    throw error
  }
}

import { supportedAssets } from '@/app/constants/assets'

// Get supported crypto assets
async function getSupportedAssets(): Promise<AssetInfo[]> {
  return supportedAssets
}

// Get asset info by currency
export async function getAssetInfo(currency: string): Promise<AssetInfo | undefined> {
  const assets = await getSupportedAssets()
  return assets.find(asset => asset.currency === currency)
}

// Get payment status from backend
export async function getPaymentStatus(): Promise<any> {
  try {
    const result = await apiClient.get('/api/payments/status')
    
    if (!result.success) {
      console.error('Failed to get payment status')
      return null
    }
    
    return result.data
  } catch (error) {
    console.error('Failed to get payment status:', error)
    return null
  }
}

// Verify payment transaction
export async function verifyPayment(transactionId: string): Promise<boolean> {
  try {
    const schema = z.object({
      transactionId: z.string().min(1)
    })
    
    const validatedData = schema.parse({ transactionId })
    
    const result = await apiClient.post('/api/payments/verify', { transactionId: validatedData.transactionId })
    
    if (!result.success) {
      console.error('Failed to verify payment')
      return false
    }
    
    return (result.data as any)?.verified || false
  } catch (error) {
    console.error('Failed to verify payment:', error)
    return false
  }
}

// Cancel pending payment
export async function cancelPayment(paymentId: string): Promise<boolean> {
  try {
    const schema = z.object({
      paymentId: z.string().min(1)
    })
    
    const validatedData = schema.parse({ paymentId })
    
    const result = await apiClient.delete(`/api/payments/${validatedData.paymentId}`)
    
    if (!result.success) {
      console.error('Failed to cancel payment')
      return false
    }
    
    return true
  } catch (error) {
    console.error('Failed to cancel payment:', error)
    return false
  }
}
