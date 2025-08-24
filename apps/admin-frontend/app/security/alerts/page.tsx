import { SecurityDashboardLayout } from '@/components/security/SecurityDashboardLayout';
import { AlertManagement } from '@/components/security/AlertManagement';

export const metadata = {
  title: 'Alert Management - EPSX Security',
  description: 'Security alerts, notification management, and webhook configuration',
};

export default function AlertManagementPage() {
  return (
    <SecurityDashboardLayout>
      <AlertManagement />
    </SecurityDashboardLayout>
  );
}