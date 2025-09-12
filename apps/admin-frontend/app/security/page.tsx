import { Metadata } from 'next';
import SecurityDashboard from '@/components/security/SecurityDashboard';

export const metadata: Metadata = {
  title: 'Security Monitoring | EPSX Admin',
  description: 'Real-time security monitoring and threat detection dashboard',
};

export default function SecurityPage() {
  return <SecurityDashboard />;
}