/**
 * Billing Invoice Download Button - Client Component for invoice downloads
 * Handles invoice downloads with Server Actions
 */

'use client'

import { useState, useTransition } from 'react'
import { Download } from 'lucide-react'
import { Button } from '@epsx/ui'
import { toast } from 'react-hot-toast'
import { downloadInvoice } from '@/lib/actions/billing-actions'

interface BillingInvoiceDownloadButtonProps {
  invoiceId: string
  invoicePeriod: string
}

export function BillingInvoiceDownloadButton({ 
  invoiceId, 
  invoicePeriod 
}: BillingInvoiceDownloadButtonProps) {
  const [isPending, startTransition] = useTransition()

  const handleDownload = () => {
    startTransition(async () => {
      try {
        const result = await downloadInvoice(invoiceId)

        if (result.success) {
          // Open download URL in new tab
          window.open(result.data.downloadUrl, '_blank')
          toast.success(`Downloading invoice for ${invoicePeriod}`)
        } else {
          toast.error(result.error?.message || 'Failed to download invoice')
        }
      } catch (error) {
        toast.error('An unexpected error occurred')
      }
    })
  }

  return (
    <Button
      size="sm"
      variant="outline"
      onClick={handleDownload}
      disabled={isPending}
      className="flex items-center gap-1"
    >
      <Download className="w-4 h-4" />
      {isPending ? 'Downloading...' : 'Download'}
    </Button>
  )
}