'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';
import { 
  Shield, 
  AlertTriangle, 
  Lock, 
  Bell, 
  Gauge, 
  Zap, 
  Eye,
  Map,
  Activity,
  FileText,
  Settings,
  ChevronLeft,
  ChevronRight,
  Target,
  Users
} from 'lucide-react';

interface SecuritySidebarProps {
  open: boolean;
  onToggle: () => void;
  currentPath: string;
  alertCount: number;
  systemStatus: 'normal' | 'warning' | 'critical';
}

interface SecurityNavItem {
  href: string;
  label: string;
  icon: React.ComponentType<any>;
  badge?: number;
  description: string;
  children?: SecurityNavItem[];
}

const securityNavItems: SecurityNavItem[] = [
  {
    href: '/security',
    label: 'Overview',
    icon: Shield,
    description: 'Security dashboard overview and system status'
  },
  {
    href: '/security/threats',
    label: 'Threat Intelligence',
    icon: Target,
    description: 'Geographic threats, IP reputation, and attack patterns',
    children: [
      {
        href: '/security/threats/map',
        label: 'Threat Map',
        icon: Map,
        description: 'Geographic visualization of threats'
      },
      {
        href: '/security/threats/patterns',
        label: 'Attack Patterns',
        icon: Activity,
        description: 'Attack pattern analysis and trends'
      },
      {
        href: '/security/threats/reputation',
        label: 'IP Reputation',
        icon: Target,
        description: 'IP reputation database and scoring'
      }
    ]
  },
  {
    href: '/security/access',
    label: 'Access Control',
    icon: Lock,
    description: 'Permission validation and access monitoring',
    children: [
      {
        href: '/security/access/permissions',
        label: 'Permissions',
        icon: Lock,
        description: 'Permission validation logs'
      },
      {
        href: '/security/access/sessions',
        label: 'Sessions',
        icon: Users,
        description: 'Active session monitoring'
      },
      {
        href: '/security/access/failed',
        label: 'Failed Attempts',
        icon: AlertTriangle,
        description: 'Failed access attempts'
      }
    ]
  },
  {
    href: '/security/alerts',
    label: 'Alert Management',
    icon: Bell,
    description: 'Security alerts and notification management',
    children: [
      {
        href: '/security/alerts/active',
        label: 'Active Alerts',
        icon: Bell,
        description: 'Currently active security alerts'
      },
      {
        href: '/security/alerts/rules',
        label: 'Alert Rules',
        icon: Settings,
        description: 'Configure alert rules and thresholds'
      },
      {
        href: '/security/alerts/webhooks',
        label: 'Webhooks',
        icon: Zap,
        description: 'Webhook endpoint management'
      }
    ]
  },
  {
    href: '/security/performance',
    label: 'Performance',
    icon: Gauge,
    description: 'Security system performance monitoring',
    children: [
      {
        href: '/security/performance/metrics',
        label: 'Metrics',
        icon: Gauge,
        description: 'Performance metrics dashboard'
      },
      {
        href: '/security/performance/capacity',
        label: 'Capacity',
        icon: Activity,
        description: 'Capacity planning and utilization'
      }
    ]
  },
  {
    href: '/security/incidents',
    label: 'Incident Response',
    icon: Zap,
    description: 'Security incident management and response',
    children: [
      {
        href: '/security/incidents/timeline',
        label: 'Timeline',
        icon: Activity,
        description: 'Security incident timeline'
      },
      {
        href: '/security/incidents/investigation',
        label: 'Investigation',
        icon: Eye,
        description: 'Investigation tools and forensics'
      },
      {
        href: '/security/incidents/reports',
        label: 'Reports',
        icon: FileText,
        description: 'Incident reports and documentation'
      }
    ]
  }
];

export function SecuritySidebar({ 
  open, 
  onToggle, 
  currentPath, 
  alertCount, 
  systemStatus 
}: SecuritySidebarProps) {
  const pathname = usePathname();

  const isActiveRoute = (href: string) => {
    if (href === '/security' && pathname === '/security') return true;
    if (href !== '/security' && pathname.startsWith(href)) return true;
    return false;
  };

  const getStatusColor = () => {
    switch (systemStatus) {
      case 'critical': return 'text-red-500';
      case 'warning': return 'text-yellow-500';
      default: return 'text-green-500';
    }
  };

  return (
    <>
      <div className={cn(
        "bg-card border-r border-border transition-all duration-300 relative z-50",
        open ? "w-80" : "w-16",
        "md:relative fixed inset-y-0 left-0"
      )}>
        {/* Sidebar Header */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          {open && (
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-gradient-to-r from-red-500 to-orange-500">
                <Shield className="w-6 h-6 text-white" />
              </div>
              <div>
                <h2 className="font-bold text-lg">Security</h2>
                <p className="text-xs text-muted-foreground">Monitoring Dashboard</p>
              </div>
            </div>
          )}
          
          <button
            onClick={onToggle}
            className="flex items-center justify-center w-8 h-8 rounded-lg hover:bg-muted transition-colors"
          >
            {open ? (
              <ChevronLeft className="w-4 h-4" />
            ) : (
              <ChevronRight className="w-4 h-4" />
            )}
          </button>
        </div>

        {/* System Status Indicator */}
        <div className="p-4 border-b border-border">
          <div className={cn(
            "flex items-center gap-3 p-3 rounded-lg border transition-all",
            {
              'bg-green-500/10 border-green-500/20': systemStatus === 'normal',
              'bg-yellow-500/10 border-yellow-500/20': systemStatus === 'warning',
              'bg-red-500/10 border-red-500/20 animate-pulse': systemStatus === 'critical',
            }
          )}>
            <Activity className={cn("w-5 h-5", getStatusColor())} />
            {open && (
              <div className="flex-1">
                <p className="font-medium text-sm capitalize">{systemStatus}</p>
                <p className="text-xs text-muted-foreground">
                  {alertCount > 0 ? `${alertCount} alerts` : 'All systems operational'}
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Navigation Items */}
        <nav className="p-2 space-y-1 overflow-y-auto flex-1">
          {securityNavItems.map((item) => {
            const isActive = isActiveRoute(item.href);
            const hasChildren = item.children && item.children.length > 0;
            const showChildren = hasChildren && (isActive || pathname.startsWith(item.href));

            return (
              <div key={item.href}>
                <Link
                  href={item.href}
                  className={cn(
                    "flex items-center gap-3 px-3 py-2 rounded-lg transition-all group relative",
                    isActive
                      ? "bg-primary text-primary-foreground shadow-md"
                      : "hover:bg-muted/50 hover:text-foreground"
                  )}
                  title={!open ? item.description : undefined}
                >
                  <item.icon className={cn(
                    "w-5 h-5 flex-shrink-0 transition-transform group-hover:scale-110",
                    isActive ? "text-primary-foreground" : ""
                  )} />
                  
                  {open && (
                    <>
                      <span className="flex-1 font-medium">{item.label}</span>
                      {item.badge && (
                        <span className="bg-red-500 text-white text-xs rounded-full px-2 py-0.5 min-w-[20px] text-center">
                          {item.badge}
                        </span>
                      )}
                    </>
                  )}
                  
                  {!open && isActive && (
                    <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-xs rounded shadow-md border opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap">
                      {item.label}
                    </div>
                  )}
                </Link>

                {/* Sub-navigation */}
                {open && showChildren && item.children && (
                  <div className="ml-6 mt-1 space-y-1">
                    {item.children.map((child) => {
                      const isChildActive = pathname === child.href;
                      return (
                        <Link
                          key={child.href}
                          href={child.href}
                          className={cn(
                            "flex items-center gap-3 px-3 py-1.5 rounded-md transition-all text-sm",
                            isChildActive
                              ? "bg-primary/10 text-primary font-medium"
                              : "hover:bg-muted/50 text-muted-foreground hover:text-foreground"
                          )}
                        >
                          <child.icon className="w-4 h-4 flex-shrink-0" />
                          <span>{child.label}</span>
                        </Link>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
        </nav>

        {/* Emergency Actions */}
        {open && (
          <div className="p-4 border-t border-border">
            <div className="space-y-2">
              <button className="w-full px-3 py-2 bg-red-500 text-white rounded-lg text-sm font-medium hover:bg-red-600 transition-colors">
                Emergency Lockdown
              </button>
              <button className="w-full px-3 py-2 bg-yellow-500 text-white rounded-lg text-sm font-medium hover:bg-yellow-600 transition-colors">
                System Maintenance
              </button>
            </div>
          </div>
        )}
      </div>
    </>
  );
}