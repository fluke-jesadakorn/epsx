import { Metadata } from 'next';
import { BillingDashboard } from '@/components/admin/BillingDashboard';

export const metadata: Metadata = {
  title: 'Billing & Usage Analytics | Admin',
  description: 'Monitor module usage, billing, and analytics for the EPSX platform',
};

export default function BillingPage() {
  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <BillingDashboard />
    </div>
  );
}