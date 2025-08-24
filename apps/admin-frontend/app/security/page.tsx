import { SecurityDashboardLayout } from '@/components/security/SecurityDashboardLayout';
import { SecurityOverview } from '@/components/security/SecurityOverview';

export const metadata = {
  title: 'Security Dashboard - EPSX Admin',
  description: 'Comprehensive security monitoring and threat detection dashboard',
};

export default function SecurityPage() {
  return (
    <SecurityDashboardLayout>
      <SecurityOverview />
    </SecurityDashboardLayout>
  );
}