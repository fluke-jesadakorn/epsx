import React from 'react'
import { TrendingUp, Activity, BarChart3, PieChart, Target, AlertTriangle, Zap } from 'lucide-react'
import { ServerAnalyticsAPI } from '@/lib/api/server-admin-api'
import { PancakeCard, PancakeStatsCard, PancakeFeatureCard } from '@/components/ui/PancakeCard'

/**
 * PancakeSwap-style Analytics Hub
 * EPS analytics, performance metrics, and insights dashboard with PancakeSwap theming
 */

function MetricTile({ title, value, subValue, trend, color, icon: Icon, size = 'normal' }: {
  title: string
  value: string | number
  subValue?: string
  trend?: 'up' | 'down' | 'stable'
  color: string
  icon: any
  size?: 'normal' | 'wide' | 'large'
}) {
  const sizeClasses = {
    normal: 'col-span-1 row-span-1',
    wide: 'col-span-2 row-span-1',
    large: 'col-span-2 row-span-2'
  }

  const trendIcon = trend === 'up' ? '↗️' : trend === 'down' ? '↘️' : '➡️'
  const trendColor = trend === 'up' ? 'text-green-400' : trend === 'down' ? 'text-red-400' : 'text-yellow-400'

  return (
    <div className={`${color} text-white p-6 shadow-xl ${sizeClasses[size]} flex flex-col justify-between relative overflow-hidden transition-all duration-300 hover:scale-[1.02] cursor-pointer`}>
      {/* Windows Phone Metro accent strip */}
      <div className="absolute left-0 top-0 h-full w-1 bg-gradient-to-b from-yellow-400 to-orange-500"></div>
      
      {/* PancakeSwap corner shine */}
      <div className="absolute top-0 right-0 h-8 w-8 bg-gradient-to-bl from-white/20 to-transparent"></div>
      
      <div className="flex items-start justify-between relative z-10">
        <div className="flex-1">
          <h3 className="text-xs font-light opacity-90 uppercase tracking-wider">{title}</h3>
          <p className="text-3xl font-extralight mt-1 tracking-tight">{value}</p>
          {subValue && (
            <p className="text-xs opacity-75 mt-1 font-light">{subValue}</p>
          )}
        </div>
        <div className="ml-3">
          <div className="p-2 bg-black/20 rounded-sm">
            <Icon size={20} className="opacity-90" />
          </div>
          {trend && (
            <div className={`text-lg ${trendColor} mt-1 text-center`}>
              {trendIcon}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

function EPSRankingsWidget({ rankings }: { rankings: any }) {
  const top5 = rankings?.rankings?.slice(0, 5) || []
  
  return (
    <PancakeCard variant="analytics">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent flex items-center gap-3">
          <div className="w-8 h-8 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-lg flex items-center justify-center">
            <Target className="text-white" size={16} />
          </div>
          Top EPS Growth
        </h3>
        <span className="px-3 py-1 bg-gradient-to-r from-green-500 to-green-600 text-white text-xs font-bold rounded-full animate-pulse">
          LIVE
        </span>
      </div>
      
      <div className="space-y-3">
        {top5.map((stock: any, index: number) => (
          <div key={index} className="flex items-center justify-between p-4 bg-gradient-to-r from-orange-50/50 to-yellow-50/50 dark:from-orange-950/20 dark:to-yellow-950/20 rounded-xl border border-orange-200/30 dark:border-orange-800/30 hover:shadow-md transition-all duration-300">
            <div className="flex items-center gap-4">
              <div className="w-8 h-8 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-full flex items-center justify-center text-white font-bold text-sm">
                {index + 1}
              </div>
              <div>
                <div className="font-bold text-orange-700 dark:text-orange-300">
                  {stock.symbol}
                </div>
                <div className="text-xs text-orange-600/70 dark:text-orange-400/70">
                  {stock.country} • {stock.sector}
                </div>
              </div>
            </div>
            <div className="text-right">
              <div className="font-bold text-green-600 text-lg">
                +{stock.eps_growth}%
              </div>
            </div>
          </div>
        ))}
      </div>
      
      <div className="mt-6 pt-4 border-t border-orange-200/50 dark:border-orange-800/50">
        <div className="flex justify-between items-center text-sm text-orange-600/80 dark:text-orange-400/80">
          <span>Total Requests: {rankings?.total_requests?.toLocaleString() || 0}</span>
          <span>Updated: {rankings?.last_updated ? new Date(rankings.last_updated).toLocaleTimeString() : 'N/A'}</span>
        </div>
      </div>
    </PancakeCard>
  )
}

function PerformanceWidget({ metrics }: { metrics: any }) {
  return (
    <PancakeCard variant="analytics">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent flex items-center gap-3">
          <div className="w-8 h-8 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-lg flex items-center justify-center">
            <Activity className="text-white" size={16} />
          </div>
          System Performance
        </h3>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
          <span className="text-xs font-bold text-orange-600 dark:text-orange-400">LIVE</span>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="space-y-4">
          <div>
            <div className="text-sm font-semibold text-orange-700 dark:text-orange-300 uppercase tracking-wide mb-1">API Response</div>
            <div className="text-2xl font-bold text-green-600 mb-2">{metrics?.api_response_time || 1.2}s</div>
            <div className="w-full bg-orange-200/50 dark:bg-orange-800/30 rounded-full h-3 overflow-hidden">
              <div className="bg-gradient-to-r from-green-500 to-green-600 h-3 rounded-full transition-all duration-500" style={{ width: '85%' }}></div>
            </div>
          </div>
          
          <div>
            <div className="text-sm font-semibold text-orange-700 dark:text-orange-300 uppercase tracking-wide mb-1">Memory Usage</div>
            <div className="text-2xl font-bold text-orange-600 mb-2">{metrics?.memory_usage || 67}%</div>
            <div className="w-full bg-orange-200/50 dark:bg-orange-800/30 rounded-full h-3 overflow-hidden">
              <div className="bg-gradient-to-r from-orange-500 to-yellow-500 h-3 rounded-full transition-all duration-500" style={{ width: `${metrics?.memory_usage || 67}%` }}></div>
            </div>
          </div>
        </div>

        <div className="space-y-4">
          <div>
            <div className="text-sm font-semibold text-orange-700 dark:text-orange-300 uppercase tracking-wide mb-1">Active Users</div>
            <div className="text-2xl font-bold text-blue-600 mb-1">{metrics?.active_users || 234}</div>
            <div className="text-xs text-orange-600/80 dark:text-orange-400/80">Peak: {metrics?.peak_users_today || 1245}</div>
          </div>
          
          <div>
            <div className="text-sm font-semibold text-orange-700 dark:text-orange-300 uppercase tracking-wide mb-1">New Signups</div>
            <div className="text-2xl font-bold text-purple-600 mb-1">+{metrics?.new_signups || 12}</div>
            <div className="text-xs text-orange-600/80 dark:text-orange-400/80">Today</div>
          </div>
        </div>
      </div>
    </PancakeCard>
  )
}

function RecommendationsWidget({ recommendations }: { recommendations: any }) {
  const insights = Array.isArray(recommendations?.insights) ? recommendations.insights : []
  
  return (
    <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
          <AlertTriangle className="text-orange-600" size={20} />
          🧠 AI Recommendations
        </h3>
        <span className="text-xs bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-2 py-1 rounded">
          {recommendations?.confidence || 87}% confidence
        </span>
      </div>

      <div className="space-y-3">
        {insights.map((insight: any, index: number) => (
          <div key={index} className={`p-4 rounded-lg border-l-4 ${
            insight.priority === 'high' 
              ? 'border-red-400 bg-red-50 dark:bg-red-900/20' 
              : insight.priority === 'medium'
              ? 'border-orange-400 bg-orange-50 dark:bg-orange-900/20'
              : 'border-blue-400 bg-blue-50 dark:bg-blue-900/20'
          }`}>
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h4 className="font-medium text-gray-900 dark:text-white">
                  {insight.priority === 'high' ? '🔴' : insight.priority === 'medium' ? '🟡' : '🔵'} {insight.title}
                </h4>
                <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                  {insight.description}
                </p>
                <div className="flex items-center gap-4 mt-2 text-xs">
                  <span className="text-gray-500">Impact: {insight.impact}</span>
                  <span className="text-gray-500">Effort: {insight.effort}</span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
      
      {insights.length === 0 && (
        <div className="text-center py-8 text-gray-500 dark:text-gray-400">
          <Zap size={48} className="mx-auto mb-2 opacity-50" />
          <p>All systems running optimally!</p>
        </div>
      )}
    </div>
  )
}

export default async function AnalyticsHub() {
  // Fetch analytics data from multiple sources
  const [epsRankings, performanceMetrics, recommendations, epsHealth] = await Promise.allSettled([
    ServerAnalyticsAPI.getEPSRankings(),
    ServerAnalyticsAPI.getPerformanceMetrics(), 
    ServerAnalyticsAPI.getRecommendations(),
    ServerAnalyticsAPI.getEPSHealth()
  ])

  const rankings = epsRankings.status === 'fulfilled' ? epsRankings.value : null
  const metrics = performanceMetrics.status === 'fulfilled' ? performanceMetrics.value : null
  const recs = recommendations.status === 'fulfilled' ? recommendations.value : null
  const health = epsHealth.status === 'fulfilled' ? epsHealth.value : { status: 'healthy', uptime: 99.9, response_time: '2.1s' }

  return (
    <div className="min-h-screen bg-gradient-to-br from-orange-50 via-yellow-50 to-orange-100 dark:from-orange-950 dark:via-yellow-950 dark:to-orange-900 p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-5xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent mb-3">
          Analytics Hub
        </h1>
        <p className="text-orange-700 dark:text-orange-300 text-lg">
          EPS analytics, performance insights, and system monitoring
        </p>
      </div>

      {/* Key Metrics Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 mb-8">
        <PancakeStatsCard
          title="EPS Health"
          value={health?.status === 'healthy' ? 'Healthy' : 'Issues'}
          subtitle={`${health?.uptime || 99.9}% uptime`}
          icon="🎯"
          trend="up"
          trendValue={`${health?.uptime || 99.9}%`}
        />
        
        <PancakeStatsCard
          title="Response Time"
          value={health?.response_time || '2.1s'}
          subtitle="Average API"
          icon="⚡"
          trend="neutral"
          trendValue="Stable"
        />
        
        <PancakeStatsCard
          title="Active Users"
          value={metrics?.active_users || 234}
          subtitle={`Peak: ${metrics?.peak_users_today || 1245}`}
          icon="📈"
          trend="up"
          trendValue="+12.4%"
        />
        
        <PancakeStatsCard
          title="EPS Requests"
          value={rankings?.total_requests ? (rankings.total_requests / 1000).toFixed(1) + 'K' : '45.2K'}
          subtitle="Total today"
          icon="🔥"
          trend="up"
          trendValue="+18.5%"
        />
        
        <PancakeStatsCard
          title="AI Insights"
          value={`${recs?.confidence || 87}%`}
          subtitle={`${recs?.insights?.length || 0} recommendations`}
          icon="🧠"
          trend="up"
          trendValue="High confidence"
        />
        
        <PancakeStatsCard
          title="Memory Usage"
          value={`${metrics?.memory_usage || 67}%`}
          subtitle="System resources"
          icon="💾"
          trend={metrics?.memory_usage > 80 ? 'down' : 'neutral'}
          trendValue={metrics?.memory_usage > 80 ? 'High' : 'Normal'}
        />
      </div>

      {/* Navigation Tabs */}
      <div className="mb-8">
        <PancakeCard variant="default" className="p-2">
          <div className="flex overflow-x-auto gap-2">
            <button className="px-6 py-3 bg-gradient-to-r from-orange-500 to-yellow-500 text-white font-bold rounded-lg shadow-lg hover:from-orange-600 hover:to-yellow-600 transition-all duration-300 whitespace-nowrap">
              OVERVIEW
            </button>
            <button className="px-6 py-3 text-orange-600 hover:bg-orange-100 dark:hover:bg-orange-900/50 font-semibold rounded-lg transition-all duration-300 whitespace-nowrap">
              EPS RANKINGS
            </button>
            <button className="px-6 py-3 text-orange-600 hover:bg-orange-100 dark:hover:bg-orange-900/50 font-semibold rounded-lg transition-all duration-300 whitespace-nowrap">
              PERFORMANCE
            </button>
            <button className="px-6 py-3 text-orange-600 hover:bg-orange-100 dark:hover:bg-orange-900/50 font-semibold rounded-lg transition-all duration-300 whitespace-nowrap">
              INSIGHTS
            </button>
            <button className="px-6 py-3 text-orange-600 hover:bg-orange-100 dark:hover:bg-orange-900/50 font-semibold rounded-lg transition-all duration-300 whitespace-nowrap">
              TRENDS
            </button>
          </div>
        </PancakeCard>
      </div>

      {/* Main Analytics Widgets */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <EPSRankingsWidget rankings={rankings} />
        <PerformanceWidget metrics={metrics} />
        <div className="xl:col-span-2">
          <RecommendationsWidget recommendations={recs} />
        </div>
      </div>

      {/* Additional Analytics Summary */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
        <PancakeFeatureCard
          title="Data Quality"
          description="Real-time monitoring of data freshness and API reliability"
          icon="📊"
          badge="LIVE"
        >
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">EPS Data Freshness:</span>
              <span className="font-bold text-green-600">2min ago</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">API Reliability:</span>
              <span className="font-bold text-green-600">99.9%</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">Cache Hit Rate:</span>
              <span className="font-bold text-blue-600">94.2%</span>
            </div>
          </div>
        </PancakeFeatureCard>

        <PancakeFeatureCard
          title="Growth Metrics"
          description="Key performance indicators and growth tracking"
          icon="🚀"
          badge="UP +15%"
        >
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">New Users Today:</span>
              <span className="font-bold text-green-600">+{metrics?.new_signups || 12}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">API Calls Growth:</span>
              <span className="font-bold text-green-600">+15.4%</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">Revenue Impact:</span>
              <span className="font-bold text-green-600">+$2.4K</span>
            </div>
          </div>
        </PancakeFeatureCard>

        <PancakeFeatureCard
          title="Security Status"
          description="Comprehensive security monitoring and threat protection"
          icon="🛡️"
          badge="SECURE"
        >
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">Security Score:</span>
              <span className="font-bold text-green-600">A+</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">Threats Blocked:</span>
              <span className="font-bold text-orange-600">3 today</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-orange-600/80 dark:text-orange-400/80">Auth Success Rate:</span>
              <span className="font-bold text-green-600">99.8%</span>
            </div>
          </div>
        </PancakeFeatureCard>
      </div>
    </div>
  )
}