/**
 * Billing Stats Cards - Pure Server Component
 * Displays billing statistics without client-side state
 */

import { DollarSign, Activity, Settings, AlertTriangle } from 'lucide-react'

interface BillingStatsCardsProps {
  currentBill: number
  totalUsage: number
  activeModules: number
  paymentStatus: 'pending' | 'paid' | 'overdue'
  paymentDue: string
}

export function BillingStatsCards({ 
  currentBill, 
  totalUsage, 
  activeModules,
  paymentStatus,
  paymentDue
}: BillingStatsCardsProps) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount)
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'paid': return 'text-green-600 bg-green-100'
      case 'pending': return 'text-yellow-600 bg-yellow-100'
      case 'overdue': return 'text-red-600 bg-red-100'
      default: return 'text-gray-600 bg-gray-100'
    }
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
      <div className="pancake-card p-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-blue-600">{formatCurrency(currentBill)}</p>
            <p className="text-sm text-muted-foreground">Current Bill</p>
            <p className="text-xs text-gray-500 mt-1">
              Due: {new Date(paymentDue).toLocaleDateString()}
            </p>
          </div>
          <div className="p-3 bg-blue-100 rounded-lg">
            <DollarSign className="h-8 w-8 text-blue-500" />
          </div>
        </div>
        
        {/* Payment Status Indicator */}
        <div className="mt-4">
          <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(paymentStatus)}`}>
            {paymentStatus.toUpperCase()}
          </span>
        </div>
      </div>
      
      <div className="pancake-card p-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-green-600">{totalUsage.toLocaleString()}</p>
            <p className="text-sm text-muted-foreground">Total Usage</p>
            <p className="text-xs text-gray-500 mt-1">API requests</p>
          </div>
          <div className="p-3 bg-green-100 rounded-lg">
            <Activity className="h-8 w-8 text-green-500" />
          </div>
        </div>
      </div>

      <div className="pancake-card p-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-purple-600">{activeModules}</p>
            <p className="text-sm text-muted-foreground">Active Modules</p>
            <p className="text-xs text-gray-500 mt-1">modules in use</p>
          </div>
          <div className="p-3 bg-purple-100 rounded-lg">
            <Settings className="h-8 w-8 text-purple-500" />
          </div>
        </div>
      </div>

      <div className="pancake-card p-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-gray-600">
              ${((currentBill / totalUsage) * 1000).toFixed(2)}
            </p>
            <p className="text-sm text-muted-foreground">Cost per 1K</p>
            <p className="text-xs text-gray-500 mt-1">requests</p>
          </div>
          <div className="p-3 bg-gray-100 rounded-lg">
            <AlertTriangle className="h-8 w-8 text-gray-500" />
          </div>
        </div>
      </div>
    </div>
  )
}