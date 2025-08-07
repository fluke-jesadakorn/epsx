/**
 * User Modal Manager Component
 * Handles modal state and deep linking for user management
 */

'use client'

import { useRouter, useSearchParams } from 'next/navigation'
import { useCallback } from 'react'
import { UserCreateModal } from './UserCreateModal'

export function UserModalManager() {
  const router = useRouter()
  const searchParams = useSearchParams()
  
  const currentModal = searchParams?.get('modal')
  
  const closeModal = useCallback(() => {
    const params = new URLSearchParams(searchParams?.toString())
    params.delete('modal')
    
    const newURL = params.toString() ? `?${params.toString()}` : '/users'
    router.push(newURL)
  }, [router, searchParams])

  const _openModal = useCallback((modalType: string) => {
    const params = new URLSearchParams(searchParams?.toString())
    params.set('modal', modalType)
    
    router.push(`?${params.toString()}`)
  }, [router, searchParams])

  return (
    <>
      <UserCreateModal
        isOpen={currentModal === 'create'}
        onClose={closeModal}
      />
      {/* Future modals can be added here:
          <UserEditModal isOpen={currentModal === 'edit'} onClose={closeModal} />
          <UserDeleteModal isOpen={currentModal === 'delete'} onClose={closeModal} />
      */}
    </>
  )
}