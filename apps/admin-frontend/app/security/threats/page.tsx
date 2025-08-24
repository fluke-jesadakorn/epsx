import { SecurityDashboardLayout } from '@/components/security/SecurityDashboardLayout';
import { ThreatIntelligence } from '@/components/security/ThreatIntelligence';

export const metadata = {
  title: 'Threat Intelligence - EPSX Security',
  description: 'Geographic threats, IP reputation, and attack pattern analysis',
};

export default function ThreatIntelligencePage() {
  return (
    <SecurityDashboardLayout>
      <ThreatIntelligence />
    </SecurityDashboardLayout>
  );
}