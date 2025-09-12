/**
 * Analytics Export Button - Client Component for analytics exports
 * Navigates to dedicated export page instead of modal
 */

'use client'

import { useRouter } from 'next/navigation'
import { Download } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface AnalyticsExportButtonProps {
  dateRange: string
  selectedModule: string
}

export function AnalyticsExportButton({ dateRange, selectedModule }: AnalyticsExportButtonProps) {
  const router = useRouter()

  const handleExportClick = () => {
    const params = new URLSearchParams({
      dateRange,
      selectedModule
    })
    router.push(`/analytics/export?${params.toString()}`)
  }

  return (
    <Button 
      variant="outline" 
      size="sm"
      onClick={handleExportClick}
      className="flex items-center gap-2"
    >
      <Download className="w-4 h-4" />
      Export
    </Button>
  )
}