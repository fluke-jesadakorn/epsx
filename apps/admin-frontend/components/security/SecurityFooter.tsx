'use client';

import { useState, useEffect } from 'react';
import { cn } from '@/lib/utils';
import { 
  Activity, 
  Cpu, 
  HardDrive, 
  Network, 
  Clock,
  AlertCircle,
  CheckCircle,
  Info
} from 'lucide-react';

interface SecurityFooterProps {
  systemStatus: 'normal' | 'warning' | 'critical';
}

interface SystemMetric {
  label: string;
  value: number;
  unit: string;
  icon: React.ComponentType<any>;
  threshold: { warning: number; critical: number };
}

export function SecurityFooter({ systemStatus }: SecurityFooterProps) {
  const [currentTime, setCurrentTime] = useState(new Date());
  const [uptime, setUptime] = useState('15d 4h 32m');
  const [systemMetrics, setSystemMetrics] = useState<SystemMetric[]>([
    {
      label: 'CPU Usage',
      value: 45,
      unit: '%',
      icon: Cpu,
      threshold: { warning: 70, critical: 90 }
    },
    {
      label: 'Memory',
      value: 62,
      unit: '%',
      icon: Activity,
      threshold: { warning: 80, critical: 95 }
    },
    {
      label: 'Disk I/O',
      value: 28,
      unit: '%',
      icon: HardDrive,
      threshold: { warning: 85, critical: 95 }
    },
    {
      label: 'Network',
      value: 34,
      unit: 'Mbps',
      icon: Network,
      threshold: { warning: 80, critical: 100 }
    }
  ]);

  // Update current time every second
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  // Simulate system metrics updates
  useEffect(() => {
    const interval = setInterval(() => {
      setSystemMetrics(prev => prev.map(metric => ({
        ...metric,
        value: Math.max(0, Math.min(100, metric.value + (Math.random() - 0.5) * 10))
      })));
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const getMetricStatus = (value: number, threshold: { warning: number; critical: number }) => {
    if (value >= threshold.critical) return 'critical';
    if (value >= threshold.warning) return 'warning';
    return 'normal';
  };

  const getMetricColor = (status: 'normal' | 'warning' | 'critical') => {
    switch (status) {
      case 'critical': return 'text-red-500';
      case 'warning': return 'text-yellow-500';
      default: return 'text-green-500';
    }
  };

  const getSystemStatusIcon = () => {
    switch (systemStatus) {
      case 'critical': return <AlertCircle className="w-4 h-4 text-red-500" />;
      case 'warning': return <Info className="w-4 h-4 text-yellow-500" />;
      default: return <CheckCircle className="w-4 h-4 text-green-500" />;
    }
  };

  const getSystemStatusText = () => {
    switch (systemStatus) {
      case 'critical': return 'System requires immediate attention';
      case 'warning': return 'System monitoring active - warnings detected';
      default: return 'All security systems operational';
    }
  };

  return (
    <footer className="bg-card border-t border-border px-6 py-3">
      <div className="flex items-center justify-between">
        {/* Left Section - System Status */}
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            {getSystemStatusIcon()}
            <span className="text-sm font-medium">{getSystemStatusText()}</span>
          </div>
          
          <div className="hidden md:flex items-center gap-1 text-sm text-muted-foreground">
            <Clock className="w-4 h-4" />
            <span>Uptime: {uptime}</span>
          </div>
        </div>

        {/* Center Section - System Metrics */}
        <div className="hidden lg:flex items-center gap-6">
          {systemMetrics.map((metric) => {
            const status = getMetricStatus(metric.value, metric.threshold);
            const MetricIcon = metric.icon;
            
            return (
              <div key={metric.label} className="flex items-center gap-2">
                <MetricIcon className={cn("w-4 h-4", getMetricColor(status))} />
                <div className="text-sm">
                  <span className="font-medium">{metric.value.toFixed(0)}{metric.unit}</span>
                  <span className="text-muted-foreground ml-1">{metric.label}</span>
                </div>
              </div>
            );
          })}
        </div>

        {/* Right Section - Current Time */}
        <div className="flex items-center gap-4">
          <div className="text-sm text-muted-foreground">
            {currentTime.toLocaleString('en-US', {
              timeZone: 'UTC',
              hour12: false,
              year: 'numeric',
              month: '2-digit',
              day: '2-digit',
              hour: '2-digit',
              minute: '2-digit',
              second: '2-digit'
            })} UTC
          </div>
          
          {/* Version Info */}
          <div className="hidden sm:block text-xs text-muted-foreground border-l border-border pl-4">
            EPSX Security v2.1.0
          </div>
        </div>
      </div>

      {/* Mobile System Metrics */}
      <div className="lg:hidden mt-2 pt-2 border-t border-border/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            {systemMetrics.slice(0, 2).map((metric) => {
              const status = getMetricStatus(metric.value, metric.threshold);
              const MetricIcon = metric.icon;
              
              return (
                <div key={metric.label} className="flex items-center gap-1">
                  <MetricIcon className={cn("w-3 h-3", getMetricColor(status))} />
                  <span className="text-xs">
                    {metric.value.toFixed(0)}{metric.unit}
                  </span>
                </div>
              );
            })}
          </div>
          
          <div className="text-xs text-muted-foreground">
            Uptime: {uptime}
          </div>
        </div>
      </div>
    </footer>
  );
}