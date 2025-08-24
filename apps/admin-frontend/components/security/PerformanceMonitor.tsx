'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { SecurityMetricsChart } from './SecurityMetricsChart';
import { cn } from '@/lib/utils';
import {
  Gauge,
  Activity,
  Cpu,
  HardDrive,
  Network,
  Clock,
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  CheckCircle,
  RefreshCw,
  Settings,
  BarChart3,
  PieChart,
  Target,
  Zap
} from 'lucide-react';

interface PerformanceMetric {
  name: string;
  value: number;
  unit: string;
  trend: 'up' | 'down' | 'stable';
  status: 'healthy' | 'warning' | 'critical';
  threshold: { warning: number; critical: number };
  description: string;
}

interface SystemResource {
  component: string;
  usage: number;
  capacity: number;
  status: 'optimal' | 'moderate' | 'high' | 'critical';
  recommendations: string[];
}

interface SLAMetric {
  service: string;
  availability: number;
  responseTime: number;
  errorRate: number;
  target: { availability: number; responseTime: number; errorRate: number };
  status: 'meeting' | 'warning' | 'breach';
}

export function PerformanceMonitor() {
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('24h');
  const [selectedMetric, setSelectedMetric] = useState<string | null>(null);

  const [performanceMetrics, setPerformanceMetrics] = useState<PerformanceMetric[]>([
    {
      name: 'Threat Detection Speed',
      value: 12.5,
      unit: 'ms',
      trend: 'down',
      status: 'healthy',
      threshold: { warning: 50, critical: 100 },
      description: 'Average time to detect and classify security threats'
    },
    {
      name: 'False Positive Rate',
      value: 0.8,
      unit: '%',
      trend: 'down',
      status: 'healthy',
      threshold: { warning: 2, critical: 5 },
      description: 'Percentage of legitimate activities flagged as threats'
    },
    {
      name: 'Alert Processing Time',
      value: 2.3,
      unit: 's',
      trend: 'stable',
      status: 'healthy',
      threshold: { warning: 5, critical: 10 },
      description: 'Time from threat detection to alert generation'
    },
    {
      name: 'Security Engine Load',
      value: 67,
      unit: '%',
      trend: 'up',
      status: 'warning',
      threshold: { warning: 70, critical: 90 },
      description: 'Current load on security processing engines'
    },
    {
      name: 'Rule Execution Time',
      value: 45.2,
      unit: 'ms',
      trend: 'up',
      status: 'warning',
      threshold: { warning: 50, critical: 100 },
      description: 'Average time to execute security rules'
    },
    {
      name: 'Data Throughput',
      value: 1247,
      unit: 'MB/s',
      trend: 'stable',
      status: 'healthy',
      threshold: { warning: 1500, critical: 2000 },
      description: 'Security data processing throughput'
    }
  ]);

  const [systemResources, setSystemResources] = useState<SystemResource[]>([
    {
      component: 'Security Engine CPU',
      usage: 67,
      capacity: 100,
      status: 'moderate',
      recommendations: ['Scale horizontally', 'Optimize detection algorithms']
    },
    {
      component: 'Threat Intelligence Memory',
      usage: 8.2,
      capacity: 16,
      status: 'optimal',
      recommendations: []
    },
    {
      component: 'Alert Queue Storage',
      usage: 245,
      capacity: 1000,
      status: 'optimal',
      recommendations: []
    },
    {
      component: 'Network Bandwidth',
      usage: 890,
      capacity: 1000,
      status: 'high',
      recommendations: ['Increase bandwidth capacity', 'Implement traffic shaping']
    },
    {
      component: 'Database Connections',
      usage: 85,
      capacity: 100,
      status: 'high',
      recommendations: ['Optimize connection pooling', 'Scale database']
    }
  ]);

  const [slaMetrics, setSlaMetrics] = useState<SLAMetric[]>([
    {
      service: 'Threat Detection Service',
      availability: 99.97,
      responseTime: 12.5,
      errorRate: 0.1,
      target: { availability: 99.9, responseTime: 50, errorRate: 1 },
      status: 'meeting'
    },
    {
      service: 'Alert Notification System',
      availability: 99.85,
      responseTime: 2.3,
      errorRate: 0.5,
      target: { availability: 99.9, responseTime: 5, errorRate: 1 },
      status: 'warning'
    },
    {
      service: 'Security Dashboard',
      availability: 99.99,
      responseTime: 180,
      errorRate: 0.2,
      target: { availability: 99.9, responseTime: 200, errorRate: 0.5 },
      status: 'meeting'
    },
    {
      service: 'API Gateway Security',
      availability: 99.92,
      responseTime: 25,
      errorRate: 0.3,
      target: { availability: 99.9, responseTime: 30, errorRate: 0.5 },
      status: 'meeting'
    }
  ]);

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setPerformanceMetrics(prev => prev.map(metric => ({
        ...metric,
        value: Math.max(0, metric.value + (Math.random() - 0.5) * (metric.value * 0.1)),
        trend: Math.random() > 0.7 ? (Math.random() > 0.5 ? 'up' : 'down') : metric.trend
      })));

      setSystemResources(prev => prev.map(resource => ({
        ...resource,
        usage: Math.max(0, Math.min(resource.capacity, 
          resource.usage + (Math.random() - 0.5) * (resource.usage * 0.05)
        ))
      })));
    }, 10000); // Update every 10 seconds

    return () => clearInterval(interval);
  }, []);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await new Promise(resolve => setTimeout(resolve, 1000));
    setIsRefreshing(false);
  };

  const getMetricStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return 'text-green-500 bg-green-500/10 border-green-500/20';
      case 'warning': return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/20';
      case 'critical': return 'text-red-500 bg-red-500/10 border-red-500/20';
      default: return 'text-gray-500 bg-gray-500/10 border-gray-500/20';
    }
  };

  const getResourceStatusColor = (status: string) => {
    switch (status) {
      case 'optimal': return 'text-green-500';
      case 'moderate': return 'text-yellow-500';
      case 'high': return 'text-orange-500';
      case 'critical': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  const getSLAStatusColor = (status: string) => {
    switch (status) {
      case 'meeting': return 'text-green-500 bg-green-500/10';
      case 'warning': return 'text-yellow-500 bg-yellow-500/10';
      case 'breach': return 'text-red-500 bg-red-500/10';
      default: return 'text-gray-500 bg-gray-500/10';
    }
  };

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'up': return TrendingUp;
      case 'down': return TrendingDown;
      default: return Activity;
    }
  };

  const getTrendColor = (trend: string, isGoodTrend: boolean = true) => {
    if (trend === 'stable') return 'text-blue-500';
    const isPositive = trend === 'up';
    return (isPositive === isGoodTrend) ? 'text-green-500' : 'text-red-500';
  };

  const formatValue = (value: number, unit: string) => {
    if (unit === '%') return `${value.toFixed(1)}${unit}`;
    if (unit === 'ms' || unit === 's') return `${value.toFixed(1)}${unit}`;
    if (unit === 'MB/s') return `${Math.round(value)}${unit}`;
    return `${value.toFixed(1)}${unit}`;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Security Performance Monitor</h1>
          <p className="text-muted-foreground mt-1">
            Monitor security system performance, capacity, and SLA compliance
          </p>
        </div>
        <div className="flex items-center gap-2">
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
          <Button 
            variant="outline" 
            size="sm" 
            onClick={handleRefresh}
            disabled={isRefreshing}
          >
            <RefreshCw className={cn("w-4 h-4 mr-2", isRefreshing && "animate-spin")} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Performance Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {performanceMetrics.map((metric) => {
          const TrendIcon = getTrendIcon(metric.trend);
          const isGoodTrend = metric.name.includes('False Positive') || 
                             metric.name.includes('Processing Time') || 
                             metric.name.includes('Response Time') ? false : true;
          
          return (
            <Card 
              key={metric.name}
              className={cn(
                "cursor-pointer transition-all hover:shadow-md",
                selectedMetric === metric.name && "ring-2 ring-primary",
                metric.status === 'critical' && "ring-2 ring-red-500/20"
              )}
              onClick={() => setSelectedMetric(selectedMetric === metric.name ? null : metric.name)}
            >
              <CardContent className="p-4">
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-sm">{metric.name}</h4>
                  <Badge className={cn("text-xs", getMetricStatusColor(metric.status))}>
                    {metric.status}
                  </Badge>
                </div>
                
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-2xl font-bold">
                    {formatValue(metric.value, metric.unit)}
                  </span>
                  <div className={cn(
                    "flex items-center gap-1",
                    getTrendColor(metric.trend, isGoodTrend)
                  )}>
                    <TrendIcon className="w-4 h-4" />
                  </div>
                </div>
                
                <p className="text-xs text-muted-foreground leading-relaxed">
                  {metric.description}
                </p>
                
                {/* Threshold indicators */}
                <div className="mt-3 flex items-center gap-1">
                  <div className="flex-1 bg-muted rounded-full h-1.5 overflow-hidden">
                    <div 
                      className={cn(
                        "h-full transition-all",
                        metric.value >= metric.threshold.critical ? "bg-red-500" :
                        metric.value >= metric.threshold.warning ? "bg-yellow-500" : "bg-green-500"
                      )}
                      style={{ 
                        width: `${Math.min(100, (metric.value / metric.threshold.critical) * 100)}%` 
                      }}
                    />
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {formatValue(metric.threshold.warning, metric.unit)} / {formatValue(metric.threshold.critical, metric.unit)}
                  </span>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Main Content Tabs */}
      <Tabs defaultValue="metrics" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="metrics">Real-time Metrics</TabsTrigger>
          <TabsTrigger value="resources">System Resources</TabsTrigger>
          <TabsTrigger value="sla">SLA Monitoring</TabsTrigger>
          <TabsTrigger value="capacity">Capacity Planning</TabsTrigger>
        </TabsList>

        {/* Real-time Metrics */}
        <TabsContent value="metrics" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <BarChart3 className="w-5 h-5" />
                Security Performance Dashboard
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Real-time performance metrics and trends for security systems
              </p>
            </CardHeader>
            <CardContent>
              <SecurityMetricsChart />
            </CardContent>
          </Card>

          {selectedMetric && (
            <Card>
              <CardHeader>
                <CardTitle>Metric Details: {selectedMetric}</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div>
                    <h4 className="font-semibold mb-3">Current Status</h4>
                    <div className="space-y-2">
                      {performanceMetrics
                        .filter(m => m.name === selectedMetric)
                        .map(metric => (
                          <div key="current" className="space-y-2">
                            <div className="flex justify-between">
                              <span>Current Value:</span>
                              <span className="font-mono">
                                {formatValue(metric.value, metric.unit)}
                              </span>
                            </div>
                            <div className="flex justify-between">
                              <span>Warning Threshold:</span>
                              <span className="font-mono text-yellow-500">
                                {formatValue(metric.threshold.warning, metric.unit)}
                              </span>
                            </div>
                            <div className="flex justify-between">
                              <span>Critical Threshold:</span>
                              <span className="font-mono text-red-500">
                                {formatValue(metric.threshold.critical, metric.unit)}
                              </span>
                            </div>
                          </div>
                        ))}
                    </div>
                  </div>
                  
                  <div>
                    <h4 className="font-semibold mb-3">Optimization Recommendations</h4>
                    <ul className="text-sm space-y-2">
                      <li className="flex items-start gap-2">
                        <Target className="w-4 h-4 mt-0.5 text-blue-500" />
                        <span>Monitor trend patterns over time</span>
                      </li>
                      <li className="flex items-start gap-2">
                        <Settings className="w-4 h-4 mt-0.5 text-green-500" />
                        <span>Configure automated alerting for thresholds</span>
                      </li>
                      <li className="flex items-start gap-2">
                        <Zap className="w-4 h-4 mt-0.5 text-orange-500" />
                        <span>Consider capacity planning for projected growth</span>
                      </li>
                    </ul>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        {/* System Resources */}
        <TabsContent value="resources" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Cpu className="w-5 h-5" />
                System Resource Utilization
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Monitor system resources and capacity across security infrastructure
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {systemResources.map((resource) => (
                  <div key={resource.component} className="flex items-center gap-4 p-4 border rounded-lg">
                    <div className="p-2 rounded-full bg-blue-500/10">
                      <Activity className="w-5 h-5 text-blue-500" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <h4 className="font-medium">{resource.component}</h4>
                        <Badge 
                          variant="outline" 
                          className={cn("text-xs", getResourceStatusColor(resource.status))}
                        >
                          {resource.status}
                        </Badge>
                      </div>
                      
                      <div className="flex items-center gap-4 mb-2">
                        <div className="flex-1 bg-muted rounded-full h-2 overflow-hidden">
                          <div 
                            className={cn(
                              "h-full transition-all",
                              resource.status === 'critical' ? "bg-red-500" :
                              resource.status === 'high' ? "bg-orange-500" :
                              resource.status === 'moderate' ? "bg-yellow-500" : "bg-green-500"
                            )}
                            style={{ width: `${(resource.usage / resource.capacity) * 100}%` }}
                          />
                        </div>
                        <span className="text-sm font-mono min-w-0">
                          {resource.usage.toFixed(1)} / {resource.capacity} 
                          {resource.component.includes('GB') ? 'GB' : 
                           resource.component.includes('Memory') ? 'GB' : 
                           resource.component.includes('Storage') ? 'GB' :
                           resource.component.includes('Bandwidth') ? 'Mbps' : '%'}
                        </span>
                      </div>
                      
                      {resource.recommendations.length > 0 && (
                        <div className="text-xs text-muted-foreground">
                          Recommendations: {resource.recommendations.join(', ')}
                        </div>
                      )}
                    </div>
                    
                    <div className="text-right">
                      <div className={cn("text-lg font-bold", getResourceStatusColor(resource.status))}>
                        {Math.round((resource.usage / resource.capacity) * 100)}%
                      </div>
                      <div className="text-xs text-muted-foreground">Utilized</div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* SLA Monitoring */}
        <TabsContent value="sla" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Target className="w-5 h-5" />
                Service Level Agreement Monitoring
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Track SLA compliance across all security services
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {slaMetrics.map((sla) => (
                  <div key={sla.service} className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between mb-3">
                      <h4 className="font-medium">{sla.service}</h4>
                      <Badge className={cn("text-xs", getSLAStatusColor(sla.status))}>
                        {sla.status === 'meeting' ? 'SLA Met' : 
                         sla.status === 'warning' ? 'SLA Warning' : 'SLA Breach'}
                      </Badge>
                    </div>
                    
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                      <div className="text-center">
                        <div className="text-2xl font-bold text-green-500">
                          {sla.availability.toFixed(2)}%
                        </div>
                        <div className="text-sm text-muted-foreground">
                          Availability (Target: {sla.target.availability}%)
                        </div>
                        <div className="mt-1">
                          {sla.availability >= sla.target.availability ? (
                            <CheckCircle className="w-4 h-4 text-green-500 mx-auto" />
                          ) : (
                            <AlertTriangle className="w-4 h-4 text-red-500 mx-auto" />
                          )}
                        </div>
                      </div>
                      
                      <div className="text-center">
                        <div className={cn(
                          "text-2xl font-bold",
                          sla.responseTime <= sla.target.responseTime ? "text-green-500" : "text-red-500"
                        )}>
                          {sla.responseTime.toFixed(1)}ms
                        </div>
                        <div className="text-sm text-muted-foreground">
                          Response Time (Target: ≤{sla.target.responseTime}ms)
                        </div>
                        <div className="mt-1">
                          {sla.responseTime <= sla.target.responseTime ? (
                            <CheckCircle className="w-4 h-4 text-green-500 mx-auto" />
                          ) : (
                            <AlertTriangle className="w-4 h-4 text-red-500 mx-auto" />
                          )}
                        </div>
                      </div>
                      
                      <div className="text-center">
                        <div className={cn(
                          "text-2xl font-bold",
                          sla.errorRate <= sla.target.errorRate ? "text-green-500" : "text-red-500"
                        )}>
                          {sla.errorRate.toFixed(1)}%
                        </div>
                        <div className="text-sm text-muted-foreground">
                          Error Rate (Target: ≤{sla.target.errorRate}%)
                        </div>
                        <div className="mt-1">
                          {sla.errorRate <= sla.target.errorRate ? (
                            <CheckCircle className="w-4 h-4 text-green-500 mx-auto" />
                          ) : (
                            <AlertTriangle className="w-4 h-4 text-red-500 mx-auto" />
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Capacity Planning */}
        <TabsContent value="capacity" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <PieChart className="w-5 h-5" />
                Capacity Planning & Recommendations
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Analyze capacity trends and get recommendations for scaling
              </p>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <h4 className="font-semibold mb-4">Capacity Utilization Trends</h4>
                  <div className="space-y-3">
                    {systemResources.map((resource) => (
                      <div key={resource.component}>
                        <div className="flex justify-between mb-1">
                          <span className="text-sm">{resource.component}</span>
                          <span className="text-sm font-mono">
                            {Math.round((resource.usage / resource.capacity) * 100)}%
                          </span>
                        </div>
                        <div className="w-full bg-muted rounded-full h-2">
                          <div 
                            className={cn(
                              "h-2 rounded-full transition-all",
                              resource.status === 'critical' ? "bg-red-500" :
                              resource.status === 'high' ? "bg-orange-500" :
                              resource.status === 'moderate' ? "bg-yellow-500" : "bg-green-500"
                            )}
                            style={{ width: `${(resource.usage / resource.capacity) * 100}%` }}
                          />
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
                
                <div>
                  <h4 className="font-semibold mb-4">Scaling Recommendations</h4>
                  <div className="space-y-3">
                    {systemResources
                      .filter(resource => resource.recommendations.length > 0)
                      .map((resource) => (
                        <div key={resource.component} className="p-3 bg-muted/50 rounded-lg">
                          <h5 className="font-medium mb-1">{resource.component}</h5>
                          <ul className="text-sm space-y-1">
                            {resource.recommendations.map((rec, index) => (
                              <li key={index} className="flex items-start gap-2">
                                <span className="text-orange-500">•</span>
                                {rec}
                              </li>
                            ))}
                          </ul>
                        </div>
                      ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}