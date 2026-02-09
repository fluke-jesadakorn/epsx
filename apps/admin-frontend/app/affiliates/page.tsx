import { notFound } from 'next/navigation';
import { Suspense } from 'react';

import { AffiliateManagement, type Affiliate } from '@/components/affiliates/affiliate-management';
import { PageLayout, PageSkeleton } from '@/components/shared';
import { UnifiedAuth } from '@/lib/auth/auth';

export const dynamic = 'force-dynamic';

// eslint-disable-next-line max-lines-per-function
async function AffiliatesDataWrapper(): Promise<React.JSX.Element> {
  const session = await UnifiedAuth.getSession();

  if (!session.user) {
    notFound();
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
  ];

  return <AffiliateManagement affiliates={demoAffiliates} currentUser={session.user} />;
}

/**
 * Affiliates Page
 * Uses unified page components for consistent design
 */
export default function AdminAffiliatesPage(): React.JSX.Element {
  return (
    <PageLayout>
      <Suspense fallback={<PageSkeleton stats={4} rows={6} />}>
        <AffiliatesDataWrapper />
      </Suspense>
    </PageLayout>
  );
}
