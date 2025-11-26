/**
 * System Health Card Component
 * Shows system performance metrics
 */

import { Cpu, Database, Zap, Activity } from 'lucide-react'

import { adminCardVariants } from '@/design-system'
import { cn } from '@/lib/utils'

interface SystemMetrics {
  serverLoad: number
  memoryUsage: number
  databaseConnections: number
  errorRate: number
  uptime: number
}

interface SystemHealthCardProps {
  metrics: SystemMetrics
}

/**
 *
 * @param root0
 * @param root0.metrics
 */
export function SystemHealthCard({ metrics }: SystemHealthCardProps) {
  const healthItems = [
    {
      label: 'Server Load',
      value: `${metrics.serverLoad}%`,
      icon: Cpu,
      status: metrics.serverLoad < 70 ? 'good' : metrics.serverLoad < 90 ? 'warning' : 'critical'
    },
    {
      label: 'Memory Usage',
      value: `${metrics.memoryUsage}%`,
      icon: Activity,
      status: metrics.memoryUsage < 80 ? 'good' : metrics.memoryUsage < 95 ? 'warning' : 'critical'
    },
    {
      label: 'DB Connections',
      value: metrics.databaseConnections,
      icon: Database,
      status: metrics.databaseConnections < 50 ? 'good' : metrics.databaseConnections < 80 ? 'warning' : 'critical'
    },
    {
      label: 'Uptime',
      value: `${metrics.uptime}%`,
      icon: Zap,
      status: metrics.uptime > 99.5 ? 'good' : metrics.uptime > 99 ? 'warning' : 'critical'
    }
  ]

  return (
    <div className={cn(adminCardVariants({ variant: 'pancake', hover: 'both' }))}>
      <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
        <Activity className="h-5 w-5" />
        System Health
      </h2>
      <div className="space-y-4">
        {healthItems.map((item, index) => (
          <div key={index} className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <item.icon className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm">{item.label}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">{item.value}</span>
              <div className={`w-2 h-2 rounded-full ${
                item.status === 'good' ? 'bg-success-500' :
                item.status === 'warning' ? 'bg-warning-500' :
                'bg-error-500'
              }`} />
            </div>
          </div>
        ))}
        
        {/* Error Rate */}
        <div className="pt-2 border-t border-muted">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">Error Rate</span>
            <span className={`text-sm font-medium ${
              metrics.errorRate < 1 ? 'text-success-600' :
              metrics.errorRate < 5 ? 'text-warning-600' :
              'text-error-600'
            }`}>
              {metrics.errorRate}%
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}