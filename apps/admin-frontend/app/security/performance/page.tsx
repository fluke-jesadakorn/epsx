import { SecurityDashboardLayout } from '@/components/security/SecurityDashboardLayout';
import { PerformanceMonitor } from '@/components/security/PerformanceMonitor';

export const metadata = {
  title: 'Performance Monitor - EPSX Security',
  description: 'Security system performance metrics, capacity planning, and optimization',
};

export default function SecurityPerformancePage() {
  return (
    <SecurityDashboardLayout>
      <PerformanceMonitor />
    </SecurityDashboardLayout>
  );
}