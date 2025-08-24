import { SecurityDashboardLayout } from '@/components/security/SecurityDashboardLayout';
import { IncidentResponse } from '@/components/security/IncidentResponse';

export const metadata = {
  title: 'Incident Response - EPSX Security',
  description: 'Security incident management, investigation tools, and response coordination',
};

export default function IncidentResponsePage() {
  return (
    <SecurityDashboardLayout>
      <IncidentResponse />
    </SecurityDashboardLayout>
  );
}