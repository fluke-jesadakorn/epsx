import React from 'react'
import { Settings, Server, Shield, Database, Wifi, AlertCircle, CheckCircle, Clock } from 'lucide-react'
import { ServerSystemAPI } from '@/lib/api/server-admin-api'

/**
 * Windows Phone-style System Hub
 * System configuration, feature flags, and monitoring
 */

function SystemStatusCard({ title, status, details, icon: Icon, color }: {
  title: string
  status: 'healthy' | 'warning' | 'error' | 'unknown'
  details: string
  icon: any
  color: string
}) {
  const statusColors = {
    healthy: 'text-green-600',
    warning: 'text-orange-600', 
    error: 'text-red-600',
    unknown: 'text-gray-600'
  }

  const statusIcons = {
    healthy: '🟢',
    warning: '🟡',
    error: '🔴',
    unknown: '⚪'
  }

  return (
    <div className={`${color} text-white p-4 rounded-lg`}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <h3 className="text-sm font-medium opacity-90 flex items-center gap-2">
            <Icon size={16} />
            {title}
          </h3>
          <div className="flex items-center gap-2 mt-2">
            <span className="text-lg">{statusIcons[status]}</span>
            <div>
              <p className="text-lg font-bold capitalize">{status}</p>
              <p className="text-xs opacity-75">{details}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function FeatureFlagCard({ flag, enabled, description }: {
  flag: string
  enabled: boolean
  description: string
}) {
  return (
    <div className="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 transition-colors">
      <div className="flex-1">
        <div className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${enabled ? 'bg-green-500' : 'bg-gray-400'}`}></div>
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white">
              {flag.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
            </h4>
            <p className="text-sm text-gray-600 dark:text-gray-400">{description}</p>
          </div>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <span className={`text-sm font-medium ${enabled ? 'text-green-600' : 'text-gray-500'}`}>
          {enabled ? 'ON' : 'OFF'}
        </span>
        <button 
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
            enabled ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-600'
          }`}
        >
          <span 
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              enabled ? 'translate-x-6' : 'translate-x-1'
            }`} 
          />
        </button>
      </div>
    </div>
  )
}

function ConfigurationSection({ title, config, icon: Icon }: {
  title: string
  config: Record<string, any>
  icon: any
}) {
  return (
    <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center gap-2 mb-4">
        <Icon className="text-blue-600" size={20} />
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{title}</h3>
      </div>
      
      <div className="space-y-3">
        {Object.entries(config).map(([key, value]) => (
          <div key={key} className="flex items-center justify-between">
            <span className="text-gray-600 dark:text-gray-400 capitalize">
              {key.replace(/_/g, ' ')}:
            </span>
            <div className="flex items-center gap-2">
              {typeof value === 'boolean' ? (
                <>
                  <span className={`font-medium ${value ? 'text-green-600' : 'text-red-600'}`}>
                    {value ? 'Configured' : 'Not Configured'}
                  </span>
                  {value ? <CheckCircle size={16} className="text-green-600" /> : <AlertCircle size={16} className="text-red-600" />}
                </>
              ) : (
                <span className="font-medium text-gray-900 dark:text-white">
                  {value?.toString() || 'N/A'}
                </span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

export default async function SystemHub() {
  // Fetch system configuration and feature flags
  const [systemConfig, featureFlags] = await Promise.allSettled([
    ServerSystemAPI.getSystemConfig(),
    ServerSystemAPI.getFeatureFlags()
  ])

  const config = systemConfig.status === 'fulfilled' ? systemConfig.value : {
    jwt_secret_configured: false,
    api_base_url: 'localhost:8080',
    smtp_configured: false,
    oauth_configured: false
  }

  const flags = featureFlags.status === 'fulfilled' ? featureFlags.value : {
    eps_analytics: false,
    realtime_updates: false,
    notifications: false,
    security_alerts: false,
    data_export: false
  }

  // Mock some additional system metrics
  const systemMetrics = {
    uptime: '99.9%',
    cpu_usage: '23%',
    memory_usage: '67%',
    disk_usage: '45%',
    active_connections: 234,
    database_connections: 12,
    redis_connections: 8
  }

  const featureFlagDescriptions: Record<string, string> = {
    eps_analytics: 'Enable EPS growth analytics and TradingView integration',
    realtime_updates: 'Enable real-time data updates via Server-Sent Events',
    notifications: 'Enable multi-channel notification system',
    security_alerts: 'Enable ML-powered security monitoring and alerts',
    data_export: 'Enable data export functionality (CSV, JSON, etc.)'
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-4xl font-light text-gray-900 dark:text-white mb-2">
          ⚙️ SYSTEM HUB
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          System configuration, feature flags, and monitoring
        </p>
      </div>

      {/* System Status Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        <SystemStatusCard
          title="API Server"
          status="healthy"
          details={`${config.api_base_url} • ${systemMetrics.uptime} uptime`}
          icon={Server}
          color="bg-green-500"
        />
        
        <SystemStatusCard
          title="Database"
          status="healthy"
          details={`${systemMetrics.database_connections} active connections`}
          icon={Database}
          color="bg-blue-500"
        />
        
        <SystemStatusCard
          title="Authentication"
          status={config.jwt_secret_configured && config.oauth_configured ? "healthy" : "warning"}
          details={`JWT: ${config.jwt_secret_configured ? 'OK' : 'Missing'} • OAuth: ${config.oauth_configured ? 'OK' : 'Missing'}`}
          icon={Shield}
          color={config.jwt_secret_configured && config.oauth_configured ? "bg-green-500" : "bg-orange-500"}
        />
        
        <SystemStatusCard
          title="SMTP"
          status={config.smtp_configured ? "healthy" : "warning"}
          details={config.smtp_configured ? 'Email delivery enabled' : 'Email not configured'}
          icon={Wifi}
          color={config.smtp_configured ? "bg-green-500" : "bg-orange-500"}
        />
      </div>

      {/* Pivot Navigation */}
      <div className="mb-6">
        <div className="flex overflow-x-auto gap-1 border-b border-gray-200 dark:border-gray-700">
          <button className="px-4 py-3 font-medium text-blue-600 border-b-2 border-blue-600 whitespace-nowrap">
            ◄ OVERVIEW ►
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            CONFIG
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            FEATURES
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            MONITORING
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            LOGS
          </button>
        </div>
      </div>

      {/* System Performance Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <Settings className="text-blue-600" size={20} />
            🖥️ System Resources
          </h3>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between items-center mb-1">
                <span className="text-sm text-gray-600 dark:text-gray-400">CPU Usage</span>
                <span className="text-sm font-medium text-gray-900 dark:text-white">{systemMetrics.cpu_usage}</span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                <div className="bg-blue-500 h-2 rounded-full" style={{ width: systemMetrics.cpu_usage }}></div>
              </div>
            </div>
            
            <div>
              <div className="flex justify-between items-center mb-1">
                <span className="text-sm text-gray-600 dark:text-gray-400">Memory</span>
                <span className="text-sm font-medium text-gray-900 dark:text-white">{systemMetrics.memory_usage}</span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                <div className="bg-green-500 h-2 rounded-full" style={{ width: systemMetrics.memory_usage }}></div>
              </div>
            </div>
            
            <div>
              <div className="flex justify-between items-center mb-1">
                <span className="text-sm text-gray-600 dark:text-gray-400">Disk</span>
                <span className="text-sm font-medium text-gray-900 dark:text-white">{systemMetrics.disk_usage}</span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                <div className="bg-orange-500 h-2 rounded-full" style={{ width: systemMetrics.disk_usage }}></div>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <Wifi className="text-green-600" size={20} />
            🔗 Connections
          </h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Active Users:</span>
              <span className="font-medium text-blue-600">{systemMetrics.active_connections}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Database:</span>
              <span className="font-medium text-green-600">{systemMetrics.database_connections}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Redis Cache:</span>
              <span className="font-medium text-purple-600">{systemMetrics.redis_connections}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Uptime:</span>
              <span className="font-medium text-green-600">{systemMetrics.uptime}</span>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <Clock className="text-orange-600" size={20} />
            📊 Quick Stats
          </h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">API Requests/min:</span>
              <span className="font-medium text-blue-600">450</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Error Rate:</span>
              <span className="font-medium text-red-600">0.1%</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Cache Hit Rate:</span>
              <span className="font-medium text-green-600">94.2%</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Avg Response:</span>
              <span className="font-medium text-green-600">1.2s</span>
            </div>
          </div>
        </div>
      </div>

      {/* Configuration Sections */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6 mb-8">
        <ConfigurationSection
          title="🔐 Security Configuration"
          config={{
            jwt_secret_configured: config.jwt_secret_configured,
            oauth_configured: config.oauth_configured,
            api_base_url: config.api_base_url,
            rate_limiting: true,
            cors_enabled: true
          }}
          icon={Shield}
        />
        
        <ConfigurationSection
          title="📧 Communication Setup"
          config={{
            smtp_configured: config.smtp_configured,
            email_notifications: config.smtp_configured,
            webhook_support: true,
            push_notifications: false,
            sms_gateway: false
          }}
          icon={Wifi}
        />
      </div>

      {/* Feature Flags */}
      <div className="mb-8">
        <div className="flex items-center gap-2 mb-4">
          <Settings className="text-blue-600" size={20} />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            🎛️ Feature Flags
          </h2>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {Object.entries(flags).map(([flag, enabled]) => (
            <FeatureFlagCard
              key={flag}
              flag={flag}
              enabled={enabled}
              description={featureFlagDescriptions[flag] || 'Feature flag configuration'}
            />
          ))}
        </div>
      </div>

      {/* System Actions */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <button className="p-4 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-left">
          <div className="flex items-center gap-2 mb-2">
            <Settings size={20} />
            <span className="font-medium">⚙️ Configuration</span>
          </div>
          <p className="text-sm opacity-90">Update system settings</p>
        </button>
        
        <button className="p-4 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors text-left">
          <div className="flex items-center gap-2 mb-2">
            <Server size={20} />
            <span className="font-medium">🔄 Restart Services</span>
          </div>
          <p className="text-sm opacity-90">Restart backend services</p>
        </button>
        
        <button className="p-4 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors text-left">
          <div className="flex items-center gap-2 mb-2">
            <Database size={20} />
            <span className="font-medium">🗃️ Database</span>
          </div>
          <p className="text-sm opacity-90">Manage database</p>
        </button>
        
        <button className="p-4 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors text-left">
          <div className="flex items-center gap-2 mb-2">
            <AlertCircle size={20} />
            <span className="font-medium">📋 System Logs</span>
          </div>
          <p className="text-sm opacity-90">View system logs</p>
        </button>
      </div>
    </div>
  )
}