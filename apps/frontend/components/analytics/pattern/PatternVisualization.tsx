'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui';
import { 
  ScatterChart, 
  Scatter, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  BarChart,
  Bar,
  LineChart,
  Line
} from 'recharts';
import { 
  TrendingUp, 
  PieChart as PieChartIcon, 
  BarChart3, 
  ScatterChart as ScatterIcon,
  Eye,
  Download
} from 'lucide-react';

interface PatternVisualizationProps {
  patterns: any[];
}

const CHART_TYPES = [
  { value: 'distribution', label: 'Pattern Distribution', icon: PieChartIcon },
  { value: 'confidence', label: 'Confidence Analysis', icon: BarChart3 },
  { value: 'timeline', label: 'Detection Timeline', icon: TrendingUp },
  { value: 'scatter', label: 'Risk vs Confidence', icon: ScatterIcon }
];

const COLORS = {
  bullish: '#10b981',
  bearish: '#ef4444',
  neutral: '#6b7280',
  breakout: '#3b82f6',
  reversal: '#8b5cf6',
  trend: '#f59e0b',
  continuation: '#06b6d4'
};

export function PatternVisualization({ patterns }: PatternVisualizationProps) {
  const [selectedChart, setSelectedChart] = useState('distribution');

  // Prepare data for different chart types
  const getDistributionData = () => {
    const counts = patterns.reduce((acc, pattern) => {
      acc[pattern.type] = (acc[pattern.type] || 0) + 1;
      return acc;
    }, {});

    return Object.entries(counts).map(([type, count]) => ({
      name: type.charAt(0).toUpperCase() + type.slice(1),
      value: count,
      fill: COLORS[type as keyof typeof COLORS] || COLORS.neutral
    }));
  };

  const getConfidenceData = () => {
    const ranges = [
      { range: '90-100%', min: 90, max: 100 },
      { range: '80-89%', min: 80, max: 89 },
      { range: '70-79%', min: 70, max: 79 },
      { range: '60-69%', min: 60, max: 69 },
      { range: '50-59%', min: 50, max: 59 }
    ];

    return ranges.map(({ range, min, max }) => ({
      range,
      count: patterns.filter(p => p.confidence >= min && p.confidence <= max).length,
      bullish: patterns.filter(p => p.confidence >= min && p.confidence <= max && p.direction === 'bullish').length,
      bearish: patterns.filter(p => p.confidence >= min && p.confidence <= max && p.direction === 'bearish').length
    }));
  };

  const getTimelineData = () => {
    const last7Days = Array.from({ length: 7 }, (_, i) => {
      const date = new Date();
      date.setDate(date.getDate() - i);
      return date.toISOString().split('T')[0];
    }).reverse();

    return last7Days.map(date => ({
      date: new Date(date).toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' }),
      patterns: Math.floor(Math.random() * 10) + 1,
      confidence: 70 + Math.random() * 20
    }));
  };

  const getScatterData = () => {
    return patterns.map(pattern => ({
      confidence: pattern.confidence,
      risk: pattern.riskLevel === 'low' ? 1 : pattern.riskLevel === 'medium' ? 2 : 3,
      symbol: pattern.symbol,
      type: pattern.type,
      direction: pattern.direction
    }));
  };

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-background border rounded-lg p-3 shadow-lg">
          <p className="font-medium">{label}</p>
          {payload.map((entry: any, index: number) => (
            <p key={index} className="text-sm" style={{ color: entry.color }}>
              <span>{entry.name}: {entry.value}</span>
            </p>
          ))}
        </div>
      );
    }
    return null;
  };

  const renderChart = () => {
    switch (selectedChart) {
      case 'distribution':
        return (
          <ResponsiveContainer width="100%" height={400}>
            <PieChart>
              <Pie
                data={getDistributionData()}
                cx="50%"
                cy="50%"
                outerRadius={120}
                dataKey="value"
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
              >
                {getDistributionData().map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.fill} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        );

      case 'confidence':
        return (
          <ResponsiveContainer width="100%" height={400}>
            <BarChart data={getConfidenceData()}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="range" />
              <YAxis />
              <Tooltip content={<CustomTooltip />} />
              <Bar dataKey="bullish" stackId="a" fill={COLORS.bullish} name="Bullish" />
              <Bar dataKey="bearish" stackId="a" fill={COLORS.bearish} name="Bearish" />
            </BarChart>
          </ResponsiveContainer>
        );

      case 'timeline':
        return (
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={getTimelineData()}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis yAxisId="left" />
              <YAxis yAxisId="right" orientation="right" />
              <Tooltip content={<CustomTooltip />} />
              <Bar yAxisId="left" dataKey="patterns" fill={COLORS.breakout} name="Patterns Detected" />
              <Line 
                yAxisId="right" 
                type="monotone" 
                dataKey="confidence" 
                stroke={COLORS.trend} 
                strokeWidth={3}
                name="Avg Confidence"
              />
            </LineChart>
          </ResponsiveContainer>
        );

      case 'scatter':
        return (
          <ResponsiveContainer width="100%" height={400}>
            <ScatterChart data={getScatterData()}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="confidence" name="Confidence" unit="%" />
              <YAxis dataKey="risk" name="Risk Level" domain={[0.5, 3.5]} 
                     tickFormatter={(value) => ['', 'Low', 'Med', 'High'][value]} />
              <Tooltip 
                formatter={(value, name) => [
                  name === 'risk' ? ['Low', 'Medium', 'High'][value - 1] : `${value}%`,
                  name === 'risk' ? 'Risk Level' : 'Confidence'
                ]}
                labelFormatter={(_, payload) => payload?.[0]?.payload?.symbol}
              />
              <Scatter 
                dataKey="risk" 
                fill={COLORS.breakout}
                onClick={(_data) => {/* Pattern clicked */}}
              />
            </ScatterChart>
          </ResponsiveContainer>
        );

      default:
        return null;
    }
  };

  const getChartInsights = () => {
    switch (selectedChart) {
      case 'distribution':
        const totalPatterns = patterns.length;
        const bullishCount = patterns.filter(p => p.direction === 'bullish').length;
        const bearishCount = patterns.filter(p => p.direction === 'bearish').length;
        return {
          title: 'Pattern Distribution Insights',
          insights: [
            `Total patterns detected: ${totalPatterns}`,
            `Bullish sentiment: ${((bullishCount / totalPatterns) * 100).toFixed(1)}%`,
            `Bearish sentiment: ${((bearishCount / totalPatterns) * 100).toFixed(1)}%`,
            `Most common pattern: ${Object.entries(patterns.reduce((acc, p) => {
              acc[p.type] = (acc[p.type] || 0) + 1;
              return acc;
            }, {})).sort(([,a], [,b]) => b - a)[0]?.[0] || 'N/A'}`
          ]
        };

      case 'confidence':
        const avgConfidence = patterns.reduce((sum, p) => sum + p.confidence, 0) / patterns.length;
        const highConfidence = patterns.filter(p => p.confidence >= 80).length;
        return {
          title: 'Confidence Analysis',
          insights: [
            `Average confidence: ${avgConfidence.toFixed(1)}%`,
            `High confidence patterns (80%+): ${highConfidence}`,
            `Confidence range: ${Math.min(...patterns.map(p => p.confidence))}% - ${Math.max(...patterns.map(p => p.confidence))}%`,
            `Most reliable direction: ${patterns.filter(p => p.confidence >= 80).reduce((acc, p) => {
              acc[p.direction] = (acc[p.direction] || 0) + 1;
              return acc;
            }, {}) ? Object.entries(patterns.filter(p => p.confidence >= 80).reduce((acc, p) => {
              acc[p.direction] = (acc[p.direction] || 0) + 1;
              return acc;
            }, {})).sort(([,a], [,b]) => b - a)[0]?.[0] : 'N/A'}`
          ]
        };

      default:
        return {
          title: 'Insights',
          insights: ['Select different chart types to view specific insights']
        };
    }
  };

  const insights = getChartInsights();

  return (
    <div className="space-y-6">
      {/* Chart Controls */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Select value={selectedChart} onValueChange={setSelectedChart}>
                <SelectTrigger className="w-48">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {CHART_TYPES.map((chart) => (
                    <SelectItem key={chart.value} value={chart.value}>
                      <div className="flex items-center gap-2">
                        <chart.icon className="h-4 w-4" />
                        {chart.label}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              
              <Badge variant="outline">
                {patterns.length} patterns
              </Badge>
            </div>
            
            <div className="flex gap-2">
              <Button variant="outline" size="sm">
                <Eye className="h-4 w-4 mr-2" />
                View Raw Data
              </Button>
              <Button variant="outline" size="sm">
                <Download className="h-4 w-4 mr-2" />
                Export Chart
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Main Visualization */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {(() => {
                const chartType = CHART_TYPES.find(c => c.value === selectedChart);
                const IconComponent = chartType?.icon;
                return IconComponent ? <IconComponent className="h-5 w-5" /> : null;
              })()}
              {CHART_TYPES.find(c => c.value === selectedChart)?.label}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {renderChart()}
          </CardContent>
        </Card>

        {/* Insights Panel */}
        <Card>
          <CardHeader>
            <CardTitle>{insights.title}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {insights.insights.map((insight, index) => (
                <div key={index} className="p-3 bg-muted rounded-lg">
                  <p className="text-sm">{insight}</p>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Pattern Legend */}
      <Card>
        <CardHeader>
          <CardTitle>Pattern Legend</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {Object.entries(COLORS).map(([type, color]) => (
              <div key={type} className="flex items-center gap-2">
                <div 
                  className="w-4 h-4 rounded-full" 
                  style={{ backgroundColor: color }}
                />
                <span className="text-sm capitalize">{type}</span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Quick Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4 text-center">
            <div className="text-2xl font-bold text-green-600">
              {patterns.filter(p => p.direction === 'bullish').length}
            </div>
            <div className="text-sm text-muted-foreground">Bullish Patterns</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4 text-center">
            <div className="text-2xl font-bold text-red-600">
              {patterns.filter(p => p.direction === 'bearish').length}
            </div>
            <div className="text-sm text-muted-foreground">Bearish Patterns</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4 text-center">
            <div className="text-2xl font-bold text-blue-600">
              {patterns.filter(p => p.confidence >= 80).length}
            </div>
            <div className="text-sm text-muted-foreground">High Confidence</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4 text-center">
            <div className="text-2xl font-bold">
              {(patterns.reduce((sum, p) => sum + p.confidence, 0) / patterns.length).toFixed(0)}%
            </div>
            <div className="text-sm text-muted-foreground">Avg Confidence</div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}