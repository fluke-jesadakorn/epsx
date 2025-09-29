/**
 * Group Assignment History Component
 * Placeholder for group assignment history
 */

'use client'

import React from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

interface GroupAssignmentHistoryProps {
  className?: string
}

export function GroupAssignmentHistory({ className }: GroupAssignmentHistoryProps) {
  return (
    <div className={`space-y-6 ${className || ''}`}>
      <Card>
        <CardHeader>
          <CardTitle>Assignment History</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-gray-600">
            Assignment history will be available in a future update.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}