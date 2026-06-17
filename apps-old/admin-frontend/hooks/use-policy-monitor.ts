import { useCallback, useEffect, useState } from 'react';

import { getPolicyStatsAction } from '@/app/policies/actions';
import { useToast } from '@/hooks/use-toast';

export interface PolicyEvaluation {
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

export interface PolicyStats {
  total_policies: number;
  active_policies: number;
  policies_by_type: Record<string, number>;
  evaluations_24h: number;
  avg_evaluation_time_ms: number;
  decision_breakdown: Record<string, number>;
}

export interface PolicyPerformance {
  policy_id: string;
  policy_name: string;
  policy_type: string;
  evaluations_count: number;
  avg_evaluation_time_ms: number;
  decision_breakdown: Record<string, number>;
  success_rate: number;
}

interface UsePolicyMonitorContext {
  isLiveMode: boolean;
  setIsLiveMode: (value: boolean) => void;
}

export function usePolicyMonitor(ctx: UsePolicyMonitorContext) {
  const [liveEvaluations, setLiveEvaluations] = useState<PolicyEvaluation[]>([]);
  const [stats, setStats] = useState<PolicyStats | null>(null);
  const [_performance, _setPerformance] = useState<PolicyPerformance[]>([]);
  const [loading, setLoading] = useState(true);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const { toast } = useToast();

  const loadPolicyStats = async () => {
    try {
      const policyStats = await getPolicyStatsAction();
      if (policyStats) {
        setStats(policyStats);
      }
    } catch (_error) {
      // Silently fail
    }
  };

  const loadRecentEvaluations = () => {
    try {
      const mockEvaluations: PolicyEvaluation[] = [
        {
          id: '1',
          user_id: 'user1',
          user_email: 'alice@epsx.io',
          action_attempted: 'epsx:analytics:execute',
          decision: 'allow',
          decision_reason: 'Business Hours Policy: Within business hours',
          evaluation_time_ms: 11,
          evaluated_at: new Date(Date.now() - 15000).toISOString(),
          policy_name: 'Business Hours',
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
          action_attempted: 'epsx:analytics:execute',
          decision: 'deny',
          decision_reason: 'Risk Control Policy: Risk score too high',
          evaluation_time_ms: 8,
          evaluated_at: new Date(Date.now() - 48000).toISOString(),
          policy_name: 'Risk Control',
        },
      ];

      setLiveEvaluations(mockEvaluations);
    } catch (_error) {
      // Silently fail
    }
  };

  const loadInitialData = useCallback(async () => {
    try {
      setLoading(true);
      await loadPolicyStats();
      loadRecentEvaluations();
    } catch (_error) {
      toast({
        title: "Error",
        description: "Failed to load monitoring data",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  }, [toast]);

  const refreshLiveData = useCallback(() => {
    try {
      if (Math.random() > 0.7) {
        const newEvaluation: PolicyEvaluation = {
          id: Date.now().toString(),
          user_id: `user${Math.floor(Math.random() * 10)}`,
          user_email: `user${Math.floor(Math.random() * 10)}@epsx.io`,
          action_attempted: (['epsx:analytics:execute', 'epsx:analytics:view', 'epsx:portfolio:export'][Math.floor(Math.random() * 3)] ?? 'epsx:analytics:view'),
          decision: (['allow', 'deny', 'require_mfa', 'require_approval'][Math.floor(Math.random() * 4)] as 'allow' | 'deny' | 'require_mfa' | 'require_approval' | undefined) ?? 'allow',
          decision_reason: 'Policy evaluation completed',
          evaluation_time_ms: Math.floor(Math.random() * 50) + 5,
          evaluated_at: new Date().toISOString(),
          policy_name: (['Business Hours', 'Risk Control', 'Data Export'][Math.floor(Math.random() * 3)] ?? 'Business Hours'),
        };

        setLiveEvaluations(prev => [newEvaluation, ...prev.slice(0, 19)]);
      }

      setLastUpdate(new Date());
    } catch (_error) {
      // Silently fail
    }
  }, []);

  useEffect(() => {
    void loadInitialData();

    let interval: NodeJS.Timeout | undefined;
    if (ctx.isLiveMode === true) {
      interval = setInterval(() => {
        refreshLiveData();
      }, 2000);
    }

    return () => {
      if (interval !== undefined) { clearInterval(interval); }
    };
  }, [ctx.isLiveMode, loadInitialData, refreshLiveData]);

  return {
    liveEvaluations,
    stats,
    loading,
    lastUpdate,
    loadInitialData,
  };
}
