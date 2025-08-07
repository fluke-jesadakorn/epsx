/**
 * System Health Card Component
 * Shows system performance metrics
 */

import { Cpu, Database, Zap, Activity } from 'lucide-react'
import type { SystemMetrics } from '@/lib/data/dashboard'

interface SystemHealthCardProps {
  metrics: SystemMetrics
}

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
    <div className="pancake-card pancake-card-hover p-6">
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
                item.status === 'good' ? 'bg-green-500' :
                item.status === 'warning' ? 'bg-yellow-500' :
                'bg-red-500'
              }`} />
            </div>
          </div>
        ))}
        
        {/* Error Rate */}
        <div className="pt-2 border-t border-muted">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">Error Rate</span>
            <span className={`text-sm font-medium ${
              metrics.errorRate < 1 ? 'text-green-600' :
              metrics.errorRate < 5 ? 'text-yellow-600' :
              'text-red-600'
            }`}>
              {metrics.errorRate}%
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}