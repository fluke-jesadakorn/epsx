/**
 * User Data Provider Component
 * Provides user data context to child components
 * This is a Server Component that passes data to client components
 */

import { ReactNode } from 'react'
import type { UnifiedUserData } from '@/lib/types/unified-user'

interface UserDataProviderProps {
  userData: UnifiedUserData
  children: ReactNode
}

export function UserDataProvider({ userData, children }: UserDataProviderProps) {
  // For Server Components, we simply pass the data through
  // Client components can access this data via props
  return (
    <div data-user-id={userData.id}>
      {children}
    </div>
  )
}