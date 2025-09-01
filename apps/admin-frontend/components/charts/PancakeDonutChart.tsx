/**
 * PancakeSwap Donut Chart Component
 * Windows Phone styled donut/pie chart for category breakdowns and distribution
 */

'use client'

import React from 'react'
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip,
  Legend,
  TooltipProps
} from 'recharts'
import { PancakeSwapColors, getChartColor, getThemeColors } from './chartColors'
import { ChartContainer } from './ChartContainer'

interface DataPoint {
  name: string
  value: number
  color?: string
  [key: string]: any
}

interface PancakeDonutChartProps {
  data: DataPoint[]
  title: string
  subtitle?: string
  height?: number
  showTooltip?: boolean
  showLegend?: boolean
  innerRadius?: number
  outerRadius?: number
  showLabels?: boolean
  showValues?: boolean
  formatTooltip?: (value: any, name: string) => [string, string]
  formatLabel?: (entry: DataPoint) => string
  className?: string
  variant?: 'analytics' | 'user' | 'permission' | 'billing' | 'pancake' | 'default'
  showLiveDot?: boolean
  isDark?: boolean
  customColors?: string[]
  centerContent?: React.ReactNode
  animationBegin?: number
  animationDuration?: number
}

const CustomTooltip = ({ 
  active, 
  payload, 
  formatTooltip 
}: TooltipProps<any, any> & { formatTooltip?: (value: any, name: string) => [string, string] }) => {
  if (!active || !payload || !payload.length) return null

  const data = payload[0]
  const [formattedValue, formattedName] = formatTooltip 
    ? formatTooltip(data.value, data.name) 
    : [data.value, data.name]

  return (
    <div className="bg-card/95 backdrop-blur-sm border border-yellow-400/30 rounded-lg p-3 shadow-xl">
      <div className="flex items-center gap-2 mb-1">
        <div 
          className="w-3 h-3 rounded-sm"
          style={{ backgroundColor: data.payload.color || data.color }}
        />
        <span className="font-light text-sm text-foreground uppercase tracking-wider">
          {formattedName}
        </span>
      </div>
      <div className="text-lg font-medium text-foreground">
        {formattedValue}
      </div>
      <div className="text-xs text-muted-foreground font-light">
        {((data.value / data.payload.totalValue) * 100).toFixed(1)}% of total
      </div>
    </div>
  )
}

const renderCustomLabel = (entry: any, formatLabel?: (entry: DataPoint) => string) => {
  const RADIAN = Math.PI / 180
  const radius = entry.innerRadius + (entry.outerRadius - entry.innerRadius) * 0.5
  const x = entry.cx + radius * Math.cos(-entry.midAngle * RADIAN)
  const y = entry.cy + radius * Math.sin(-entry.midAngle * RADIAN)

  const label = formatLabel ? formatLabel(entry) : `${entry.name}: ${entry.value}`

  return (
    <text 
      x={x} 
      y={y} 
      fill="#FFFFFF" 
      textAnchor={x > entry.cx ? 'start' : 'end'} 
      dominantBaseline="central"
      fontSize={12}
      fontWeight={300}
      className="drop-shadow-sm"
    >
      {label}
    </text>
  )
}

export function PancakeDonutChart({
  data,
  title,
  subtitle,
  height = 300,
  showTooltip = true,
  showLegend = false,
  innerRadius = 60,
  outerRadius = 100,
  showLabels = false,
  showValues = false,
  formatTooltip,
  formatLabel,
  className,
  variant = 'pancake',
  showLiveDot = false,
  isDark = false,
  customColors,
  centerContent,
  animationBegin = 0,
  animationDuration = 800
}: PancakeDonutChartProps) {
  const themeColors = getThemeColors(isDark)
  
  // Calculate total for percentage calculations
  const totalValue = data.reduce((sum, item) => sum + item.value, 0)
  const dataWithTotal = data.map(item => ({ ...item, totalValue }))
  
  // Apply custom colors or use default palette
  const colorsToUse = customColors || PancakeSwapColors.chartPalette
  
  return (
    <ChartContainer
      title={title}
      subtitle={subtitle}
      variant={variant}
      className={className}
      showLiveDot={showLiveDot}
    >
      <div className="relative">
        <ResponsiveContainer width="100%" height={height}>
          <PieChart>
            <Pie
              data={dataWithTotal}
              cx="50%"
              cy="50%"
              labelLine={false}
              label={showLabels ? (entry) => renderCustomLabel(entry, formatLabel) : false}
              outerRadius={outerRadius}
              innerRadius={innerRadius}
              fill="#8884d8"
              dataKey="value"
              animationBegin={animationBegin}
              animationDuration={animationDuration}
            >
              {dataWithTotal.map((entry, index) => (
                <Cell 
                  key={`cell-${index}`} 
                  fill={entry.color || colorsToUse[index % colorsToUse.length]}
                />
              ))}
            </Pie>
            
            {showTooltip && (
              <Tooltip
                content={<CustomTooltip formatTooltip={formatTooltip} />}
              />
            )}
            
            {showLegend && (
              <Legend
                verticalAlign="bottom"
                height={36}
                wrapperStyle={{
                  fontSize: '12px',
                  fontWeight: 300,
                  color: themeColors.textMuted
                }}
              />
            )}
          </PieChart>
        </ResponsiveContainer>
        
        {/* Center content for donut charts */}
        {centerContent && innerRadius > 0 && (
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
            <div className="text-center">
              {centerContent}
            </div>
          </div>
        )}
        
        {/* Default center content showing total */}
        {!centerContent && innerRadius > 0 && showValues && (
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
            <div className="text-center">
              <div className="text-2xl font-light text-foreground">
                {totalValue.toLocaleString()}
              </div>
              <div className="text-xs text-muted-foreground uppercase tracking-wider">
                total
              </div>
            </div>
          </div>
        )}
      </div>
      
      {/* Legend below chart for better Windows Phone aesthetics */}
      {showLegend && (
        <div className="mt-4 grid grid-cols-2 gap-2">
          {data.map((entry, index) => (
            <div key={entry.name} className="flex items-center gap-2">
              <div 
                className="w-3 h-3 rounded-sm flex-shrink-0"
                style={{ backgroundColor: entry.color || colorsToUse[index % colorsToUse.length] }}
              />
              <span className="text-xs font-light text-muted-foreground truncate">
                {entry.name}
              </span>
              <span className="text-xs font-medium text-foreground ml-auto">
                {entry.value}
              </span>
            </div>
          ))}
        </div>
      )}
    </ChartContainer>
  )
}