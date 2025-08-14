'use client';

import { useState } from 'react';
import { 
  ComposedChart, 
  Line, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  ReferenceLine
} from 'recharts';
import { Calendar, TrendingUp, BarChart3 } from 'lucide-react';
import { Card, CardContent, Button, Badge, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui';

interface HistoricalComparisonProps {
  data: any[];
  symbol: string;
  onChartView: (chartType: string) => void;
}

const COMPARISON_PERIODS = [
  { value: 'quarterly', label: 'Quarterly Comparison' },
  { value: 'yearly', label: 'Yearly Comparison' },
  { value: 'rolling', label: 'Rolling 12-Month' }
];

const METRICS = [
  { value: 'eps', label: 'EPS Value' },
  { value: 'growth', label: 'Growth Rate' },
  { value: 'both', label: 'EPS + Growth' }
];

export function HistoricalComparison({ data: _data, symbol, onChartView }: HistoricalComparisonProps) {
  const [comparisonPeriod, setComparisonPeriod] = useState('quarterly');
  const [selectedMetric, setSelectedMetric] = useState('both');

  // Mock historical data for comparison
  const historicalData = [
    { period: '2020 Q1', currentYear: 1.25, previousYear: 1.18, growth: 5.9 },
    { period: '2020 Q2', currentYear: 1.32, previousYear: 1.22, growth: 8.2 },
    { period: '2020 Q3', currentYear: 1.45, previousYear: 1.31, growth: 10.7 },
    { period: '2020 Q4', currentYear: 1.58, previousYear: 1.42, growth: 11.3 },
    { period: '2021 Q1', currentYear: 1.52, previousYear: 1.25, growth: 21.6 },
    { period: '2021 Q2', currentYear: 1.68, previousYear: 1.32, growth: 27.3 },
    { period: '2021 Q3', currentYear: 1.75, previousYear: 1.45, growth: 20.7 },
    { period: '2021 Q4', currentYear: 1.89, previousYear: 1.58, growth: 19.6 },
    { period: '2022 Q1', currentYear: 1.95, previousYear: 1.52, growth: 28.3 },
    { period: '2022 Q2', currentYear: 2.12, previousYear: 1.68, growth: 26.2 },
    { period: '2022 Q3', currentYear: 2.18, previousYear: 1.75, growth: 24.6 },
    { period: '2022 Q4', currentYear: 2.35, previousYear: 1.89, growth: 24.3 }
  ];

  // Calculate comparison statistics
  const stats = {
    avgGrowth: historicalData.reduce((sum, item) => sum + item.growth, 0) / historicalData.length,
    maxGrowth: Math.max(...historicalData.map(item => item.growth)),
    minGrowth: Math.min(...historicalData.map(item => item.growth)),
    consistency: 78, // Mock consistency score
    trend: 'positive'
  };

  const handleMetricChange = (metric: string) => {
    setSelectedMetric(metric);
    onChartView(`historical_${metric}`);
  };

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-background border rounded-lg p-3 shadow-lg">
          <p className="font-medium">{label}</p>
          {payload.map((entry: any, index: number) => (
            <p key={index} className="text-sm" style={{ color: entry.color }}>
              <span>{entry.name}: </span>
              {entry.dataKey === 'growth' 
                ? `${entry.value?.toFixed(1)}%`
                : `$${entry.value?.toFixed(2)}`
              }
            </p>
          ))}
        </div>
      );
    }
    return null;
  };

  const getGrowthTrendBadge = () => {
    const recentGrowth = historicalData.slice(-4).map(item => item.growth);
    const isIncreasing = recentGrowth[recentGrowth.length - 1] > recentGrowth[0];
    
    return (
      <Badge variant={isIncreasing ? 'default' : 'secondary'}>
        {isIncreasing ? 'Accelerating' : 'Decelerating'}
      </Badge>
    );
  };

  return (
    <div className="space-y-4">
      {/* Controls and Statistics */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Calendar className="h-4 w-4" />
          <h3 className="font-medium">Historical Trends - {symbol}</h3>
          {getGrowthTrendBadge()}
        </div>

        <div className="flex items-center gap-2">
          <Select value={comparisonPeriod} onValueChange={setComparisonPeriod}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {COMPARISON_PERIODS.map((period) => (
                <SelectItem key={period.value} value={period.value}>
                  {period.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select value={selectedMetric} onValueChange={handleMetricChange}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {METRICS.map((metric) => (
                <SelectItem key={metric.value} value={metric.value}>
                  {metric.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Statistics Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Card className="p-3">
          <div className="text-xs text-muted-foreground">Avg Growth</div>
          <div className="text-lg font-bold text-green-600">{stats.avgGrowth.toFixed(1)}%</div>
        </Card>
        <Card className="p-3">
          <div className="text-xs text-muted-foreground">Max Growth</div>
          <div className="text-lg font-bold">{stats.maxGrowth.toFixed(1)}%</div>
        </Card>
        <Card className="p-3">
          <div className="text-xs text-muted-foreground">Min Growth</div>
          <div className="text-lg font-bold">{stats.minGrowth.toFixed(1)}%</div>
        </Card>
        <Card className="p-3">
          <div className="text-xs text-muted-foreground">Consistency</div>
          <div className="text-lg font-bold">{stats.consistency}%</div>
        </Card>
      </div>

      {/* Historical Chart */}
      <Card className="p-4">
        <ResponsiveContainer width="100%" height={400}>
          <ComposedChart data={historicalData} margin={{ top: 20, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis dataKey="period" className="text-xs" />
            <YAxis yAxisId="left" className="text-xs" />
            <YAxis yAxisId="right" orientation="right" className="text-xs" />
            <Tooltip content={<CustomTooltip />} />
            
            {/* EPS Values */}
            {(selectedMetric === 'eps' || selectedMetric === 'both') && (
              <>
                <Bar 
                  yAxisId="left"
                  dataKey="currentYear" 
                  fill="hsl(var(--primary))" 
                  name="Current Year EPS"
                  opacity={0.7}
                />
                <Bar 
                  yAxisId="left"
                  dataKey="previousYear" 
                  fill="hsl(var(--muted-foreground))" 
                  name="Previous Year EPS"
                  opacity={0.5}
                />
              </>
            )}
            
            {/* Growth Rate */}
            {(selectedMetric === 'growth' || selectedMetric === 'both') && (
              <>
                <Line 
                  yAxisId="right"
                  type="monotone" 
                  dataKey="growth" 
                  stroke="hsl(var(--secondary))" 
                  strokeWidth={3}
                  name="YoY Growth %"
                  dot={{ fill: 'hsl(var(--secondary))', strokeWidth: 2, r: 4 }}
                />
                <ReferenceLine 
                  yAxisId="right"
                  y={stats.avgGrowth} 
                  stroke="hsl(var(--muted-foreground))" 
                  strokeDasharray="5 5" 
                  label="Avg Growth"
                />
              </>
            )}
          </ComposedChart>
        </ResponsiveContainer>
      </Card>

      {/* Insights and Patterns */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardContent className="p-4">
            <h4 className="font-medium mb-3 flex items-center gap-2">
              <TrendingUp className="h-4 w-4" />
              Growth Patterns
            </h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Q4 Seasonality:</span>
                <span className="font-medium text-green-600">Strong</span>
              </div>
              <div className="flex justify-between">
                <span>Growth Acceleration:</span>
                <span className="font-medium">2021-2022</span>
              </div>
              <div className="flex justify-between">
                <span>Volatility Trend:</span>
                <span className="font-medium text-blue-600">Decreasing</span>
              </div>
              <div className="flex justify-between">
                <span>Consistency Score:</span>
                <span className="font-medium">{stats.consistency}%</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <h4 className="font-medium mb-3 flex items-center gap-2">
              <BarChart3 className="h-4 w-4" />
              Performance Analysis
            </h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Best Quarter:</span>
                <span className="font-medium">2022 Q1 (+28.3%)</span>
              </div>
              <div className="flex justify-between">
                <span>Worst Quarter:</span>
                <span className="font-medium">2020 Q1 (+5.9%)</span>
              </div>
              <div className="flex justify-between">
                <span>Streak Length:</span>
                <span className="font-medium text-green-600">12 quarters positive</span>
              </div>
              <div className="flex justify-between">
                <span>Above Average:</span>
                <span className="font-medium">75% of quarters</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-2">
        <Button variant="outline" onClick={() => onChartView('export_historical')}>
          Export Historical Data
        </Button>
        <Button variant="outline" onClick={() => onChartView('compare_peers')}>
          Compare with Peers
        </Button>
        <Button variant="outline" onClick={() => onChartView('forecast_model')}>
          View Forecast Model
        </Button>
      </div>
    </div>
  );
}