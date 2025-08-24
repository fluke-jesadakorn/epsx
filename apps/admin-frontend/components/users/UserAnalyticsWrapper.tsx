'use client'

import { useState } from 'react'
import { UserAnalyticsDashboard } from './UserAnalyticsDashboard'
import type { UnifiedUserData } from '@/lib/types/unified-user'

interface UserAnalyticsWrapperProps {
  users: UnifiedUserData[]
  total: number
}

export function UserAnalyticsWrapper({ users, total }: UserAnalyticsWrapperProps) {
  const [isExpanded, setIsExpanded] = useState(false)

  const handleToggleExpanded = () => {
    setIsExpanded(!isExpanded)
  }

  return (
    <UserAnalyticsDashboard
      users={users}
      total={total}
      isExpanded={isExpanded}
      onToggleExpanded={handleToggleExpanded}
    />
  )
}