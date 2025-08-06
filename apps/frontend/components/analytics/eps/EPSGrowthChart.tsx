'use client';

import { useState, useMemo } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  LineChart, 
  Line, 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  ReferenceLine,
  Area,
  AreaChart
} from 'recharts';
import { TrendingUp, TrendingDown, BarChart3, LineChart as LineChartIcon, AreaChart as AreaChartIcon } from 'lucide-react';

interface EPSGrowthChartProps {
  data: any[];
  symbol: string;
  onChartView: (chartType: string) => void;
}

interface ChartDataPoint {
  period: string;
  eps: number;
  growth: number;
  quarter: string;
  year: number;
}

const CHART_TYPES = [
  { value: 'line', label: 'Line Chart', icon: LineChartIcon },
  { value: 'bar', label: 'Bar Chart', icon: BarChart3 },
  { value: 'area', label: 'Area Chart', icon: AreaChartIcon }
];

const TIME_RANGES = [
  { value: 'all', label: 'All Data' },
  { value: '1y', label: 'Last 1 Year' },
  { value: '2y', label: 'Last 2 Years' },
  { value: '3y', label: 'Last 3 Years' }
];

export function EPSGrowthChart({ data, symbol, onChartView }: EPSGrowthChartProps) {
  const [chartType, setChartType] = useState('line');
  const [timeRange, setTimeRange] = useState('all');
  const [showGrowthRate, setShowGrowthRate] = useState(false);

  // Process and filter data based on time range
  const chartData = useMemo(() => {
    if (!data || data.length === 0) {
      // Mock data for development
      return [
        { period: 'Q1 2022', eps: 1.52, growth: 8.5, quarter: 'Q1', year: 2022 },
        { period: 'Q2 2022', eps: 1.68, growth: 10.5, quarter: 'Q2', year: 2022 },
        { period: 'Q3 2022', eps: 1.75, growth: 4.2, quarter: 'Q3', year: 2022 },
        { period: 'Q4 2022', eps: 1.89, growth: 8.0, quarter: 'Q4', year: 2022 },
        { period: 'Q1 2023', eps: 1.95, growth: 28.3, quarter: 'Q1', year: 2023 },
        { period: 'Q2 2023', eps: 2.12, growth: 26.2, quarter: 'Q2', year: 2023 },
        { period: 'Q3 2023', eps: 2.18, growth: 24.6, quarter: 'Q3', year: 2023 },
        { period: 'Q4 2023', eps: 2.35, growth: 24.3, quarter: 'Q4', year: 2023 },
        { period: 'Q1 2024', eps: 2.45, growth: 25.6, quarter: 'Q1', year: 2024 },
        { period: 'Q2 2024', eps: 2.58, growth: 21.7, quarter: 'Q2', year: 2024 }
      ];
    }

    let filteredData = [...data];
    
    if (timeRange !== 'all') {
      const currentYear = new Date().getFullYear();
      const yearsBack = parseInt(timeRange.replace('y', ''));
      const cutoffYear = currentYear - yearsBack;
      filteredData = data.filter((item: ChartDataPoint) => item.year >= cutoffYear);
    }

    return filteredData;
  }, [data, timeRange]);

  // Calculate statistics
  const statistics = useMemo(() => {
    if (chartData.length === 0) return null;

    const epsValues = chartData.map(d => d.eps);
    const growthValues = chartData.map(d => d.growth);
    
    const currentEPS = epsValues[epsValues.length - 1];
    const previousEPS = epsValues[epsValues.length - 2];
    const avgGrowth = growthValues.reduce((a, b) => a + b, 0) / growthValues.length;
    const maxEPS = Math.max(...epsValues);
    const minEPS = Math.min(...epsValues);

    return {
      current: currentEPS,
      previous: previousEPS,
      change: previousEPS ? ((currentEPS - previousEPS) / previousEPS * 100) : 0,
      avgGrowth: avgGrowth,
      max: maxEPS,
      min: minEPS,
      trend: avgGrowth > 0 ? 'up' : 'down'
    };
  }, [chartData]);

  const handleChartTypeChange = (type: string) => {
    setChartType(type);
    onChartView(`${type}_chart`);
  };

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-background border rounded-lg p-3 shadow-lg">
          <p className="font-medium">{label}</p>
          <p className="text-sm">
            <span className="text-primary">EPS: </span>
            ${payload[0]?.value?.toFixed(2)}
          </p>
          {payload[1] && (
            <p className="text-sm">
              <span className="text-green-600">Growth: </span>
              {payload[1]?.value?.toFixed(1)}%
            </p>
          )}
        </div>
      );
    }
    return null;
  };

  const renderChart = () => {
    const commonProps = {
      data: chartData,
      margin: { top: 5, right: 30, left: 20, bottom: 5 }
    };

    switch (chartType) {
      case 'bar':
        return (
          <BarChart {...commonProps}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis dataKey="period" className="text-xs" />
            <YAxis className="text-xs" />
            <Tooltip content={<CustomTooltip />} />
            <Bar dataKey="eps" fill="hsl(var(--primary))" radius={[2, 2, 0, 0]} />
            {showGrowthRate && (
              <Bar dataKey="growth" fill="hsl(var(--secondary))" radius={[2, 2, 0, 0]} />
            )}
          </BarChart>
        );
      
      case 'area':
        return (
          <AreaChart {...commonProps}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis dataKey="period" className="text-xs" />
            <YAxis className="text-xs" />
            <Tooltip content={<CustomTooltip />} />
            <Area 
              type="monotone" 
              dataKey="eps" 
              stroke="hsl(var(--primary))" 
              fill="hsl(var(--primary))" 
              fillOpacity={0.3}
            />
            {showGrowthRate && (
              <Area 
                type="monotone" 
                dataKey="growth" 
                stroke="hsl(var(--secondary))" 
                fill="hsl(var(--secondary))" 
                fillOpacity={0.2}
              />
            )}
          </AreaChart>
        );
      
      default:
        return (
          <LineChart {...commonProps}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis dataKey="period" className="text-xs" />
            <YAxis className="text-xs" />
            <Tooltip content={<CustomTooltip />} />
            <Line 
              type="monotone" 
              dataKey="eps" 
              stroke="hsl(var(--primary))" 
              strokeWidth={2}
              dot={{ fill: 'hsl(var(--primary))', strokeWidth: 2, r: 4 }}
            />
            {showGrowthRate && (
              <Line 
                type="monotone" 
                dataKey="growth" 
                stroke="hsl(var(--secondary))" 
                strokeWidth={2}
                dot={{ fill: 'hsl(var(--secondary))', strokeWidth: 2, r: 4 }}
              />
            )}
            {statistics && (
              <ReferenceLine 
                y={statistics.avgGrowth} 
                stroke="hsl(var(--muted-foreground))" 
                strokeDasharray="5 5" 
                label="Avg Growth"
              />
            )}
          </LineChart>
        );
    }
  };

  return (
    <div className="space-y-4">
      {/* Statistics Cards */}
      {statistics && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <Card className="p-3">
            <div className="text-xs text-muted-foreground">Current EPS</div>
            <div className="text-lg font-bold">${statistics.current.toFixed(2)}</div>
          </Card>
          <Card className="p-3">
            <div className="text-xs text-muted-foreground">QoQ Change</div>
            <div className={`text-lg font-bold flex items-center gap-1 ${
              statistics.change >= 0 ? 'text-green-600' : 'text-red-600'
            }`}>
              {statistics.change >= 0 ? <TrendingUp className="h-4 w-4" /> : <TrendingDown className="h-4 w-4" />}
              {statistics.change.toFixed(1)}%
            </div>
          </Card>
          <Card className="p-3">
            <div className="text-xs text-muted-foreground">Avg Growth</div>
            <div className="text-lg font-bold">{statistics.avgGrowth.toFixed(1)}%</div>
          </Card>
          <Card className="p-3">
            <div className="text-xs text-muted-foreground">Range</div>
            <div className="text-sm font-medium">
              ${statistics.min.toFixed(2)} - ${statistics.max.toFixed(2)}
            </div>
          </Card>
        </div>
      )}

      {/* Chart Controls */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <h3 className="font-medium">EPS Growth - {symbol}</h3>
          <Badge variant={statistics?.trend === 'up' ? 'default' : 'destructive'}>
            {statistics?.trend === 'up' ? 'Trending Up' : 'Trending Down'}
          </Badge>
        </div>

        <div className="flex items-center gap-2">
          <Select value={timeRange} onValueChange={setTimeRange}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {TIME_RANGES.map((range) => (
                <SelectItem key={range.value} value={range.value}>
                  {range.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select value={chartType} onValueChange={handleChartTypeChange}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {CHART_TYPES.map((type) => (
                <SelectItem key={type.value} value={type.value}>
                  <div className="flex items-center gap-2">
                    <type.icon className="h-4 w-4" />
                    {type.label}
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Button
            variant={showGrowthRate ? 'default' : 'outline'}
            size="sm"
            onClick={() => setShowGrowthRate(!showGrowthRate)}
          >
            Growth Rate
          </Button>
        </div>
      </div>

      {/* Chart */}
      <Card className="p-4">
        <ResponsiveContainer width="100%" height={400}>
          {renderChart()}
        </ResponsiveContainer>
      </Card>
    </div>
  );
}