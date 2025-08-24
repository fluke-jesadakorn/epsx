'use client';

import { useState, useEffect } from 'react';
import { usePathname } from 'next/navigation';
import { SecuritySidebar } from './SecuritySidebar';
import { SecurityHeader } from './SecurityHeader';
import { SecurityFooter } from './SecurityFooter';
import { cn } from '@/lib/utils';
import { useAuth } from '@/lib/auth';
import { AlertTriangle, Shield, Activity } from 'lucide-react';

interface SecurityDashboardLayoutProps {
  children: React.ReactNode;
  className?: string;
}

export function SecurityDashboardLayout({ children, className }: SecurityDashboardLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [criticalAlerts, setCriticalAlerts] = useState(0);
  const [systemStatus, setSystemStatus] = useState<'normal' | 'warning' | 'critical'>('normal');
  const pathname = usePathname();
  const { user } = useAuth();

  // Check if user has required security permissions
  const hasSecurityAccess = user?.admin_modules?.some(module => 
    ['security_management', 'audit_logs'].includes(module)
  ) ?? false;

  // Simulate real-time security status updates
  useEffect(() => {
    const interval = setInterval(() => {
      // This would be replaced with real API calls
      const randomAlerts = Math.floor(Math.random() * 5);
      setCriticalAlerts(randomAlerts);
      
      if (randomAlerts > 3) {
        setSystemStatus('critical');
      } else if (randomAlerts > 1) {
        setSystemStatus('warning');
      } else {
        setSystemStatus('normal');
      }
    }, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, []);

  if (!hasSecurityAccess) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center p-8 max-w-md">
          <Shield className="w-16 h-16 mx-auto mb-4 text-muted-foreground" />
          <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
          <p className="text-muted-foreground mb-6">
            You need security management or audit logs permissions to access the security dashboard.
          </p>
          <div className="bg-muted/50 p-4 rounded-lg">
            <h3 className="font-semibold mb-2">Required Permissions:</h3>
            <ul className="text-sm text-muted-foreground space-y-1">
              <li>• security_management</li>
              <li>• audit_logs</li>
            </ul>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Emergency Alert Banner */}
      {systemStatus === 'critical' && (
        <div className="bg-red-500 text-white px-4 py-2 text-center text-sm font-medium animate-pulse">
          <AlertTriangle className="w-4 h-4 inline-block mr-2" />
          Critical Security Alert: {criticalAlerts} active threats detected
        </div>
      )}

      <div className="flex h-screen">
        {/* Security Sidebar */}
        <SecuritySidebar 
          open={sidebarOpen}
          onToggle={() => setSidebarOpen(!sidebarOpen)}
          currentPath={pathname}
          alertCount={criticalAlerts}
          systemStatus={systemStatus}
        />

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Security Header */}
          <SecurityHeader 
            onSidebarToggle={() => setSidebarOpen(!sidebarOpen)}
            criticalAlerts={criticalAlerts}
            systemStatus={systemStatus}
          />

          {/* Main Content */}
          <main className={cn(
            "flex-1 overflow-auto bg-muted/20 p-6",
            className
          )}>
            <div className="max-w-7xl mx-auto">
              {children}
            </div>
          </main>

          {/* Security Footer */}
          <SecurityFooter systemStatus={systemStatus} />
        </div>
      </div>

      {/* Mobile Sidebar Overlay */}
      {sidebarOpen && (
        <div 
          className="fixed inset-0 bg-black/50 backdrop-blur-sm z-40 md:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* System Status Indicator */}
      <div className="fixed bottom-4 right-4 z-50">
        <div className={cn(
          "flex items-center gap-2 px-3 py-2 rounded-full text-sm font-medium shadow-lg backdrop-blur-sm border transition-all",
          {
            'bg-green-500/10 border-green-500/20 text-green-700 dark:text-green-300': systemStatus === 'normal',
            'bg-yellow-500/10 border-yellow-500/20 text-yellow-700 dark:text-yellow-300': systemStatus === 'warning',
            'bg-red-500/10 border-red-500/20 text-red-700 dark:text-red-300 animate-pulse': systemStatus === 'critical',
          }
        )}>
          <Activity className={cn(
            "w-4 h-4",
            {
              'text-green-500': systemStatus === 'normal',
              'text-yellow-500': systemStatus === 'warning',
              'text-red-500': systemStatus === 'critical',
            }
          )} />
          <span className="capitalize">{systemStatus}</span>
        </div>
      </div>
    </div>
  );
}