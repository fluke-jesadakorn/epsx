/**
 * Billing Invoices Server Component - Server-rendered invoice list
 * Shows invoice history with download capabilities
 */

import { FileText, Download } from 'lucide-react'
import type { InvoiceData } from '@/lib/actions/billing-actions'
import { BillingInvoiceDownloadButton } from './BillingInvoiceDownloadButton'
import { adminCardVariants, cn } from '@/design-system'

interface BillingInvoicesServerProps {
  invoices: InvoiceData[]
}

export function BillingInvoicesServer({ invoices }: BillingInvoicesServerProps) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount)
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'paid': return 'bg-success-100 text-success-800'
      case 'pending': return 'bg-warning-100 text-warning-800'
      case 'overdue': return 'bg-error-100 text-error-800'
      default: return 'bg-neutral-100 text-neutral-800'
    }
  }

  if (invoices.length === 0) {
    return (
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="text-center py-12">
          <FileText className="w-12 h-12 text-neutral-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-neutral-900 mb-2">No Invoices Found</h3>
          <p className="text-neutral-600">Invoice history will appear here once billing cycles are complete.</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className={cn(adminCardVariants({ variant: 'pancake' }), 'overflow-hidden')}>
        <div className="px-6 py-4 border-b border-neutral-200">
          <h3 className="text-lg font-semibold text-neutral-900 flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Invoice History
          </h3>
        </div>
        
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-neutral-200">
            <thead className="bg-neutral-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Period
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Amount
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Due Date
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-neutral-200">
              {invoices.map((invoice) => (
                <tr key={invoice.id} className="hover:bg-neutral-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="font-medium text-neutral-900">{invoice.period}</div>
                    <div className="text-sm text-neutral-500">ID: {invoice.id}</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="font-semibold text-neutral-900">
                      {formatCurrency(invoice.amount)}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(invoice.status)}`}>
                      {invoice.status.charAt(0).toUpperCase() + invoice.status.slice(1)}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-neutral-900">
                    <div>{new Date(invoice.dueDate).toLocaleDateString()}</div>
                    {invoice.status === 'overdue' && (
                      <div className="text-xs text-error-600 font-medium">
                        {Math.ceil((Date.now() - new Date(invoice.dueDate).getTime()) / (1000 * 60 * 60 * 24))} days overdue
                      </div>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <BillingInvoiceDownloadButton 
                      invoiceId={invoice.id}
                      invoicePeriod={invoice.period}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Invoice Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="text-center">
            <div className="text-2xl font-bold text-success-600">
              {invoices.filter(i => i.status === 'paid').length}
            </div>
            <div className="text-sm text-muted-foreground">Paid Invoices</div>
          </div>
        </div>
        
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="text-center">
            <div className="text-2xl font-bold text-warning-600">
              {invoices.filter(i => i.status === 'pending').length}
            </div>
            <div className="text-sm text-muted-foreground">Pending Invoices</div>
          </div>
        </div>
        
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="text-center">
            <div className="text-2xl font-bold text-neutral-600">
              {formatCurrency(invoices.reduce((sum, i) => sum + i.amount, 0))}
            </div>
            <div className="text-sm text-muted-foreground">Total Billed</div>
          </div>
        </div>
      </div>
    </div>
  )
}