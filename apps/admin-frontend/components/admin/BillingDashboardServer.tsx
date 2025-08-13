/**
 * Billing Dashboard Server Component - Server-side first architecture
 * Fetches billing data on the server and renders dashboard with client components for interactions
 */

import { DollarSign, CreditCard, TrendingUp, Download, AlertTriangle, Settings, Activity, FileText } from 'lucide-react'
import { getCurrentUser } from '@/lib/actions/server-auth'
import { getBillingDashboardData } from '@/lib/actions/billing-actions'
import { BillingStatsCards } from './BillingStatsCards'
import { BillingOverviewServer } from './BillingOverviewServer'
import { BillingInvoicesServer } from './BillingInvoicesServer'
import { BillingTabNavigation } from './BillingTabNavigation'
import { BillingExportButton } from './BillingExportButton'

interface BillingDashboardServerProps {
  searchParams: {
    tab?: string
    period?: string
  }
}

export async function BillingDashboardServer({ searchParams }: BillingDashboardServerProps) {
  const currentUser = await getCurrentUser()
  
  // Check authentication
  if (!currentUser) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-yellow-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Authentication Required</h3>
          <p className="text-gray-600">Please sign in to access billing information.</p>
        </div>
      </div>
    )
  }

  // Check billing admin permissions
  const canManageBilling = currentUser.admin && 
    (currentUser.admin_modules.includes('billing_admin') || 
     currentUser.admin_modules.includes('system_admin'))

  if (!canManageBilling) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-yellow-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Access Restricted</h3>
          <p className="text-gray-600">You don&apos;t have permission to view billing information.</p>
        </div>
      </div>
    )
  }

  // Fetch billing data on the server
  const period = searchParams.period || 'current'
  const billingResult = await getBillingDashboardData(period)

  if (!billingResult.success) {
    return (
      <div className="max-w-7xl mx-auto p-6">
        <div className="pancake-card p-6 text-center">
          <AlertTriangle className="w-12 h-12 text-red-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Failed to Load Billing Data</h3>
          <p className="text-gray-600">{billingResult.error?.message}</p>
        </div>
      </div>
    )
  }

  const { currentBill, invoices, usageStats } = billingResult.data
  const activeTab = searchParams.tab || 'overview'

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Billing & Analytics</h1>
          <p className="text-gray-600">Monitor usage, costs, and billing across all modules</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <BillingPeriodSelector currentPeriod={period} />
          <BillingExportButton period={period} />
        </div>
      </div>

      {/* Tab Navigation */}
      <BillingTabNavigation activeTab={activeTab} />

      {/* Stats Cards - Always visible */}
      <BillingStatsCards 
        currentBill={currentBill.pricing.totalCost}
        totalUsage={usageStats.totalApiCalls}
        activeModules={usageStats.activeModules}
        paymentStatus={currentBill.status}
        paymentDue={currentBill.paymentDue}
      />

      {/* Tab Content */}
      {activeTab === 'overview' && (
        <BillingOverviewServer 
          currentBill={currentBill}
          period={period}
        />
      )}

      {activeTab === 'analytics' && (
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Module Analytics
          </h3>
          {/* Module Analytics would be imported from ModuleAnalyticsDashboard */}
          <div className="text-center py-8 text-muted-foreground">
            <TrendingUp className="h-12 w-12 mx-auto mb-2 opacity-50" />
            <p>Analytics dashboard integration pending</p>
          </div>
        </div>
      )}

      {activeTab === 'invoices' && (
        <BillingInvoicesServer invoices={invoices} />
      )}

      {activeTab === 'alerts' && (
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            Billing Alerts
          </h3>
          {/* Billing Alerts would be rendered here */}
          <div className="text-center py-8 text-muted-foreground">
            <AlertTriangle className="h-12 w-12 mx-auto mb-2 opacity-50" />
            <p>No active billing alerts</p>
          </div>
        </div>
      )}
    </div>
  )
}

/**
 * Billing Period Selector - Server Component with Client interactions
 */
function BillingPeriodSelector({ currentPeriod }: { currentPeriod: string }) {
  return (
    <form method="get">
      <select 
        name="period"
        defaultValue={currentPeriod}
        className="border rounded-md px-3 py-2 text-sm"
        onChange={(e) => {
          const form = e.target.form
          if (form) {
            form.submit()
          }
        }}
      >
        <option value="current">Current Period</option>
        <option value="last_month">Last Month</option>
        <option value="last_quarter">Last Quarter</option>
        <option value="last_year">Last Year</option>
      </select>
    </form>
  )
}