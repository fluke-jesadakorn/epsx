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
  _key: string;
  field: string;
  operator: ComparisonOperator;
  value: string | number | boolean;
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
  restrictions?: Record<string, unknown>;
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
  template_data: Record<string, unknown>;
  usage_count: number;
}

const INITIAL_FORM_DATA: PolicyFormData = {
  name: '',
  description: '',
  policy_type: 'time_based',
  target_actions: [],
  conditions: { operator: 'AND', conditions: [] },
  actions: { primary: 'require_approval', secondary_actions: [] },
  priority: 100,
};

const TEST_CONTEXT = {
  user_id: 'test-user',
  user_email: 'test@epsx.io',
  action: 'epsx:test:action',
  simulate_context: { time_of_day: 14, day_of_week: 2, device_trust_score: 75, location_country: 'US', network_type: 'office', risk_score: 25 },
};

interface PolicyActions {
  formData: PolicyFormData;
  setFormData: React.Dispatch<React.SetStateAction<PolicyFormData>>;
  setTestResults: (r: Record<string, unknown> | null) => void;
  setSaving: (v: boolean) => void;
  toast: ReturnType<typeof useToast>['toast'];
}

async function runTestPolicy({ formData, setTestResults, toast }: Pick<PolicyActions, 'formData' | 'setTestResults' | 'toast'>) {
  try {
    const ctx = { ...TEST_CONTEXT, action: formData.target_actions[0] ?? TEST_CONTEXT.action };
    const result = await evaluatePolicyAction(ctx);
    if (result !== null && result !== undefined) {
      const r = result as Record<string, unknown>;
      setTestResults(r);
      toast({ title: "Test Complete", description: `Policy decision: ${String(r.decision ?? '')}` });
    }
  } catch (_error) {
    toast({ title: "Test Failed", description: "Failed to test policy evaluation", variant: "destructive" });
  }
}

async function savePolicy({ formData, setFormData, setSaving, toast }: Omit<PolicyActions, 'setTestResults'>) {
  try {
    setSaving(true);
    if (formData.name === '' || formData.target_actions.length === 0 || formData.conditions.conditions.length === 0) {
      toast({ title: "Validation Error", description: "Please fill in all required fields", variant: "destructive" });
      return;
    }
    await createPolicyAction(formData as unknown as Record<string, unknown>);
    toast({ title: "Success", description: `Policy "${formData.name}" created successfully` });
    setFormData(INITIAL_FORM_DATA);
  } catch (_error) {
    toast({ title: "Error", description: _error instanceof Error ? _error.message : "Failed to save policy", variant: "destructive" });
  } finally {
    setSaving(false);
  }
}

export function usePolicyBuilder() {
  const [formData, setFormData] = useState<PolicyFormData>(INITIAL_FORM_DATA);
  const [templates, setTemplates] = useState<PolicyTemplate[]>([]);
  const [showTemplates, setShowTemplates] = useState(false);
  const [testResults, setTestResults] = useState<Record<string, unknown> | null>(null);
  const [saving, setSaving] = useState(false);
  const { toast } = useToast();

  useEffect(() => {
    void (async () => {
      try {
        const loaded = await getPolicyTemplatesAction();
        setTemplates(loaded as PolicyTemplate[]);
      } catch (_e) { /* silently ignore */ }
    })();
  }, []);

  const addCondition = () => {
    setFormData(prev => ({
      ...prev,
      conditions: { ...prev.conditions, conditions: [...prev.conditions.conditions, { _key: crypto.randomUUID(), field: 'user.tier', operator: 'equals', value: '' }] }
    }));
  };

  const removeCondition = (index: number) => {
    setFormData(prev => ({ ...prev, conditions: { ...prev.conditions, conditions: prev.conditions.conditions.filter((_, i) => i !== index) } }));
  };

  const updateCondition = (index: number, updates: Partial<SingleCondition>) => {
    setFormData(prev => ({ ...prev, conditions: { ...prev.conditions, conditions: prev.conditions.conditions.map((c, i) => i === index ? { ...c, ...updates } : c) } }));
  };

  const addTargetAction = (value: string) => {
    if (value !== '' && !formData.target_actions.includes(value)) {
      setFormData(prev => ({ ...prev, target_actions: [...prev.target_actions, value] }));
    }
  };

  const removeTargetAction = (index: number) => {
    setFormData(prev => ({ ...prev, target_actions: prev.target_actions.filter((_, i) => i !== index) }));
  };

  const handleTestPolicy = async () => runTestPolicy({ formData, setTestResults, toast });
  const handleSavePolicy = async () => savePolicy({ formData, setFormData, setSaving, toast });

  return { formData, setFormData, templates, showTemplates, setShowTemplates, testResults, saving, addCondition, removeCondition, updateCondition, addTargetAction, removeTargetAction, handleTestPolicy, handleSavePolicy };
}
