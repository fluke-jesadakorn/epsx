'use client';

import { useState, useEffect, useMemo } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { cn } from '@/lib/utils';
import {
  Activity,
  Shield,
  TrendingUp,
  TrendingDown,
  Zap,
  Target,
  Clock,
  RefreshCw
} from 'lucide-react';

interface SecurityMetric {
  timestamp: Date;
  attacksBlocked: number;
  threatLevel: number;
  systemLoad: number;
  responseTime: number;
  falsePositives: number;
  accuracy: number;
}

interface ChartProps {
  data: SecurityMetric[];
  metric: keyof Omit<SecurityMetric, 'timestamp'>;
  title: string;
  color: string;
  unit?: string;
  target?: number;
}

export function SecurityMetricsChart() {
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('24h');
  const [isRealTime, setIsRealTime] = useState(true);
  const [data, setData] = useState<SecurityMetric[]>([]);

  // Generate mock data based on time range
  useEffect(() => {
    const generateData = () => {
      const now = new Date();
      const intervals = {
        '1h': { points: 60, interval: 60000 }, // 1 minute intervals
        '6h': { points: 72, interval: 300000 }, // 5 minute intervals
        '24h': { points: 96, interval: 900000 }, // 15 minute intervals
        '7d': { points: 168, interval: 3600000 } // 1 hour intervals
      };

      const { points, interval } = intervals[timeRange];
      const newData: SecurityMetric[] = [];

      for (let i = points - 1; i >= 0; i--) {
        const timestamp = new Date(now.getTime() - (i * interval));
        
        // Generate realistic security metrics with some correlation
        const baseLoad = 30 + Math.sin(i * 0.1) * 15;
        const attacksBlocked = Math.max(0, Math.floor(Math.random() * 50 + baseLoad));
        const threatLevel = Math.min(100, Math.max(0, 
          20 + (attacksBlocked / 50) * 30 + (Math.random() - 0.5) * 20
        ));
        const systemLoad = Math.min(100, Math.max(10, 
          baseLoad + (attacksBlocked / 50) * 20 + (Math.random() - 0.5) * 15
        ));
        
        newData.push({
          timestamp,
          attacksBlocked,
          threatLevel,
          systemLoad,
          responseTime: Math.max(5, 12 + (systemLoad / 100) * 20 + (Math.random() - 0.5) * 8),
          falsePositives: Math.max(0, Math.floor(Math.random() * 5)),
          accuracy: Math.min(100, Math.max(95, 99.5 - (threatLevel / 100) * 2 + (Math.random() - 0.5) * 1))
        });
      }

      return newData;
    };

    setData(generateData());
  }, [timeRange]);

  // Real-time updates
  useEffect(() => {
    if (!isRealTime) return;

    const interval = setInterval(() => {
      setData(prevData => {
        const newPoint: SecurityMetric = {
          timestamp: new Date(),
          attacksBlocked: Math.floor(Math.random() * 50 + 20),
          threatLevel: Math.floor(Math.random() * 60 + 20),
          systemLoad: Math.floor(Math.random() * 40 + 30),
          responseTime: Math.max(5, 12 + Math.random() * 10),
          falsePositives: Math.floor(Math.random() * 3),
          accuracy: 99.5 - Math.random() * 2
        };

        return [...prevData.slice(1), newPoint];
      });
    }, timeRange === '1h' ? 60000 : 300000); // Update every 1-5 minutes

    return () => clearInterval(interval);
  }, [isRealTime, timeRange]);

  const formatTime = (date: Date) => {
    if (timeRange === '1h' || timeRange === '6h') {
      return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
    }
    if (timeRange === '24h') {
      return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
    }
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };

  const calculateTrend = (values: number[]) => {
    if (values.length < 2) return 'stable';
    const recent = values.slice(-10);
    const older = values.slice(-20, -10);
    const recentAvg = recent.reduce((a, b) => a + b, 0) / recent.length;
    const olderAvg = older.length > 0 ? older.reduce((a, b) => a + b, 0) / older.length : recentAvg;
    
    if (recentAvg > olderAvg * 1.1) return 'up';
    if (recentAvg < olderAvg * 0.9) return 'down';
    return 'stable';
  };

  // Chart component for individual metrics
  const MetricChart = ({ data, metric, title, color, unit = '', target }: ChartProps) => {
    const values = data.map(d => d[metric] as number);
    const maxValue = Math.max(...values, target || 0);
    const minValue = Math.min(...values);
    const range = maxValue - minValue;
    const trend = calculateTrend(values);
    const currentValue = values[values.length - 1];
    const previousValue = values[values.length - 2];
    const change = previousValue ? ((currentValue - previousValue) / previousValue * 100) : 0;

    const getTrendColor = () => {
      switch (trend) {
        case 'up': return metric === 'attacksBlocked' || metric === 'accuracy' ? 'text-green-500' : 'text-red-500';
        case 'down': return metric === 'attacksBlocked' || metric === 'accuracy' ? 'text-red-500' : 'text-green-500';
        default: return 'text-yellow-500';
      }
    };

    const getTrendIcon = () => {
      switch (trend) {
        case 'up': return TrendingUp;
        case 'down': return TrendingDown;
        default: return Activity;
      }
    };

    const TrendIcon = getTrendIcon();

    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h4 className="font-medium">{title}</h4>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className={cn("text-xs", getTrendColor())}>
              <TrendIcon className="w-3 h-3 mr-1" />
              {change > 0 ? '+' : ''}{change.toFixed(1)}%
            </Badge>
            <span className="text-sm font-bold">
              {currentValue.toFixed(metric === 'accuracy' ? 1 : 0)}{unit}
            </span>
          </div>
        </div>

        {/* Simple SVG Chart */}
        <div className="relative">
          <svg width="100%" height="80" className="overflow-visible">
            <defs>
              <linearGradient id={`gradient-${metric}`} x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor={color} stopOpacity="0.3" />
                <stop offset="100%" stopColor={color} stopOpacity="0.1" />
              </linearGradient>
            </defs>
            
            {/* Background grid */}
            <g stroke="hsl(var(--border))" strokeOpacity="0.2">
              <line x1="0" y1="20" x2="100%" y2="20" />
              <line x1="0" y1="40" x2="100%" y2="40" />
              <line x1="0" y1="60" x2="100%" y2="60" />
            </g>

            {/* Target line */}
            {target && (
              <line
                x1="0"
                y1={80 - ((target - minValue) / range) * 60}
                x2="100%"
                y2={80 - ((target - minValue) / range) * 60}
                stroke="hsl(var(--muted-foreground))"
                strokeDasharray="3,3"
                strokeOpacity="0.5"
              />
            )}

            {/* Area chart */}
            <path
              d={`M 0 80 ${data.map((d, i) => {
                const x = (i / (data.length - 1)) * 100;
                const y = 80 - (((d[metric] as number) - minValue) / range) * 60;
                return `${i === 0 ? 'M' : 'L'} ${x}% ${Math.max(10, Math.min(70, y))}`;
              }).join(' ')} L 100% 80 Z`}
              fill={`url(#gradient-${metric})`}
              strokeWidth="0"
            />

            {/* Line chart */}
            <path
              d={data.map((d, i) => {
                const x = (i / (data.length - 1)) * 100;
                const y = 80 - (((d[metric] as number) - minValue) / range) * 60;
                return `${i === 0 ? 'M' : 'L'} ${x}% ${Math.max(10, Math.min(70, y))}`;
              }).join(' ')}
              fill="none"
              stroke={color}
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />

            {/* Data points */}
            {data.slice(-10).map((d, i) => {
              const dataIndex = data.length - 10 + i;
              const x = (dataIndex / (data.length - 1)) * 100;
              const y = 80 - (((d[metric] as number) - minValue) / range) * 60;
              return (
                <circle
                  key={i}
                  cx={`${x}%`}
                  cy={Math.max(10, Math.min(70, y))}
                  r="2"
                  fill={color}
                  className="opacity-70 hover:opacity-100 hover:r-3 transition-all"
                />
              );
            })}
          </svg>

          {/* Y-axis labels */}
          <div className="absolute left-0 top-0 h-full flex flex-col justify-between text-xs text-muted-foreground -translate-x-8">
            <span>{maxValue.toFixed(0)}{unit}</span>
            <span>{(minValue + range * 0.5).toFixed(0)}{unit}</span>
            <span>{minValue.toFixed(0)}{unit}</span>
          </div>
        </div>

        {/* X-axis time labels */}
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>{formatTime(data[0]?.timestamp)}</span>
          <span>{formatTime(data[Math.floor(data.length / 2)]?.timestamp)}</span>
          <span>{formatTime(data[data.length - 1]?.timestamp)}</span>
        </div>
      </div>
    );
  };

  const chartConfigs = [
    {
      metric: 'attacksBlocked' as const,
      title: 'Attacks Blocked',
      color: '#ef4444',
      target: 30
    },
    {
      metric: 'threatLevel' as const,
      title: 'Threat Level',
      color: '#f97316',
      unit: '%',
      target: 25
    },
    {
      metric: 'systemLoad' as const,
      title: 'System Load',
      color: '#eab308',
      unit: '%',
      target: 80
    },
    {
      metric: 'responseTime' as const,
      title: 'Response Time',
      color: '#06b6d4',
      unit: 'ms',
      target: 15
    },
    {
      metric: 'accuracy' as const,
      title: 'Detection Accuracy',
      color: '#10b981',
      unit: '%',
      target: 99
    },
    {
      metric: 'falsePositives' as const,
      title: 'False Positives',
      color: '#8b5cf6',
      target: 2
    }
  ];

  return (
    <div className="space-y-4">
      {/* Controls */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant={isRealTime ? 'default' : 'outline'}
            onClick={() => setIsRealTime(!isRealTime)}
          >
            {isRealTime ? (
              <>
                <Activity className="w-4 h-4 mr-2" />
                Live
              </>
            ) : (
              <>
                <RefreshCw className="w-4 h-4 mr-2" />
                Paused
              </>
            )}
          </Button>
          {isRealTime && (
            <Badge variant="outline" className="text-green-500 border-green-500">
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse mr-1"></div>
              Real-time
            </Badge>
          )}
        </div>

        {/* Time Range Selector */}
        <div className="flex rounded-lg border p-1">
          {(['1h', '6h', '24h', '7d'] as const).map((range) => (
            <Button
              key={range}
              size="sm"
              variant={timeRange === range ? 'default' : 'ghost'}
              onClick={() => setTimeRange(range)}
              className="h-8 px-3"
            >
              {range}
            </Button>
          ))}
        </div>
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {chartConfigs.map((config) => (
          <Card key={config.metric}>
            <CardContent className="p-4">
              <MetricChart
                data={data}
                {...config}
              />
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Summary Stats */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="w-5 h-5" />
            Security Performance Summary
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-green-500">
                {data.length > 0 ? data[data.length - 1].accuracy.toFixed(1) : '0'}%
              </div>
              <div className="text-sm text-muted-foreground">Detection Accuracy</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-500">
                {data.length > 0 ? data[data.length - 1].responseTime.toFixed(0) : '0'}ms
              </div>
              <div className="text-sm text-muted-foreground">Avg Response Time</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-500">
                {data.reduce((sum, d) => sum + d.attacksBlocked, 0)}
              </div>
              <div className="text-sm text-muted-foreground">Total Blocked</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-yellow-500">
                {data.length > 0 ? data[data.length - 1].systemLoad.toFixed(0) : '0'}%
              </div>
              <div className="text-sm text-muted-foreground">System Load</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}