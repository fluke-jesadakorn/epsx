/**
 * Create User Button Component
 * Button that opens create user modal with deep linking
 */

'use client'

import { Plus } from 'lucide-react'
import { useRouter, useSearchParams } from 'next/navigation'

export function CreateUserButton() {
  const router = useRouter()
  const searchParams = useSearchParams()

  const handleClick = () => {
    const params = new URLSearchParams(searchParams?.toString())
    params.set('modal', 'create')
    router.push(`?${params.toString()}`)
  }

  return (
    <button 
      onClick={handleClick}
      className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
      data-testid="create-user-button"
    >
      <Plus className="h-4 w-4" />
      Create User
    </button>
  )
}