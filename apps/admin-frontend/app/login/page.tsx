'use client'

import { useEffect } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'

export default function AdminLoginPage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  
  useEffect(() => {
    // Preserve all search params when redirecting
    const params = new URLSearchParams(searchParams.toString())
    const redirectUrl = `/login/web3${params.toString() ? `?${params.toString()}` : ''}`
    
    console.log('🔄 Login: Redirecting to Web3 login page:', redirectUrl)
    router.replace(redirectUrl)
  }, [router, searchParams])

  // Return null since we're immediately redirecting
  return null
}