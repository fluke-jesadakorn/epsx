/**
 * Billing Export Button - Client Component for report exports
 * Handles billing report exports with Server Actions
 */

'use client'

import { useState, useTransition } from 'react'
import { Download, FileText } from 'lucide-react'
import { Button } from '@epsx/ui'
import { toast } from 'react-hot-toast'
import { exportBillingReport } from '@/lib/actions/billing-actions'

interface BillingExportButtonProps {
  period: string
}

export function BillingExportButton({ period }: BillingExportButtonProps) {
  const [isPending, startTransition] = useTransition()
  const [showExportModal, setShowExportModal] = useState(false)

  const handleExport = (format: 'csv' | 'pdf' | 'xlsx') => {
    startTransition(async () => {
      try {
        const result = await exportBillingReport(period, format)

        if (result.success) {
          // Open download URL in new tab
          window.open(result.data.downloadUrl, '_blank')
          toast.success(`Exporting billing report as ${format.toUpperCase()}`)
          setShowExportModal(false)
        } else {
          toast.error(result.error?.message || 'Failed to export report')
        }
      } catch (error) {
        toast.error('An unexpected error occurred')
      }
    })
  }

  return (
    <>
      <Button 
        variant="outline" 
        onClick={() => setShowExportModal(true)}
        disabled={isPending}
        className="flex items-center gap-2"
      >
        <Download className="w-4 h-4" />
        {isPending ? 'Exporting...' : 'Export Report'}
      </Button>

      {/* Export Modal */}
      {showExportModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <FileText className="w-5 h-5" />
              Export Billing Report
            </h3>
            
            <div className="space-y-4">
              <p className="text-gray-600">
                Choose the format for your billing report export:
              </p>
              
              <div className="space-y-2">
                <Button
                  className="w-full justify-start"
                  variant="outline"
                  onClick={() => handleExport('pdf')}
                  disabled={isPending}
                >
                  <FileText className="w-4 h-4 mr-2" />
                  PDF Report
                </Button>
                
                <Button
                  className="w-full justify-start"
                  variant="outline"
                  onClick={() => handleExport('xlsx')}
                  disabled={isPending}
                >
                  <FileText className="w-4 h-4 mr-2" />
                  Excel Spreadsheet (.xlsx)
                </Button>

                <Button
                  className="w-full justify-start"
                  variant="outline"
                  onClick={() => handleExport('csv')}
                  disabled={isPending}
                >
                  <FileText className="w-4 h-4 mr-2" />
                  CSV Data (.csv)
                </Button>
              </div>
              
              <div className="flex gap-2 pt-4 border-t">
                <Button
                  variant="outline"
                  onClick={() => setShowExportModal(false)}
                  disabled={isPending}
                  className="flex-1"
                >
                  Cancel
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  )
}