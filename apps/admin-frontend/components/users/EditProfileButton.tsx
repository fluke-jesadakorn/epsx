/**
 * Edit Profile Button Component
 * Button that navigates to edit profile page
 */

'use client'

import { Edit } from 'lucide-react'
import { useRouter } from 'next/navigation'

interface EditProfileButtonProps {
  userId: string
  className?: string
  disabled?: boolean
}

export function EditProfileButton({ userId, className = '', disabled = false }: EditProfileButtonProps) {
  const router = useRouter()

  const handleClick = () => {
    if (disabled) return
    
    // Navigate to edit page
    router.push(`/users/${userId}/edit`)
  }

  return (
    <button 
      onClick={handleClick}
      disabled={disabled}
      className={`inline-flex items-center gap-2 px-3 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${className}`}
      data-testid="edit-profile-button"
    >
      <Edit className="h-4 w-4" />
      Edit Profile
    </button>
  )
}