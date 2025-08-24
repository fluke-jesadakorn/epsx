import { SecurityDashboardLayout } from '@/components/security/SecurityDashboardLayout';
import { AccessControlMonitor } from '@/components/security/AccessControlMonitor';

export const metadata = {
  title: 'Access Control Monitor - EPSX Security',
  description: 'Permission validation, session monitoring, and access control analytics',
};

export default function AccessControlPage() {
  return (
    <SecurityDashboardLayout>
      <AccessControlMonitor />
    </SecurityDashboardLayout>
  );
}