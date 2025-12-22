import { notFound } from 'next/navigation'
import { Suspense } from 'react'

import { AffiliateManagement, type Affiliate } from '@/components/affiliates/AffiliateManagement'
import { UnifiedAuth } from '@/lib/auth/auth'

export const dynamic = 'force-dynamic'

function AffiliatesHubSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-indigo-400 to-violet-500 rounded-2xl w-96 mx-auto mb-4 animate-pulse shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto animate-pulse"></div>
        </div>

        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-gradient-to-br from-indigo-400 via-violet-500 to-purple-500 rounded-3xl p-8 shadow-2xl animate-pulse">
              <div className="h-12 w-12 bg-white/20 rounded-2xl mb-6"></div>
              <div className="h-8 bg-white/30 rounded-xl mb-4 w-3/4"></div>
              <div className="h-5 bg-white/20 rounded-lg mb-6 w-full"></div>
              <div className="h-12 bg-white/40 rounded-2xl"></div>
            </div>
          ))}
        </div>

        {/* Stats grid skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-6 shadow-xl border border-white/20 animate-pulse">
              <div className="h-6 bg-gradient-to-r from-indigo-300 to-violet-400 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-gray-200 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>

        {/* Affiliates table skeleton */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-gradient-to-r from-indigo-400 to-violet-500 rounded-xl mb-6 w-1/3 animate-pulse"></div>
            <div className="space-y-4">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl animate-pulse">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="h-10 w-10 bg-gradient-to-br from-indigo-400 to-violet-500 rounded-2xl"></div>
                    <div className="space-y-2 flex-1">
                      <div className="h-5 bg-gray-300 rounded-lg w-1/3"></div>
                      <div className="h-4 bg-gray-200 rounded-lg w-1/2"></div>
                    </div>
                  </div>
                  <div className="h-8 w-24 bg-gradient-to-r from-green-400 to-green-500 rounded-full"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

async function AffiliatesDataWrapper() {
  const session = await UnifiedAuth.getSession()

  if (!session?.user) {
    notFound()
  }

  // NOTE: Permission enforcement moved to backend
  // If user lacks permission, API calls will return 403 and show Access Denied UI

  // Demo affiliate data
  const demoAffiliates: Affiliate[] = [
    {
      id: 1,
      name: 'TechInfluencer Pro',
      email: 'sarah@techinfluencer.com',
      affiliateCode: 'TECHPRO',
      status: 'active',
      commissionRate: 20,
      tier: 'Premium',
      totalReferrals: 245,
      totalSales: 12450.75,
      totalCommissions: 2490.15,
      pendingCommissions: 450.20,
      paidCommissions: 2039.95,
      conversionRate: 15.8,
      avgOrderValue: 87.50,
      paymentMethod: 'PayPal',
      paymentEmail: 'sarah@techinfluencer.com',
      joinedAt: '2024-03-15T00:00:00Z',
      lastActive: '2024-12-08T14:30:00Z',
      approvedAt: '2024-03-16T09:00:00Z',
      notes: 'High-performing tech influencer with engaged audience'
    },
    {
      id: 2,
      name: 'CryptoTrader Hub',
      email: 'mike@cryptotraderhub.io',
      affiliateCode: 'CRYPTOHUB',
      status: 'active',
      commissionRate: 25,
      tier: 'Elite',
      totalReferrals: 456,
      totalSales: 28750.50,
      totalCommissions: 7187.63,
      pendingCommissions: 1250.75,
      paidCommissions: 5936.88,
      conversionRate: 22.3,
      avgOrderValue: 156.25,
      paymentMethod: 'Bank Transfer',
      paymentEmail: null,
      joinedAt: '2024-01-20T00:00:00Z',
      lastActive: '2024-12-09T10:15:00Z',
      approvedAt: '2024-01-21T16:45:00Z',
      notes: 'Elite partner with crypto trading community'
    },
    {
      id: 3,
      name: 'FinanceYouTuber',
      email: 'alex@financechannel.com',
      affiliateCode: 'FINYOUTUBE',
      status: 'pending',
      commissionRate: 15,
      tier: 'Standard',
      totalReferrals: 0,
      totalSales: 0,
      totalCommissions: 0,
      pendingCommissions: 0,
      paidCommissions: 0,
      conversionRate: 0,
      avgOrderValue: 0,
      paymentMethod: 'PayPal',
      paymentEmail: 'payments@financechannel.com',
      joinedAt: '2024-12-05T00:00:00Z',
      lastActive: '2024-12-05T12:00:00Z',
      approvedAt: null,
      notes: 'New application, requires review'
    },
    {
      id: 4,
      name: 'API Developer Community',
      email: 'community@apidevs.org',
      affiliateCode: 'APIDEVS',
      status: 'active',
      commissionRate: 18,
      tier: 'Premium',
      totalReferrals: 187,
      totalSales: 18650.25,
      totalCommissions: 3357.05,
      pendingCommissions: 275.80,
      paidCommissions: 3081.25,
      conversionRate: 28.4,
      avgOrderValue: 199.75,
      paymentMethod: 'Cryptocurrency',
      paymentEmail: null,
      joinedAt: '2024-05-10T00:00:00Z',
      lastActive: '2024-12-07T09:20:00Z',
      approvedAt: '2024-05-11T11:30:00Z',
      notes: 'Developer community with high conversion rates'
    },
    {
      id: 5,
      name: 'Investment Podcast Network',
      email: 'host@investmentpodcast.fm',
      affiliateCode: 'INVESTPOD',
      status: 'inactive',
      commissionRate: 12,
      tier: 'Standard',
      totalReferrals: 89,
      totalSales: 4235.80,
      totalCommissions: 508.30,
      pendingCommissions: 0,
      paidCommissions: 508.30,
      conversionRate: 8.2,
      avgOrderValue: 47.60,
      paymentMethod: 'PayPal',
      paymentEmail: 'payments@investmentpodcast.fm',
      joinedAt: '2024-07-22T00:00:00Z',
      lastActive: '2024-11-15T16:45:00Z',
      approvedAt: '2024-07-23T14:20:00Z',
      notes: 'Inactive for 3+ weeks, needs follow-up'
    },
    {
      id: 6,
      name: 'Trading Academy Pro',
      email: 'partners@tradingacademy.pro',
      affiliateCode: 'TRADEPRO',
      status: 'active',
      commissionRate: 22,
      tier: 'Premium',
      totalReferrals: 334,
      totalSales: 41250.90,
      totalCommissions: 9075.20,
      pendingCommissions: 850.40,
      paidCommissions: 8224.80,
      conversionRate: 19.7,
      avgOrderValue: 123.50,
      paymentMethod: 'Bank Transfer',
      paymentEmail: null,
      joinedAt: '2024-02-08T00:00:00Z',
      lastActive: '2024-12-09T11:45:00Z',
      approvedAt: '2024-02-09T10:15:00Z',
      notes: 'Consistent performer with trading education audience'
    }
  ]

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <AffiliateManagement
        affiliates={demoAffiliates}
        currentUser={session.user}
      />
    </div>
  )
}

/**
 *
 */
export default function AdminAffiliatesPage() {
  return (
    <Suspense fallback={<AffiliatesHubSkeleton />}>
      <AffiliatesDataWrapper />
    </Suspense>
  )
}