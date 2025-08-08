/**
 * Create User Button Component
 * Button that navigates to create user page
 */

'use client'

import { Plus } from 'lucide-react'
import { useRouter } from 'next/navigation'

interface CreateUserButtonProps {
  className?: string
  disabled?: boolean
}

export function CreateUserButton({ className = '', disabled = false }: CreateUserButtonProps) {
  const router = useRouter()

  const handleClick = () => {
    if (disabled) return
    
    // Navigate to create page
    router.push('/users/create')
  }

  return (
    <button 
      onClick={handleClick}
      disabled={disabled}
      className={`inline-flex items-center gap-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${className}`}
      data-testid="create-user-button"
    >
      <Plus className="h-4 w-4" />
      Create User
    </button>
  )
}