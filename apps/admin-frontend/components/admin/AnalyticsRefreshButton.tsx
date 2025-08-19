/**
 * Analytics Refresh Button - Client Component for data refresh
 * Handles analytics data refresh with Server Actions
 */

'use client'

import { useTransition } from 'react'
import { RefreshCcw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { toast } from 'react-hot-toast'
import { useRouter } from 'next/navigation'
import { refreshAnalyticsData } from '@/lib/actions/analytics-actions'

interface AnalyticsRefreshButtonProps {
  dateRange: string
  selectedModule: string
}

export function AnalyticsRefreshButton({ dateRange, selectedModule }: AnalyticsRefreshButtonProps) {
  const router = useRouter()
  const [isPending, startTransition] = useTransition()

  const handleRefresh = () => {
    startTransition(async () => {
      try {
        const result = await refreshAnalyticsData(dateRange, selectedModule)

        if (result.success) {
          toast.success('Analytics data refreshed')
          router.refresh() // Refresh the page to show updated data
        } else {
          toast.error(result.error?.message || 'Failed to refresh data')
        }
      } catch (error) {
        toast.error('An unexpected error occurred')
      }
    })
  }

  return (
    <Button 
      variant="outline" 
      size="sm"
      onClick={handleRefresh}
      disabled={isPending}
      className="flex items-center gap-2"
    >
      <RefreshCcw className={`w-4 h-4 ${isPending ? 'animate-spin' : ''}`} />
      {isPending ? 'Refreshing...' : 'Refresh'}
    </Button>
  )
}