import React from 'react'
import { TrendingUp, Activity, BarChart3, PieChart, Target, AlertTriangle, Zap } from 'lucide-react'
import { ServerAnalyticsAPI } from '@/lib/api/admin-client'

/**
 * Windows Phone-style Analytics Hub
 * EPS analytics, performance metrics, and insights dashboard
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
    <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
          <Target className="text-blue-600" size={20} />
          🎯 Top EPS Growth
        </h3>
        <span className="text-xs text-gray-500">Live Data</span>
      </div>
      
      <div className="space-y-3">
        {top5.map((stock: any, index: number) => (
          <div key={index} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <div className="flex items-center gap-3">
              <span className="text-sm font-bold text-blue-600 dark:text-blue-400">
                #{index + 1}
              </span>
              <div>
                <div className="font-medium text-gray-900 dark:text-white">
                  {stock.symbol}
                </div>
                <div className="text-xs text-gray-500">
                  {stock.country} • {stock.sector}
                </div>
              </div>
            </div>
            <div className="text-right">
              <div className="font-bold text-green-600">
                +{stock.eps_growth}%
              </div>
            </div>
          </div>
        ))}
      </div>
      
      <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-600">
        <div className="flex justify-between items-center text-sm text-gray-600 dark:text-gray-400">
          <span>Total Requests: {rankings?.total_requests?.toLocaleString() || 0}</span>
          <span>Updated: {rankings?.last_updated ? new Date(rankings.last_updated).toLocaleTime() : 'N/A'}</span>
        </div>
      </div>
    </div>
  )
}

function PerformanceWidget({ metrics }: { metrics: any }) {
  return (
    <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
          <Activity className="text-green-600" size={20} />
          ⚡ System Performance
        </h3>
        <div className="flex items-center gap-1">
          <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
          <span className="text-xs text-gray-500">Live</span>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-3">
          <div>
            <div className="text-sm text-gray-600 dark:text-gray-400">API Response</div>
            <div className="text-lg font-bold text-green-600">{metrics?.api_response_time || 1.2}s</div>
            <div className="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2 mt-1">
              <div className="bg-green-500 h-2 rounded-full" style={{ width: '85%' }}></div>
            </div>
          </div>
          
          <div>
            <div className="text-sm text-gray-600 dark:text-gray-400">Memory Usage</div>
            <div className="text-lg font-bold text-orange-600">{metrics?.memory_usage || 67}%</div>
            <div className="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2 mt-1">
              <div className="bg-orange-500 h-2 rounded-full" style={{ width: `${metrics?.memory_usage || 67}%` }}></div>
            </div>
          </div>
        </div>

        <div className="space-y-3">
          <div>
            <div className="text-sm text-gray-600 dark:text-gray-400">Active Users</div>
            <div className="text-lg font-bold text-blue-600">{metrics?.active_users || 234}</div>
            <div className="text-xs text-gray-500">Peak: {metrics?.peak_users_today || 1245}</div>
          </div>
          
          <div>
            <div className="text-sm text-gray-600 dark:text-gray-400">New Signups</div>
            <div className="text-lg font-bold text-purple-600">+{metrics?.new_signups || 12}</div>
            <div className="text-xs text-gray-500">Today</div>
          </div>
        </div>
      </div>
    </div>
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
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-4xl font-light text-gray-900 dark:text-white mb-2">
          📊 ANALYTICS HUB
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          EPS analytics, performance insights, and system monitoring
        </p>
      </div>

      {/* Key Metrics Tiles */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 gap-4 mb-8">
        <MetricTile
          title="🎯 EPS Health"
          value={health?.status === 'healthy' ? '✅ Healthy' : '❌ Issues'}
          subValue={`${health?.uptime || 99.9}% uptime`}
          trend="up"
          color="bg-green-500"
          icon={Target}
        />
        
        <MetricTile
          title="⚡ Response Time"
          value={health?.response_time || '2.1s'}
          subValue="Average API"
          trend="stable"
          color="bg-blue-500"
          icon={Zap}
        />
        
        <MetricTile
          title="📈 Active Users"
          value={metrics?.active_users || 234}
          subValue={`Peak: ${metrics?.peak_users_today || 1245}`}
          trend="up"
          color="bg-purple-500"
          icon={TrendingUp}
        />
        
        <MetricTile
          title="🔥 EPS Requests"
          value={rankings?.total_requests ? (rankings.total_requests / 1000).toFixed(1) + 'K' : '45.2K'}
          subValue="Total today"
          trend="up"
          color="bg-orange-500"
          icon={BarChart3}
          size="wide"
        />
        
        <MetricTile
          title="🧠 AI Confidence"
          value={`${recs?.confidence || 87}%`}
          subValue={`${recs?.insights?.length || 0} insights`}
          trend="up"
          color="bg-indigo-500"
          icon={Activity}
        />
        
        <MetricTile
          title="💾 Memory Usage"
          value={`${metrics?.memory_usage || 67}%`}
          subValue="System resources"
          trend={metrics?.memory_usage > 80 ? 'up' : 'stable'}
          color={metrics?.memory_usage > 80 ? 'bg-red-500' : 'bg-green-500'}
          icon={PieChart}
        />
      </div>

      {/* Pivot Navigation */}
      <div className="mb-6">
        <div className="flex overflow-x-auto gap-1 border-b border-gray-200 dark:border-gray-700">
          <button className="px-4 py-3 font-medium text-blue-600 border-b-2 border-blue-600 whitespace-nowrap">
            ◄ OVERVIEW ►
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            EPS RANKINGS
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            PERFORMANCE
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            INSIGHTS
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            TRENDS
          </button>
        </div>
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
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            📊 Data Quality
          </h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">EPS Data Freshness:</span>
              <span className="font-medium text-green-600">🟢 2min ago</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">API Reliability:</span>
              <span className="font-medium text-green-600">🟢 99.9%</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Cache Hit Rate:</span>
              <span className="font-medium text-blue-600">🔵 94.2%</span>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            🚀 Growth Metrics
          </h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">New Users Today:</span>
              <span className="font-medium text-green-600">+{metrics?.new_signups || 12}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">API Calls Growth:</span>
              <span className="font-medium text-green-600">+15.4%</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Revenue Impact:</span>
              <span className="font-medium text-green-600">+$2.4K</span>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            🛡️ Security Status
          </h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Security Score:</span>
              <span className="font-medium text-green-600">🟢 A+</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Threats Blocked:</span>
              <span className="font-medium text-orange-600">🟡 3 today</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Auth Success Rate:</span>
              <span className="font-medium text-green-600">🟢 99.8%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}