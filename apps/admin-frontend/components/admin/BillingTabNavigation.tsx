/**
 * Billing Tab Navigation - Server Component with URL-based navigation
 * Uses Next.js Link components for server-side navigation
 */

import Link from 'next/link'
import { DollarSign, TrendingUp, FileText, AlertTriangle } from 'lucide-react'

interface BillingTabNavigationProps {
  activeTab: string
}

const tabs = [
  { id: 'overview', name: 'Overview', icon: DollarSign },
  { id: 'analytics', name: 'Analytics', icon: TrendingUp },
  { id: 'invoices', name: 'Invoices', icon: FileText },
  { id: 'alerts', name: 'Alerts', icon: AlertTriangle }
]

export function BillingTabNavigation({ activeTab }: BillingTabNavigationProps) {
  return (
    <div className="border-b border-gray-200">
      <nav className="-mb-px flex space-x-8">
        {tabs.map((tab) => {
          const Icon = tab.icon
          const isActive = activeTab === tab.id
          
          return (
            <Link
              key={tab.id}
              href={`?tab=${tab.id}`}
              className={`group inline-flex items-center py-2 px-1 border-b-2 font-medium text-sm transition-colors ${
                isActive
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <Icon className="w-4 h-4 mr-2" />
              {tab.name}
            </Link>
          )
        })}
      </nav>
    </div>
  )
}