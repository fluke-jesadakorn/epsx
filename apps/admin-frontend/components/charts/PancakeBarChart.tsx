/**
 * PancakeSwap Bar Chart Component
 * Windows Phone styled bar chart for usage statistics and rankings
 */

'use client'

import React from 'react'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  Cell,
  TooltipProps
} from 'recharts'

import { PancakeSwapColors, getChartColor, getThemeColors } from './chartColors'
import { ChartContainer } from './ChartContainer'

interface DataPoint {
  [key: string]: any
}

interface BarConfig {
  dataKey: string
  name: string
  color?: string
  gradient?: boolean
}

interface PancakeBarChartProps {
  data: DataPoint[]
  bars: BarConfig[]
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
  barRadius?: number
  layout?: 'horizontal' | 'vertical'
  useGradient?: boolean
  customColors?: string[]
}

const CustomTooltip = ({ 
  active, 
  payload, 
  label, 
  formatTooltip 
}: any) => {
  if (!active || !payload?.length) {return null}

  return (
    <div className="bg-card/95 backdrop-blur-sm border border-yellow-400/30 rounded-lg p-3 shadow-xl">
      <div className="font-light text-sm text-foreground mb-2 uppercase tracking-wider">
        {label}
      </div>
      <div className="space-y-1">
        {payload.map((entry: any, index: number) => {
          const [formattedValue, formattedName] = formatTooltip 
            ? formatTooltip(entry.value, entry.name || '') 
            : [entry.value, entry.name || '']
          
          return (
            <div key={index} className="flex items-center gap-2">
              <div 
                className="w-2 h-2 rounded-sm"
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

/**
 *
 * @param root0
 * @param root0.data
 * @param root0.bars
 * @param root0.title
 * @param root0.subtitle
 * @param root0.height
 * @param root0.showGrid
 * @param root0.showLegend
 * @param root0.showTooltip
 * @param root0.xAxisDataKey
 * @param root0.formatTooltip
 * @param root0.formatXAxis
 * @param root0.formatYAxis
 * @param root0.className
 * @param root0.variant
 * @param root0.showLiveDot
 * @param root0.isDark
 * @param root0.barRadius
 * @param root0.layout
 * @param root0.useGradient
 * @param root0.customColors
 */
export function PancakeBarChart({
  data,
  bars,
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
  isDark = false,
  barRadius = 4,
  layout = 'vertical',
  useGradient = false,
  customColors
}: PancakeBarChartProps) {
  const themeColors = getThemeColors(isDark)
  
  // Generate gradient definitions for bars
  const gradients = bars.map((bar, index) => {
    const color = bar.color || customColors?.[index] || getChartColor(index)
    const gradientId = `gradient-${bar.dataKey}-${index}`
    
    return (
      <defs key={gradientId}>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="5%" stopColor={color} stopOpacity={0.9}/>
          <stop offset="95%" stopColor={color} stopOpacity={0.6}/>
        </linearGradient>
      </defs>
    )
  })
  
  return (
    <ChartContainer
      title={title}
      subtitle={subtitle}
      variant={variant}
      className={className}
      showLiveDot={showLiveDot}
    >
      <ResponsiveContainer width="100%" height={height}>
        <BarChart 
          data={data} 
          layout={layout}
          margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
        >
          {useGradient && gradients}
          
          {showGrid && (
            <CartesianGrid 
              strokeDasharray="3 3" 
              stroke={themeColors.grid}
              opacity={0.3}
            />
          )}
          
          <XAxis
            type={layout === 'vertical' ? 'category' : 'number'}
            dataKey={layout === 'vertical' ? xAxisDataKey : undefined}
            axisLine={false}
            tickLine={false}
            tick={{ 
              fontSize: 12, 
              fill: themeColors.textMuted,
              fontWeight: 300
            }}
            tickFormatter={formatXAxis}
            angle={layout === 'vertical' ? -45 : 0}
            textAnchor={layout === 'vertical' ? 'end' : 'middle'}
            height={layout === 'vertical' ? 80 : undefined}
          />
          
          <YAxis
            type={layout === 'vertical' ? 'number' : 'category'}
            dataKey={layout === 'horizontal' ? xAxisDataKey : undefined}
            axisLine={false}
            tickLine={false}
            tick={{ 
              fontSize: 12, 
              fill: themeColors.textMuted,
              fontWeight: 300
            }}
            tickFormatter={formatYAxis}
            width={layout === 'horizontal' ? 120 : undefined}
          />
          
          {showTooltip && (
            <Tooltip
              content={<CustomTooltip formatTooltip={formatTooltip} />}
              cursor={{ fill: PancakeSwapColors.primary, opacity: 0.1 }}
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
          
          {bars.map((barConfig, barIndex) => (
            <Bar
              key={barConfig.dataKey}
              dataKey={barConfig.dataKey}
              name={barConfig.name}
              fill={useGradient ? `url(#gradient-${barConfig.dataKey}-${barIndex})` : (barConfig.color || getChartColor(barIndex))}
              radius={barRadius}
            >
              {/* Custom colors for each data point if provided */}
              {customColors && data.map((entry, index) => (
                <Cell 
                  key={`cell-${index}`} 
                  fill={customColors[index % customColors.length]} 
                />
              ))}
            </Bar>
          ))}
        </BarChart>
      </ResponsiveContainer>
    </ChartContainer>
  )
}