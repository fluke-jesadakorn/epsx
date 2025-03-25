'use server'

import { z } from 'zod'
import { cookies } from 'next/headers'
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
    
    // Get session token from cookies
    const cookieStore = await cookies()
    const sessionToken = cookieStore.get('__session')
    
    if (!sessionToken) {
      throw new Error('Authentication required')
    }

    // Pass session token in headers
    return await apiClient.post<CreatePaymentResponse>(
      '/payment/create', 
      validatedData,
      { 'Authorization': `Bearer ${sessionToken.value}` }
    )
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
