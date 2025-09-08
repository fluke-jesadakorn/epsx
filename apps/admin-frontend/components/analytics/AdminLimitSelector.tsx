'use client'

import { useRouter } from 'next/navigation'

interface AdminLimitSelectorProps {
  currentParams: string
  currentLimit: number
}

export default function AdminLimitSelector({ currentParams, currentLimit }: AdminLimitSelectorProps) {
  const router = useRouter()

  const handleLimitChange = (newLimit: string) => {
    const params = new URLSearchParams(currentParams)
    params.set('limit', newLimit)
    params.set('page', '1') // Reset to first page when changing limit
    router.push(`/analytics/eps?${params.toString()}`)
  }

  return (
    <div className="flex items-center gap-2">
      <span className="text-sm text-gray-600 dark:text-gray-400">Items per page:</span>
      <select
        value={currentLimit}
        onChange={(e) => handleLimitChange(e.target.value)}
        className="rounded-lg border border-gray-200 bg-white/80 px-3 py-1 text-sm transition-all focus:border-orange-400 focus:outline-none focus:ring-2 focus:ring-orange-400/20 dark:border-gray-600 dark:bg-gray-800/80 dark:text-gray-200"
      >
        <option value="6">6</option>
        <option value="12">12</option>
        <option value="24">24</option>
        <option value="48">48</option>
      </select>
    </div>
  )
}