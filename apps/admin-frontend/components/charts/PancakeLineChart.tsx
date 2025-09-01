/**
 * PancakeSwap Line Chart Component
 * Windows Phone styled line chart for trends and performance metrics
 */

'use client'

import React from 'react'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  TooltipProps
} from 'recharts'
import { PancakeSwapColors, getChartColor, getThemeColors } from './chartColors'
import { ChartContainer } from './ChartContainer'

interface DataPoint {
  [key: string]: any
}

interface LineConfig {
  dataKey: string
  name: string
  color?: string
  strokeWidth?: number
  strokeDasharray?: string
}

interface PancakeLineChartProps {
  data: DataPoint[]
  lines: LineConfig[]
  title: string
  subtitle?: string
  height?: number
  showGrid?: boolean
  showLegend?: boolean
  showTooltip?: boolean
  xAxisDataKey?: string
  formatTooltip?: (value: any, name: string) => [string, string]
  formatXAxis?: (value: any) => string
  formatYAxis?: (value: any) => string
  className?: string
  variant?: 'analytics' | 'user' | 'permission' | 'billing' | 'pancake' | 'default'
  showLiveDot?: boolean
  isDark?: boolean
}

const CustomTooltip = ({ 
  active, 
  payload, 
  label, 
  formatTooltip 
}: TooltipProps<any, any> & { formatTooltip?: (value: any, name: string) => [string, string] }) => {
  if (!active || !payload || !payload.length) return null

  return (
    <div className="bg-card/95 backdrop-blur-sm border border-yellow-400/30 rounded-lg p-3 shadow-xl">
      <div className="font-light text-sm text-foreground mb-2 uppercase tracking-wider">
        {label}
      </div>
      <div className="space-y-1">
        {payload.map((entry, index) => {
          const [formattedValue, formattedName] = formatTooltip 
            ? formatTooltip(entry.value, entry.name || '') 
            : [entry.value, entry.name || '']
          
          return (
            <div key={index} className="flex items-center gap-2">
              <div 
                className="w-2 h-2 rounded-full"
                style={{ backgroundColor: entry.color }}
              />
              <span className="text-xs text-muted-foreground font-light">
                {formattedName}:
              </span>
              <span className="text-sm font-medium text-foreground">
                {formattedValue}
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}

export function PancakeLineChart({
  data,
  lines,
  title,
  subtitle,
  height = 300,
  showGrid = true,
  showLegend = true,
  showTooltip = true,
  xAxisDataKey = 'name',
  formatTooltip,
  formatXAxis,
  formatYAxis,
  className,
  variant = 'pancake',
  showLiveDot = false,
  isDark = false
}: PancakeLineChartProps) {
  const themeColors = getThemeColors(isDark)
  
  return (
    <ChartContainer
      title={title}
      subtitle={subtitle}
      variant={variant}
      className={className}
      showLiveDot={showLiveDot}
    >
      <ResponsiveContainer width="100%" height={height}>
        <LineChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
          {showGrid && (
            <CartesianGrid 
              strokeDasharray="3 3" 
              stroke={themeColors.grid}
              opacity={0.3}
            />
          )}
          
          <XAxis
            dataKey={xAxisDataKey}
            axisLine={false}
            tickLine={false}
            tick={{ 
              fontSize: 12, 
              fill: themeColors.textMuted,
              fontWeight: 300
            }}
            tickFormatter={formatXAxis}
          />
          
          <YAxis
            axisLine={false}
            tickLine={false}
            tick={{ 
              fontSize: 12, 
              fill: themeColors.textMuted,
              fontWeight: 300
            }}
            tickFormatter={formatYAxis}
          />
          
          {showTooltip && (
            <Tooltip
              content={<CustomTooltip formatTooltip={formatTooltip} />}
              cursor={{ stroke: PancakeSwapColors.primary, strokeWidth: 1, opacity: 0.3 }}
            />
          )}
          
          {showLegend && (
            <Legend
              wrapperStyle={{
                fontSize: '12px',
                fontWeight: 300,
                color: themeColors.textMuted
              }}
            />
          )}
          
          {lines.map((lineConfig, index) => (
            <Line
              key={lineConfig.dataKey}
              type="monotone"
              dataKey={lineConfig.dataKey}
              name={lineConfig.name}
              stroke={lineConfig.color || getChartColor(index)}
              strokeWidth={lineConfig.strokeWidth || 2}
              strokeDasharray={lineConfig.strokeDasharray}
              dot={{
                r: 4,
                fill: lineConfig.color || getChartColor(index),
                strokeWidth: 2,
                stroke: themeColors.background
              }}
              activeDot={{
                r: 6,
                fill: PancakeSwapColors.primary,
                stroke: themeColors.background,
                strokeWidth: 2
              }}
              connectNulls={false}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </ChartContainer>
  )
}