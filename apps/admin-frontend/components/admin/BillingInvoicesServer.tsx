/**
 * Billing Invoices Server Component - Server-rendered invoice list
 * Shows invoice history with download capabilities
 */

import { FileText, Download } from 'lucide-react'
import type { InvoiceData } from '@/lib/actions/billing-actions'
import { BillingInvoiceDownloadButton } from './BillingInvoiceDownloadButton'

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
      case 'paid': return 'bg-green-100 text-green-800'
      case 'pending': return 'bg-yellow-100 text-yellow-800'
      case 'overdue': return 'bg-red-100 text-red-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  if (invoices.length === 0) {
    return (
      <div className="pancake-card p-6">
        <div className="text-center py-12">
          <FileText className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">No Invoices Found</h3>
          <p className="text-gray-600">Invoice history will appear here once billing cycles are complete.</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="pancake-card overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Invoice History
          </h3>
        </div>
        
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Period
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Amount
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Due Date
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {invoices.map((invoice) => (
                <tr key={invoice.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="font-medium text-gray-900">{invoice.period}</div>
                    <div className="text-sm text-gray-500">ID: {invoice.id}</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="font-semibold text-gray-900">
                      {formatCurrency(invoice.amount)}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(invoice.status)}`}>
                      {invoice.status.charAt(0).toUpperCase() + invoice.status.slice(1)}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                    <div>{new Date(invoice.dueDate).toLocaleDateString()}</div>
                    {invoice.status === 'overdue' && (
                      <div className="text-xs text-red-600 font-medium">
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
        <div className="pancake-card p-4">
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {invoices.filter(i => i.status === 'paid').length}
            </div>
            <div className="text-sm text-muted-foreground">Paid Invoices</div>
          </div>
        </div>
        
        <div className="pancake-card p-4">
          <div className="text-center">
            <div className="text-2xl font-bold text-yellow-600">
              {invoices.filter(i => i.status === 'pending').length}
            </div>
            <div className="text-sm text-muted-foreground">Pending Invoices</div>
          </div>
        </div>
        
        <div className="pancake-card p-4">
          <div className="text-center">
            <div className="text-2xl font-bold text-gray-600">
              {formatCurrency(invoices.reduce((sum, i) => sum + i.amount, 0))}
            </div>
            <div className="text-sm text-muted-foreground">Total Billed</div>
          </div>
        </div>
      </div>
    </div>
  )
}