'use client';

import { useState, useEffect, useMemo } from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import {
  Clock,
  TrendingUp,
  TrendingDown,
  Activity,
  Shield,
  Zap,
  Lock,
  AlertTriangle
} from 'lucide-react';

interface AttackPattern {
  type: string;
  count: number;
  trend: 'increasing' | 'decreasing' | 'stable';
  peakHour: number;
  description: string;
}

interface HourlyData {
  hour: number;
  attacks: { [key: string]: number };
  total: number;
}

interface AttackTimelineProps {
  patterns: AttackPattern[];
}

export function AttackTimeline({ patterns }: AttackTimelineProps) {
  const [selectedHour, setSelectedHour] = useState<number | null>(null);
  const [viewMode, setViewMode] = useState<'bar' | 'heatmap'>('bar');
  const [timelineData, setTimelineData] = useState<HourlyData[]>([]);

  // Generate hourly attack data
  useEffect(() => {
    const generateHourlyData = () => {
      const data: HourlyData[] = [];
      
      for (let hour = 0; hour < 24; hour++) {
        const attacks: { [key: string]: number } = {};
        let total = 0;
        
        patterns.forEach(pattern => {
          // Create realistic distribution with peak hours
          const hourDistance = Math.abs(hour - pattern.peakHour);
          const normalizedDistance = Math.min(hourDistance, 24 - hourDistance);
          const peakFactor = Math.exp(-normalizedDistance / 4); // Bell curve around peak
          
          // Add some randomness and trend influence
          const trendMultiplier = {
            'increasing': 1.2,
            'stable': 1.0,
            'decreasing': 0.8
          }[pattern.trend];
          
          const baseCount = (pattern.count / 24) * peakFactor * trendMultiplier;
          const randomFactor = 0.7 + Math.random() * 0.6; // ±30% variation
          const attackCount = Math.max(0, Math.floor(baseCount * randomFactor));
          
          attacks[pattern.type] = attackCount;
          total += attackCount;
        });
        
        data.push({ hour, attacks, total });
      }
      
      return data;
    };

    setTimelineData(generateHourlyData());
  }, [patterns]);

  // Get color for attack type
  const getAttackColor = (type: string) => {
    const colors: { [key: string]: string } = {
      'Brute Force': '#ef4444',
      'SQL Injection': '#f97316',
      'DDoS': '#eab308',
      'Malware': '#8b5cf6',
      'Phishing': '#06b6d4'
    };
    return colors[type] || '#6b7280';
  };

  // Get attack icon
  const getAttackIcon = (type: string) => {
    const icons: { [key: string]: React.ComponentType<any> } = {
      'Brute Force': Lock,
      'SQL Injection': Shield,
      'DDoS': Zap,
      'Malware': AlertTriangle,
      'Phishing': Activity
    };
    return icons[type] || Activity;
  };

  // Format hour for display
  const formatHour = (hour: number) => {
    return `${hour.toString().padStart(2, '0')}:00`;
  };

  // Get intensity class for heatmap
  const getIntensityClass = (value: number, maxValue: number) => {
    const intensity = value / maxValue;
    if (intensity > 0.8) return 'bg-red-500';
    if (intensity > 0.6) return 'bg-orange-500';
    if (intensity > 0.4) return 'bg-yellow-500';
    if (intensity > 0.2) return 'bg-blue-500';
    if (intensity > 0) return 'bg-gray-400';
    return 'bg-gray-100 dark:bg-gray-800';
  };

  const maxAttacks = Math.max(...timelineData.map(d => d.total));
  const currentHour = new Date().getHours();

  return (
    <div className="space-y-4">
      {/* View Mode Toggle */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Clock className="w-4 h-4" />
          <span className="font-medium">24-Hour Attack Pattern</span>
        </div>
        <div className="flex rounded-lg border p-1">
          <Button
            size="sm"
            variant={viewMode === 'bar' ? 'default' : 'ghost'}
            onClick={() => setViewMode('bar')}
            className="h-8 px-3"
          >
            Bar Chart
          </Button>
          <Button
            size="sm"
            variant={viewMode === 'heatmap' ? 'default' : 'ghost'}
            onClick={() => setViewMode('heatmap')}
            className="h-8 px-3"
          >
            Heatmap
          </Button>
        </div>
      </div>

      {/* Timeline Visualization */}
      {viewMode === 'bar' ? (
        <div className="space-y-3">
          {/* Bar Chart */}
          <div className="relative">
            <svg width="100%" height="200" className="overflow-visible">
              {/* Grid lines */}
              <defs>
                <pattern id="grid" width="4.166%" height="40" patternUnits="userSpaceOnUse">
                  <path d="M 0 0 L 0 40" stroke="hsl(var(--border))" strokeWidth="0.5" opacity="0.3"/>
                </pattern>
              </defs>
              <rect width="100%" height="200" fill="url(#grid)" />
              
              {/* Bars */}
              {timelineData.map((data, index) => {
                const barWidth = 100 / 24;
                const x = (index / 24) * 100;
                const isSelected = selectedHour === data.hour;
                const isCurrentHour = data.hour === currentHour;
                
                // Stacked bars for different attack types
                let yOffset = 0;
                return (
                  <g key={data.hour}>
                    {patterns.map(pattern => {
                      const attackCount = data.attacks[pattern.type] || 0;
                      const height = (attackCount / maxAttacks) * 160;
                      const color = getAttackColor(pattern.type);
                      const rect = (
                        <rect
                          key={pattern.type}
                          x={`${x}%`}
                          y={180 - yOffset - height}
                          width={`${barWidth * 0.8}%`}
                          height={height}
                          fill={color}
                          opacity={isSelected ? 1 : 0.7}
                          className={cn(
                            "cursor-pointer transition-all hover:opacity-100",
                            isCurrentHour && "stroke-white stroke-2"
                          )}
                          onClick={() => setSelectedHour(isSelected ? null : data.hour)}
                        />
                      );
                      yOffset += height;
                      return rect;
                    })}
                    
                    {/* Hour label */}
                    <text
                      x={`${x + barWidth / 2}%`}
                      y="195"
                      textAnchor="middle"
                      className={cn(
                        "text-xs fill-current",
                        isCurrentHour ? "font-bold fill-primary" : "fill-muted-foreground"
                      )}
                    >
                      {data.hour}
                    </text>
                    
                    {/* Current hour indicator */}
                    {isCurrentHour && (
                      <circle
                        cx={`${x + barWidth / 2}%`}
                        cy="10"
                        r="3"
                        fill="hsl(var(--primary))"
                        className="animate-pulse"
                      />
                    )}
                  </g>
                );
              })}
            </svg>

            {/* Y-axis labels */}
            <div className="absolute left-0 top-0 h-full flex flex-col justify-between text-xs text-muted-foreground -translate-x-8">
              <span>{maxAttacks}</span>
              <span>{Math.floor(maxAttacks * 0.5)}</span>
              <span>0</span>
            </div>
          </div>

          {/* Legend */}
          <div className="flex flex-wrap gap-4 justify-center">
            {patterns.map(pattern => (
              <div key={pattern.type} className="flex items-center gap-2">
                <div
                  className="w-3 h-3 rounded"
                  style={{ backgroundColor: getAttackColor(pattern.type) }}
                />
                <span className="text-sm">{pattern.type}</span>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          {/* Heatmap */}
          <div className="space-y-2">
            {patterns.map(pattern => (
              <div key={pattern.type} className="flex items-center gap-2">
                <div className="w-24 text-sm font-medium truncate">
                  {pattern.type}
                </div>
                <div className="flex-1 flex gap-1">
                  {timelineData.map(data => {
                    const attackCount = data.attacks[pattern.type] || 0;
                    const maxForType = Math.max(...timelineData.map(d => d.attacks[pattern.type] || 0));
                    
                    return (
                      <div
                        key={data.hour}
                        className={cn(
                          "flex-1 h-8 rounded cursor-pointer transition-all hover:scale-105 relative",
                          getIntensityClass(attackCount, maxForType),
                          selectedHour === data.hour && "ring-2 ring-primary"
                        )}
                        onClick={() => setSelectedHour(selectedHour === data.hour ? null : data.hour)}
                        title={`${formatHour(data.hour)}: ${attackCount} ${pattern.type} attacks`}
                      >
                        {data.hour === currentHour && (
                          <div className="absolute inset-0 border-2 border-white rounded animate-pulse" />
                        )}
                        {attackCount > 0 && (
                          <div className="absolute inset-0 flex items-center justify-center text-white text-xs font-bold">
                            {attackCount > 99 ? '99+' : attackCount || ''}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>

          {/* Hour labels */}
          <div className="flex gap-1 ml-26">
            {Array.from({ length: 24 }, (_, i) => (
              <div
                key={i}
                className={cn(
                  "flex-1 text-center text-xs",
                  i === currentHour ? "font-bold text-primary" : "text-muted-foreground"
                )}
              >
                {i % 4 === 0 ? formatHour(i) : ''}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Selected Hour Details */}
      {selectedHour !== null && (
        <Card className="bg-muted/50">
          <CardContent className="p-4">
            <div className="flex items-center justify-between mb-3">
              <h4 className="font-semibold">
                {formatHour(selectedHour)} - Attack Details
              </h4>
              <Badge variant="outline">
                {timelineData[selectedHour]?.total || 0} total attacks
              </Badge>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
              {patterns.map(pattern => {
                const attackCount = timelineData[selectedHour]?.attacks[pattern.type] || 0;
                const AttackIcon = getAttackIcon(pattern.type);
                const percentage = timelineData[selectedHour]?.total 
                  ? (attackCount / timelineData[selectedHour].total * 100).toFixed(1)
                  : '0';

                return (
                  <div
                    key={pattern.type}
                    className="flex items-center gap-2 p-2 rounded-lg border"
                  >
                    <div
                      className="p-1 rounded"
                      style={{ backgroundColor: getAttackColor(pattern.type) + '20' }}
                    >
                      <AttackIcon 
                        className="w-4 h-4" 
                        style={{ color: getAttackColor(pattern.type) }}
                      />
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="font-medium text-sm">{attackCount}</div>
                      <div className="text-xs text-muted-foreground truncate">
                        {pattern.type}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {percentage}%
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Summary Stats */}
      <div className="grid grid-cols-3 gap-4 text-center">
        <div>
          <div className="text-2xl font-bold text-red-500">
            {Math.max(...timelineData.map(d => d.total))}
          </div>
          <div className="text-sm text-muted-foreground">Peak Hour</div>
        </div>
        <div>
          <div className="text-2xl font-bold text-blue-500">
            {Math.round(timelineData.reduce((sum, d) => sum + d.total, 0) / 24)}
          </div>
          <div className="text-sm text-muted-foreground">Avg/Hour</div>
        </div>
        <div>
          <div className="text-2xl font-bold text-green-500">
            {timelineData.reduce((sum, d) => sum + d.total, 0)}
          </div>
          <div className="text-sm text-muted-foreground">Total/Day</div>
        </div>
      </div>
    </div>
  );
}