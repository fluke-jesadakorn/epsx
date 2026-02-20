/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any */
'use client';

import { useState, useEffect } from 'react';

import { createPolicyAction, evaluatePolicyAction, getPolicyTemplatesAction } from '@/app/policies/actions';
import { useToast } from '@/hooks/use-toast';

type PolicyType = 'time_based' | 'location_based' | 'risk_based' | 'device_based' | 'behavioral' | 'compliance' | 'custom';
type ConditionOperator = 'AND' | 'OR' | 'NOT';
type ComparisonOperator = 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'greater_than_or_equal' | 'less_than_or_equal' | 'contains' | 'not_contains' | 'starts_with' | 'ends_with' | 'between' | 'not_between' | 'in' | 'not_in' | 'regex';
type PolicyDecision = 'allow' | 'deny' | 'require_mfa' | 'require_approval' | 'restricted_access';
type SecondaryAction = 'log_audit' | 'email_risk_team' | 'send_notification' | 'increment_risk_score' | 'require_approval' | 'trigger_alert';

export interface SingleCondition {
  field: string;
  operator: ComparisonOperator;
  value: any;
  negate?: boolean;
}

export interface PolicyCondition {
  operator: ConditionOperator;
  conditions: SingleCondition[];
}

export interface PolicyAction {
  primary: PolicyDecision;
  message?: string;
  secondary_actions: SecondaryAction[];
  restrictions?: Record<string, any>;
}

export interface PolicyFormData {
  name: string;
  description?: string;
  policy_type: PolicyType;
  target_actions: string[];
  conditions: PolicyCondition;
  actions: PolicyAction;
  priority?: number;
  effective_from?: string;
  effective_until?: string;
}

export interface PolicyTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  template_data: any;
  usage_count: number;
}

const INITIAL_FORM_DATA: PolicyFormData = {
  name: '',
  description: '',
  policy_type: 'time_based',
  target_actions: [],
  conditions: {
    operator: 'AND',
    conditions: []
  },
  actions: {
    primary: 'require_approval',
    secondary_actions: [],
  },
  priority: 100,
};

export function usePolicyBuilder() {
  const [formData, setFormData] = useState<PolicyFormData>(INITIAL_FORM_DATA);
  const [templates, setTemplates] = useState<PolicyTemplate[]>([]);
  const [showTemplates, setShowTemplates] = useState(false);
  const [testResults, setTestResults] = useState<any>(null);
  const [saving, setSaving] = useState(false);
  const { toast } = useToast();

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      const templates = await getPolicyTemplatesAction();
      setTemplates(templates);
    } catch (_error) {
      console.error('Error loading templates:', _error);
    }
  };

  const addCondition = () => {
    setFormData(prev => ({
      ...prev,
      conditions: {
        ...prev.conditions,
        conditions: [
          ...prev.conditions.conditions,
          {
            field: 'user.tier',
            operator: 'equals',
            value: '',
          }
        ]
      }
    }));
  };

  const removeCondition = (index: number) => {
    setFormData(prev => ({
      ...prev,
      conditions: {
        ...prev.conditions,
        conditions: prev.conditions.conditions.filter((_, i) => i !== index)
      }
    }));
  };

  const updateCondition = (index: number, updates: Partial<SingleCondition>) => {
    setFormData(prev => ({
      ...prev,
      conditions: {
        ...prev.conditions,
        conditions: prev.conditions.conditions.map((condition, i) =>
          i === index ? { ...condition, ...updates } : condition
        )
      }
    }));
  };

  const addTargetAction = () => {
    const newAction = prompt('Enter target action (e.g., epsx:trading:execute):');
    if (newAction && !formData.target_actions.includes(newAction)) {
      setFormData(prev => ({
        ...prev,
        target_actions: [...prev.target_actions, newAction]
      }));
    }
  };

  const removeTargetAction = (index: number) => {
    setFormData(prev => ({
      ...prev,
      target_actions: prev.target_actions.filter((_, i) => i !== index)
    }));
  };

  const handleTestPolicy = async () => {
    try {
      const testContext = {
        user_id: 'test-user',
        user_email: 'test@epsx.io',
        action: formData.target_actions[0] ?? 'epsx:test:action',
        simulate_context: {
          time_of_day: 14,
          day_of_week: 2,
          device_trust_score: 75,
          location_country: 'US',
          network_type: 'office',
          risk_score: 25,
        }
      };

      const result = await evaluatePolicyAction(testContext);

      if (result) {
        setTestResults(result);
        toast({
          title: "Test Complete",
          description: `Policy decision: ${result.decision}`,
        });
      }
    } catch (_error) {
      console.error('Error testing policy:', _error);
      toast({
        title: "Test Failed",
        description: "Failed to test policy evaluation",
        variant: "destructive",
      });
    }
  };

  const handleSavePolicy = async () => {
    try {
      setSaving(true);

      if (!formData.name ?? formData.target_actions.length === 0 ?? formData.conditions.conditions.length === 0) {
        toast({
          title: "Validation Error",
          description: "Please fill in all required fields",
          variant: "destructive",
        });
        return;
      }

      await createPolicyAction(formData);

      toast({
        title: "Success",
        description: `Policy "${formData.name}" created successfully`,
      });

      setFormData(INITIAL_FORM_DATA);
    } catch (_error) {
      console.error('Error saving policy:', _error);
      toast({
        title: "Error",
        description: _error instanceof Error ? _error.message : "Failed to save policy",
        variant: "destructive",
      });
    } finally {
      setSaving(false);
    }
  };

  return {
    formData,
    setFormData,
    templates,
    showTemplates,
    setShowTemplates,
    testResults,
    saving,
    addCondition,
    removeCondition,
    updateCondition,
    addTargetAction,
    removeTargetAction,
    handleTestPolicy,
    handleSavePolicy,
  };
}
