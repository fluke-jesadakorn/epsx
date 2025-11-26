'use client';

import { 
  ActivityIcon, 
  ShieldIcon, 
  PlayIcon,
  PauseIcon,
  RefreshCwIcon,
  FilterIcon,
  DownloadIcon,
  AlertTriangleIcon,
  CheckCircleIcon,
  ClockIcon,
  UserIcon,
  BarChart3Icon,
  TrendingUpIcon,
  TrendingDownIcon,
  ZapIcon,
} from 'lucide-react';
import React, { useState, useEffect } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { useToast } from '@/hooks/use-toast';

interface PolicyEvaluation {
  id: string;
  user_id: string;
  user_email: string;
  action_attempted: string;
  decision: 'allow' | 'deny' | 'require_mfa' | 'require_approval' | 'restricted_access';
  decision_reason: string;
  evaluation_time_ms: number;
  evaluated_at: string;
  policy_id?: string;
  policy_name?: string;
}

interface PolicyStats {
  total_policies: number;
  active_policies: number;
  policies_by_type: Record<string, number>;
  evaluations_24h: number;
  avg_evaluation_time_ms: number;
  decision_breakdown: Record<string, number>;
}

interface PolicyPerformance {
  policy_id: string;
  policy_name: string;
  policy_type: string;
  evaluations_count: number;
  avg_evaluation_time_ms: number;
  decision_breakdown: Record<string, number>;
  success_rate: number;
}

const DECISION_COLORS = {
  allow: 'green',
  deny: 'red', 
  require_mfa: 'yellow',
  require_approval: 'orange',
  restricted_access: 'purple',
};

const DECISION_ICONS = {
  allow: CheckCircleIcon,
  deny: AlertTriangleIcon,
  require_mfa: ShieldIcon,
  require_approval: ClockIcon,
  restricted_access: FilterIcon,
};

/**
 *
 */
export default function PolicyMonitor() {
  const [liveEvaluations, setLiveEvaluations] = useState<PolicyEvaluation[]>([]);
  const [stats, setStats] = useState<PolicyStats | null>(null);
  const [performance, setPerformance] = useState<PolicyPerformance[]>([]);
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [loading, setLoading] = useState(true);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const { toast } = useToast();

  useEffect(() => {
    loadInitialData();
    
    let interval: NodeJS.Timeout;
    if (isLiveMode) {
      interval = setInterval(refreshLiveData, 2000); // Update every 2 seconds
    }
    
    return () => {
      if (interval) {clearInterval(interval);}
    };
  }, [isLiveMode]);

  const loadInitialData = async () => {
    try {
      setLoading(true);
      await Promise.all([
        loadPolicyStats(),
        loadRecentEvaluations(),
      ]);
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error loading initial data:', _error);
      toast({
        title: "Error",
        description: "Failed to load monitoring data",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const loadPolicyStats = async () => {
    try {
      const response = await fetch('/api/admin/policies/stats');
      if (response.ok) {
        const data = await response.json();
        setStats(data.stats);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error loading policy stats:', _error);
    }
  };

  const loadRecentEvaluations = async () => {
    try {
      // Simulate recent evaluations (in real implementation, this would come from the backend)
      const mockEvaluations: PolicyEvaluation[] = [
        {
          id: '1',
          user_id: 'user1',
          user_email: 'alice@epsx.io',
          action_attempted: 'epsx:trading:execute',
          decision: 'allow',
          decision_reason: 'Trading Hours Policy: Within business hours',
          evaluation_time_ms: 11,
          evaluated_at: new Date(Date.now() - 15000).toISOString(),
          policy_name: 'Trading Hours',
        },
        {
          id: '2',
          user_id: 'user2',
          user_email: 'bob@epsx.io',
          action_attempted: 'epsx:analytics:export',
          decision: 'require_mfa',
          decision_reason: 'Data Export Policy: Large dataset requires MFA',
          evaluation_time_ms: 23,
          evaluated_at: new Date(Date.now() - 32000).toISOString(),
          policy_name: 'Data Export Control',
        },
        {
          id: '3',
          user_id: 'user3',
          user_email: 'carol@epsx.io',
          action_attempted: 'epsx:trading:execute',
          decision: 'deny',
          decision_reason: 'Risk Control Policy: Risk score too high',
          evaluation_time_ms: 8,
          evaluated_at: new Date(Date.now() - 48000).toISOString(),
          policy_name: 'Risk Control',
        },
      ];
      
      setLiveEvaluations(mockEvaluations);
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error loading recent evaluations:', _error);
    }
  };

  const refreshLiveData = async () => {
    try {
      // Simulate new evaluations
      if (Math.random() > 0.7) { // 30% chance of new evaluation
        const newEvaluation: PolicyEvaluation = {
          id: Date.now().toString(),
          user_id: `user${Math.floor(Math.random() * 10)}`,
          user_email: `user${Math.floor(Math.random() * 10)}@epsx.io`,
          action_attempted: ['epsx:trading:execute', 'epsx:analytics:view', 'epsx:portfolio:export'][Math.floor(Math.random() * 3)],
          decision: ['allow', 'deny', 'require_mfa', 'require_approval'][Math.floor(Math.random() * 4)] as any,
          decision_reason: 'Policy evaluation completed',
          evaluation_time_ms: Math.floor(Math.random() * 50) + 5,
          evaluated_at: new Date().toISOString(),
          policy_name: ['Trading Hours', 'Risk Control', 'Data Export'][Math.floor(Math.random() * 3)],
        };
        
        setLiveEvaluations(prev => [newEvaluation, ...prev.slice(0, 19)]); // Keep last 20
      }
      
      setLastUpdate(new Date());
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error refreshing live data:', _error);
    }
  };

  const formatTime = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleTimeString('en-US', { 
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  const getDecisionBadge = (decision: string) => {
    const color = DECISION_COLORS[decision as keyof typeof DECISION_COLORS] || 'gray';
    const Icon = DECISION_ICONS[decision as keyof typeof DECISION_ICONS] || ActivityIcon;
    
    let variant: "default" | "destructive" | "outline" | "secondary" = "outline";
    if (decision === 'allow') {variant = 'default';}
    if (decision === 'deny') {variant = 'destructive';}
    if (decision === 'require_mfa') {variant = 'secondary';}
    
    return (
      <Badge variant={variant} className="flex items-center gap-1 text-xs">
        <Icon className="h-3 w-3" />
        {decision.replace('_', ' ').toUpperCase()}
      </Badge>
    );
  };

  const calculatePercentage = (value: number, total: number) => {
    return total > 0 ? Math.round((value / total) * 100) : 0;
  };

  if (loading) {
    return (
      <Card className="p-4 sm:p-6">
        <div className="flex items-center justify-center h-32 sm:h-48">
          <RefreshCwIcon className="h-6 w-6 text-gray-400" />
          <span className="ml-2 text-gray-600">Loading monitoring data...</span>
        </div>
      </Card>
    );
  }

  return (
    <div className="space-y-4 sm:space-y-6">
      {/* Header */}
      <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-4">
        <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
          <div className="flex items-center gap-2 sm:gap-3">
            <ActivityIcon className="h-5 w-5 sm:h-6 sm:w-6 text-blue-600" />
            <h2 className="text-lg sm:text-xl font-semibold">Policy Monitoring Dashboard</h2>
          </div>
          <Badge variant={isLiveMode ? "default" : "outline"} className="flex items-center gap-1 w-fit">
            {isLiveMode ? <span className="w-2 h-2 bg-green-400 rounded-full" /> : null}
            {isLiveMode ? 'LIVE' : 'PAUSED'}
          </Badge>
        </div>
        
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full lg:w-auto">
          <span className="text-sm text-gray-500 text-center sm:text-left">
            Last update: {lastUpdate.toLocaleTimeString()}
          </span>
          
          <div className="flex flex-col sm:flex-row gap-2 sm:gap-3">
            <Button 
              variant="outline" 
              size="sm"
              onClick={() => setIsLiveMode(!isLiveMode)}
              className="min-h-[44px] justify-center"
            >
              {isLiveMode ? (
                <>
                  <PauseIcon className="h-4 w-4 mr-2" />
                  Pause
                </>
              ) : (
                <>
                  <PlayIcon className="h-4 w-4 mr-2" />
                  Resume
                </>
              )}
            </Button>
            
            <Button 
              variant="outline" 
              size="sm"
              onClick={loadInitialData}
              className="min-h-[44px] justify-center"
            >
              <RefreshCwIcon className="h-4 w-4 mr-2" />
              Refresh
            </Button>
            
            <Button 
              size="sm"
              disabled
              className="min-h-[44px] justify-center"
            >
              <DownloadIcon className="h-4 w-4 mr-2" />
              Export Log
            </Button>
          </div>
        </div>
      </div>

      {/* Stats Overview */}
      {stats && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-3 sm:gap-4">
          <Card className="p-3 sm:p-4">
            <div className="flex items-center gap-2">
              <ShieldIcon className="h-4 w-4 text-blue-600 flex-shrink-0" />
              <div>
                <p className="text-xs sm:text-sm text-gray-600">Active Policies</p>
                <p className="text-lg sm:text-xl font-semibold">{stats.active_policies}</p>
              </div>
            </div>
          </Card>
          
          <Card className="p-3 sm:p-4">
            <div className="flex items-center gap-2">
              <BarChart3Icon className="h-4 w-4 text-green-600 flex-shrink-0" />
              <div>
                <p className="text-xs sm:text-sm text-gray-600">Evaluations (24h)</p>
                <p className="text-lg sm:text-xl font-semibold">{stats.evaluations_24h}</p>
              </div>
            </div>
          </Card>
          
          <Card className="p-3 sm:p-4">
            <div className="flex items-center gap-2">
              <ZapIcon className="h-4 w-4 text-orange-600 flex-shrink-0" />
              <div>
                <p className="text-xs sm:text-sm text-gray-600">Avg Response</p>
                <p className="text-lg sm:text-xl font-semibold">{Math.round(stats.avg_evaluation_time_ms)}ms</p>
              </div>
            </div>
          </Card>
          
          <Card className="p-3 sm:p-4">
            <div className="flex items-center gap-2">
              <CheckCircleIcon className="h-4 w-4 text-green-600 flex-shrink-0" />
              <div>
                <p className="text-xs sm:text-sm text-gray-600">Allowed</p>
                <p className="text-lg sm:text-xl font-semibold">
                  {calculatePercentage(stats.decision_breakdown.allow || 0, stats.evaluations_24h)}%
                </p>
              </div>
            </div>
          </Card>
          
          <Card className="p-3 sm:p-4">
            <div className="flex items-center gap-2">
              <AlertTriangleIcon className="h-4 w-4 text-red-600 flex-shrink-0" />
              <div>
                <p className="text-xs sm:text-sm text-gray-600">Blocked</p>
                <p className="text-lg sm:text-xl font-semibold">
                  {calculatePercentage(stats.decision_breakdown.deny || 0, stats.evaluations_24h)}%
                </p>
              </div>
            </div>
          </Card>
          
          <Card className="p-3 sm:p-4">
            <div className="flex items-center gap-2">
              <ClockIcon className="h-4 w-4 text-yellow-600 flex-shrink-0" />
              <div>
                <p className="text-xs sm:text-sm text-gray-600">Pending</p>
                <p className="text-lg sm:text-xl font-semibold">
                  {calculatePercentage(
                    (stats.decision_breakdown.require_approval || 0) + (stats.decision_breakdown.require_mfa || 0), 
                    stats.evaluations_24h
                  )}%
                </p>
              </div>
            </div>
          </Card>
        </div>
      )}

      {/* Live Evaluations */}
      <Card className="p-4 sm:p-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-4 gap-3">
          <div className="flex flex-col sm:flex-row sm:items-center gap-2">
            <div className="flex items-center gap-2">
              <ActivityIcon className="h-5 w-5 text-blue-600" />
              <h3 className="text-base sm:text-lg font-medium">Live Policy Evaluations</h3>
            </div>
            {isLiveMode && (
              <span className="text-sm text-gray-500">(refreshing every 2s)</span>
            )}
          </div>
          
          <div className="flex items-center gap-3 w-full sm:w-auto">
            <Button variant="outline" size="sm" disabled className="w-full sm:w-auto min-h-[44px]">
              <FilterIcon className="h-4 w-4 mr-2" />
              Filter
            </Button>
          </div>
        </div>
        
        <div className="space-y-2 max-h-96 overflow-y-auto">
          {/* Desktop Header */}
          <div className="hidden lg:grid grid-cols-12 gap-2 px-3 py-2 text-xs font-medium text-gray-600 border-b">
            <div className="col-span-1">Time</div>
            <div className="col-span-2">User</div>
            <div className="col-span-3">Action</div>
            <div className="col-span-2">Policy</div>
            <div className="col-span-2">Decision</div>
            <div className="col-span-2">Details</div>
          </div>
          
          {liveEvaluations.map((evaluation, index) => (
            <div key={evaluation.id}>
              {/* Desktop Layout */}
              <div 
                className={`hidden lg:grid grid-cols-12 gap-2 px-3 py-2 text-sm hover:bg-gray-50 rounded ${
                  index === 0 && isLiveMode ? 'bg-blue-50 border-l-4 border-blue-500' : ''
                }`}
              >
                <div className="col-span-1 font-mono text-xs">
                  {formatTime(evaluation.evaluated_at)}
                </div>
                
                <div className="col-span-2">
                  <div className="flex items-center gap-1">
                    <UserIcon className="h-3 w-3 text-gray-400" />
                    <span className="truncate">{evaluation.user_email}</span>
                  </div>
                </div>
                
                <div className="col-span-3">
                  <span className="font-mono text-xs bg-gray-100 px-2 py-1 rounded">
                    {evaluation.action_attempted}
                  </span>
                </div>
                
                <div className="col-span-2">
                  <span className="text-xs text-gray-600">
                    {evaluation.policy_name || 'Unknown'}
                  </span>
                </div>
                
                <div className="col-span-2">
                  {getDecisionBadge(evaluation.decision)}
                </div>
                
                <div className="col-span-2">
                  <div className="flex items-center gap-2 text-xs text-gray-500">
                    <ClockIcon className="h-3 w-3" />
                    <span>{evaluation.evaluation_time_ms}ms</span>
                  </div>
                </div>
              </div>
              
              {/* Mobile Layout */}
              <div className={`lg:hidden p-3 rounded border bg-white hover:bg-gray-50 ${
                  index === 0 && isLiveMode ? 'bg-blue-50 border-blue-200' : 'border-gray-200'
                }`}>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <UserIcon className="h-4 w-4 text-gray-400" />
                    <span className="font-medium text-sm truncate">{evaluation.user_email}</span>
                  </div>
                  <span className="font-mono text-xs text-gray-500">
                    {formatTime(evaluation.evaluated_at)}
                  </span>
                </div>
                
                <div className="mb-2">
                  <span className="font-mono text-xs bg-gray-100 px-2 py-1 rounded break-all">
                    {evaluation.action_attempted}
                  </span>
                </div>
                
                <div className="flex items-center justify-between">
                  <div className="flex flex-col gap-1">
                    <span className="text-xs text-gray-600">
                      {evaluation.policy_name || 'Unknown'}
                    </span>
                    <div className="flex items-center gap-2 text-xs text-gray-500">
                      <ClockIcon className="h-3 w-3" />
                      <span>{evaluation.evaluation_time_ms}ms</span>
                    </div>
                  </div>
                  <div>
                    {getDecisionBadge(evaluation.decision)}
                  </div>
                </div>
              </div>
            </div>
          ))}
          
          {liveEvaluations.length === 0 && (
            <div className="text-center py-8 sm:py-12 text-gray-500">
              <ActivityIcon className="h-8 w-8 sm:h-12 sm:w-12 mx-auto mb-4 text-gray-300" />
              <p className="text-base sm:text-lg font-medium">No recent evaluations</p>
              <p className="text-sm">Policy evaluations will appear here in real-time</p>
            </div>
          )}
        </div>
      </Card>

      {/* Decision Breakdown Chart */}
      {stats && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 sm:gap-6">
          <Card className="p-4 sm:p-6">
            <h3 className="text-base sm:text-lg font-medium mb-4">Decision Breakdown (24h)</h3>
            
            <div className="space-y-3">
              {Object.entries(stats.decision_breakdown).map(([decision, count]) => {
                const percentage = calculatePercentage(count, stats.evaluations_24h);
                const Icon = DECISION_ICONS[decision as keyof typeof DECISION_ICONS] || ActivityIcon;
                
                return (
                  <div key={decision} className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 sm:gap-0">
                    <div className="flex items-center gap-2">
                      <Icon className="h-4 w-4 text-gray-600" />
                      <span className="text-sm capitalize">{decision.replace('_', ' ')}</span>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <div className="w-20 sm:w-24 bg-gray-200 rounded-full h-2">
                        <div 
                          className="bg-blue-600 h-2 rounded-full"
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                      <span className="text-sm font-medium w-16 sm:w-12 text-right">
                        {count} ({percentage}%)
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </Card>

          <Card className="p-4 sm:p-6">
            <h3 className="text-base sm:text-lg font-medium mb-4">Performance Trends</h3>
            
            <div className="space-y-3 sm:space-y-4">
              <div className="flex items-center justify-between p-3 bg-green-50 rounded-lg">
                <div className="flex items-center gap-2">
                  <TrendingUpIcon className="h-4 w-4 text-green-600" />
                  <span className="text-sm font-medium">Policy Efficiency</span>
                </div>
                <div className="text-right">
                  <div className="text-base sm:text-lg font-semibold text-green-700">+12%</div>
                  <div className="text-xs text-green-600">vs last week</div>
                </div>
              </div>
              
              <div className="flex items-center justify-between p-3 bg-blue-50 rounded-lg">
                <div className="flex items-center gap-2">
                  <ZapIcon className="h-4 w-4 text-blue-600" />
                  <span className="text-sm font-medium">Response Time</span>
                </div>
                <div className="text-right">
                  <div className="text-base sm:text-lg font-semibold text-blue-700">{Math.round(stats.avg_evaluation_time_ms)}ms</div>
                  <div className="text-xs text-blue-600">avg evaluation</div>
                </div>
              </div>
              
              <div className="flex items-center justify-between p-3 bg-orange-50 rounded-lg">
                <div className="flex items-center gap-2">
                  <TrendingDownIcon className="h-4 w-4 text-orange-600" />
                  <span className="text-sm font-medium">False Positives</span>
                </div>
                <div className="text-right">
                  <div className="text-base sm:text-lg font-semibold text-orange-700">-8%</div>
                  <div className="text-xs text-orange-600">vs last week</div>
                </div>
              </div>
            </div>
          </Card>
        </div>
      )}

      {/* Alert Center */}
      <Card className="p-4 sm:p-6 bg-yellow-50 border-yellow-200">
        <div className="flex items-center gap-2 mb-4">
          <AlertTriangleIcon className="h-5 w-5 text-yellow-600" />
          <h3 className="text-base sm:text-lg font-medium">Alert & Notification Center</h3>
        </div>
        
        <div className="space-y-3">
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
              <div className="flex items-start sm:items-center gap-2">
                <div className="w-2 h-2 bg-red-500 rounded-full mt-1 sm:mt-0 flex-shrink-0" />
                <span className="font-medium text-red-900 flex-shrink-0">HIGH</span>
                <span className="text-sm text-red-700">Behavioral Anomaly Policy showing 89% false positives</span>
              </div>
              <span className="text-xs text-red-600 self-end sm:self-auto">2m ago</span>
            </div>
          </div>
          
          <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
              <div className="flex items-start sm:items-center gap-2">
                <div className="w-2 h-2 bg-yellow-500 rounded-full mt-1 sm:mt-0 flex-shrink-0" />
                <span className="font-medium text-yellow-900 flex-shrink-0">MED</span>
                <span className="text-sm text-yellow-700">Geographic Policy blocked 15 legitimate EU users</span>
              </div>
              <span className="text-xs text-yellow-600 self-end sm:self-auto">5m ago</span>
            </div>
          </div>
          
          <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
              <div className="flex items-start sm:items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full mt-1 sm:mt-0 flex-shrink-0" />
                <span className="font-medium text-green-900 flex-shrink-0">LOW</span>
                <span className="text-sm text-green-700">Trading Hours Policy evaluation time increased to 45ms</span>
              </div>
              <span className="text-xs text-green-600 self-end sm:self-auto">12m ago</span>
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}