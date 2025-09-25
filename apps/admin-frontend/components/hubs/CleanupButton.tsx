'use client'

import { RefreshCw } from 'lucide-react'
import { cleanupExpiredPermissionsAction } from '@/lib/actions/consolidated-admin-actions'

export function CleanupButton() {
  const handleCleanup = async () => {
    try {
      const result = await cleanupExpiredPermissionsAction()
      console.log('Cleanup result:', result)
    } catch (error) {
      console.error('Cleanup failed:', error)
    }
  }

  return (
    <button 
      onClick={handleCleanup}
      className="flex items-center gap-2 px-4 py-3 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors"
    >
      <RefreshCw size={20} />
      🧹 Cleanup
    </button>
  )
}