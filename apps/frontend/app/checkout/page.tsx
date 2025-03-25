'use client'

import { useState } from 'react'
import AssetSelection from './components/AssetSelection'
import CheckoutForm from './components/CheckoutForm'
import { useAuth } from '@/context/auth-context'
import { useRouter } from 'next/navigation'

export default function CheckoutPage() {
  const { isLoggedIn, userEmail } = useAuth()
  const router = useRouter()
  const [selectedAsset, setSelectedAsset] = useState('')

  if (!isLoggedIn || !userEmail) {
    router.push('/login')
    return null
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold mb-6">Checkout</h1>
      <div className="max-w-2xl mx-auto">
        <AssetSelection 
          selectedAsset={selectedAsset} 
          onSelect={setSelectedAsset} 
        />
        {selectedAsset && (
          <CheckoutForm 
            currency={selectedAsset}
          />
        )}
      </div>
    </div>
  )
}
