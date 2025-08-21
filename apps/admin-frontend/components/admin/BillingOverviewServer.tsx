/**
 * Billing Overview Server Component - Server-rendered billing details
 * Shows current bill breakdown and usage statistics
 */

import { CreditCard } from 'lucide-react'
import type { UsageBill } from '@/lib/actions/billing-actions'
import { BillingPaymentButton } from './BillingPaymentButton'
import { adminCardVariants, cn } from '@/design-system'

interface BillingOverviewServerProps {
  currentBill: UsageBill
  period: string
}

export function BillingOverviewServer({ currentBill, period }: BillingOverviewServerProps) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount)
  }

  return (
    <div className="space-y-6">
      {/* Current Bill Details */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-semibold text-neutral-900">Current Bill Details</h3>
          {currentBill.status === 'pending' && (
            <BillingPaymentButton billId={currentBill.id} />
          )}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <div className="space-y-4">
              <div className="flex justify-between">
                <span className="text-neutral-600">Billing Period</span>
                <span className="font-medium">
                  {new Date(currentBill.period.start).toLocaleDateString()} - {new Date(currentBill.period.end).toLocaleDateString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-neutral-600">Base Cost</span>
                <span className="font-medium">{formatCurrency(currentBill.pricing.baseCost)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-neutral-600">Usage Cost</span>
                <span className="font-medium">
                  {formatCurrency(currentBill.pricing.totalCost - currentBill.pricing.baseCost)}
                </span>
              </div>
              {currentBill.pricing.discounts.length > 0 && (
                <div className="flex justify-between text-success-600">
                  <span>Total Discounts</span>
                  <span className="font-medium">
                    -{formatCurrency(currentBill.pricing.discounts.reduce((sum, d) => sum + d.amount, 0))}
                  </span>
                </div>
              )}
              <hr className="my-4" />
              <div className="flex justify-between text-lg font-bold">
                <span>Total Amount</span>
                <span>{formatCurrency(currentBill.pricing.totalCost)}</span>
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <h4 className="font-semibold text-neutral-900">Usage Summary</h4>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-neutral-600">Total API Calls</span>
                <span className="font-medium">{currentBill.usage.totalRequests.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-neutral-600">Average Cost per 1K</span>
                <span className="font-medium">
                  ${((currentBill.pricing.totalCost / currentBill.usage.totalRequests) * 1000).toFixed(4)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-neutral-600">Active Modules</span>
                <span className="font-medium">{currentBill.usage.moduleBreakdown.length}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Usage Breakdown by Module */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }), 'overflow-hidden')}>
        <div className="px-6 py-4 border-b border-neutral-200">
          <h3 className="text-lg font-semibold text-neutral-900">Module Usage Breakdown</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-neutral-200">
            <thead className="bg-neutral-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Module
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Requests
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Cost
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Per 1K Requests
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  % of Total
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-neutral-200">
              {currentBill.usage.moduleBreakdown.map((module, index) => {
                const percentage = ((module.requests / currentBill.usage.totalRequests) * 100).toFixed(1)
                const costPer1K = (module.cost / module.requests * 1000).toFixed(4)
                
                return (
                  <tr key={index} className="hover:bg-neutral-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="font-medium text-neutral-900">{module.moduleName}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-neutral-900">
                      {module.requests.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-neutral-900">
                      {formatCurrency(module.cost)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-neutral-600">
                      ${costPer1K}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <div className="w-16 bg-neutral-200 rounded-full h-2 mr-2">
                          <div 
                            className="bg-info-600 h-2 rounded-full" 
                            style={{ width: `${percentage}%` }}
                          ></div>
                        </div>
                        <span className="text-sm text-neutral-600">{percentage}%</span>
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Discounts Applied */}
      {currentBill.pricing.discounts.length > 0 && (
        <div className="bg-success-50 border border-success-200 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-success-900 mb-4">Discounts Applied</h3>
          <div className="space-y-3">
            {currentBill.pricing.discounts.map((discount, index) => (
              <div key={index} className="flex justify-between items-center">
                <div>
                  <div className="font-medium text-success-800">{discount.description}</div>
                  <div className="text-sm text-success-600">Applied to this billing period</div>
                </div>
                <span className="font-bold text-success-900">
                  -{formatCurrency(discount.amount)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}