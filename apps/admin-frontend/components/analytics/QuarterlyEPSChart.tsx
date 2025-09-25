import React from 'react'
import { formatEPS, formatPercentage, getGrowthIndicator, formatQuarterDate, getQuarterLabel } from '@/lib/utils'

interface QuarterlyEPSData {
  quarter: string           // "Q-2", "Q-1", "Q0", "Q+1"
  value: number | null      // EPS value
  date: string | null       // Quarter date
  growth: number | null     // QoQ growth %
  isEstimate: boolean       // True for Q+1
  isBase: boolean           // True for Q-2 (baseline)
}

interface QuarterlyEPSChartProps {
  symbol: string
  quarterlyData: QuarterlyEPSData[]
  qoqGrowthCurrent?: number | null
  yoyGrowthCurrent?: number | null
  trendDirection?: 'UP' | 'DOWN' | 'FLAT'
  avgGrowthRate?: number | null
  consistency?: 'HIGH' | 'MEDIUM' | 'LOW'
}

export function QuarterlyEPSChart({
  symbol,
  quarterlyData,
  qoqGrowthCurrent,
  yoyGrowthCurrent,
  trendDirection,
  avgGrowthRate,
  consistency
}: QuarterlyEPSChartProps) {
  // Sort quarters in chronological order (Q-2, Q-1, Q0, Q+1)
  const sortedQuarters = quarterlyData.sort((a, b) => {
    const order = { 'Q-2': 0, 'Q-1': 1, 'Q0': 2, 'Q+1': 3 }
    return (order[a.quarter as keyof typeof order] || 0) - (order[b.quarter as keyof typeof order] || 0)
  })

  const getTrendIcon = (direction?: string) => {
    switch (direction) {
      case 'UP': return '📈'
      case 'DOWN': return '📉'
      case 'FLAT': return '➡️'
      default: return '➡️'
    }
  }

  const getConsistencyColor = (level?: string) => {
    switch (level) {
      case 'HIGH': return 'text-green-600'
      case 'MEDIUM': return 'text-yellow-600'
      case 'LOW': return 'text-red-600'
      default: return 'text-gray-600'
    }
  }

  return (
    <div className="rounded-2xl border border-slate-200/50 bg-white/50 p-3 backdrop-blur-sm">
      {/* Header */}
      <div className="mb-2 text-center">
        <div className="text-xs font-bold text-slate-600 uppercase tracking-wide">📊 4-Quarter EPS Analysis</div>
        <div className="flex items-center justify-center gap-1 mt-1">
          <span className="text-xs font-medium text-blue-600">{symbol}</span>
          <span className="text-xs text-gray-500">•</span>
          <span className="text-xs">{getTrendIcon(trendDirection)}</span>
          <span className="text-xs font-medium text-gray-600">{trendDirection || 'FLAT'}</span>
        </div>
      </div>

      {/* 4-Quarter Grid - Column Layout */}
      <div className="grid grid-rows-4 gap-1 mb-2">
        {sortedQuarters.map((quarter, index) => {
          const isCurrentQuarter = quarter.quarter === 'Q0'
          const isEstimate = quarter.isEstimate
          const isBase = quarter.isBase
          
          return (
            <div
              key={quarter.quarter}
              className={`
                relative p-2 rounded border transition-all duration-200 flex items-center gap-3 text-xs
                ${isCurrentQuarter ? 'border-blue-400 bg-blue-50/80' : 'border-slate-300/50 bg-slate-50/50'}
                ${isEstimate ? 'border-dashed border-purple-400 bg-purple-50/80' : ''}
                hover:shadow-sm
              `}
            >
              {/* Quarter Label - Row Layout */}
              <div className="flex-shrink-0 w-12">
                <div className="text-xs font-medium text-slate-600">
                  {quarter.quarter === 'Q-2' && '2Q Ago'}
                  {quarter.quarter === 'Q-1' && 'Last Q'}
                  {quarter.quarter === 'Q0' && 'Current'}
                  {quarter.quarter === 'Q+1' && 'Next Est.'}
                </div>
              </div>

              {/* EPS Value - Row Layout */}
              <div className="flex-1 text-center">
                <div className="text-sm font-bold text-slate-800">
                  {quarter.value ? formatEPS(quarter.value) : 'N/A'}
                </div>
              </div>

              {/* Growth Indicator - Row Layout */}
              <div className="flex-shrink-0 w-16 text-right">
                {quarter.growth !== null && quarter.growth !== undefined ? (
                  <div className={`flex items-center justify-end gap-1 text-xs font-medium ${
                    quarter.growth > 0 ? 'text-green-600' : quarter.growth < 0 ? 'text-red-600' : 'text-gray-600'
                  }`}>
                    <span>{getGrowthIndicator(quarter.growth).emoji}</span>
                    <span>{formatPercentage(quarter.growth)}</span>
                  </div>
                ) : isBase ? (
                  <div className="text-xs text-gray-500 font-medium">📊 Base</div>
                ) : isEstimate ? (
                  <div className="text-xs text-purple-600 font-medium">🔮 Est.</div>
                ) : (
                  <div className="text-xs text-gray-400">—</div>
                )}
              </div>

              {/* Special Markers */}
              {isCurrentQuarter && (
                <div className="absolute -top-1 -right-1 w-2 h-2 bg-blue-500 rounded-full"></div>
              )}
              {isEstimate && (
                <div className="absolute -top-1 -left-1 w-2 h-2 bg-purple-500 rounded-full"></div>
              )}
            </div>
          )
        })}
      </div>

      {/* Summary Metrics - Compact */}
      <div className="grid grid-cols-3 gap-1 pt-1 border-t border-slate-200/50 text-center text-xs">
        <div>
          <div className="text-slate-500">QoQ</div>
          <div className={`font-semibold ${
            qoqGrowthCurrent && qoqGrowthCurrent > 0 ? 'text-green-600' : 
            qoqGrowthCurrent && qoqGrowthCurrent < 0 ? 'text-red-600' : 'text-slate-600'
          } truncate`}>
            {qoqGrowthCurrent !== null && qoqGrowthCurrent !== undefined ? formatPercentage(qoqGrowthCurrent) : 'N/A'}
          </div>
        </div>
        
        <div>
          <div className="text-slate-500">YoY</div>
          <div className={`font-semibold ${
            yoyGrowthCurrent && yoyGrowthCurrent > 0 ? 'text-green-600' : 
            yoyGrowthCurrent && yoyGrowthCurrent < 0 ? 'text-red-600' : 'text-slate-600'
          } truncate`}>
            {yoyGrowthCurrent !== null && yoyGrowthCurrent !== undefined ? formatPercentage(yoyGrowthCurrent) : 'N/A'}
          </div>
        </div>
        
        <div>
          <div className="text-slate-500">Score</div>
          <div className={`font-semibold ${getConsistencyColor(consistency)} truncate`}>
            {consistency || 'N/A'}
          </div>
        </div>
      </div>
    </div>
  )
}