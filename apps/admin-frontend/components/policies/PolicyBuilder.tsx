'use client';

import React, { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/hooks/use-toast';
import { 
  ShieldIcon, 
  PlusIcon, 
  TrashIcon, 
  SettingsIcon,
  ClockIcon,
  MapPinIcon,
  SmartphoneIcon,
  NetworkIcon,
  BrainIcon,
  FileTextIcon,
  PlayIcon,
  SaveIcon,
  XIcon,
  AlertTriangleIcon,
  CheckCircleIcon,
} from 'lucide-react';

type PolicyType = 'time_based' | 'location_based' | 'risk_based' | 'device_based' | 'behavioral' | 'compliance' | 'custom';
type ConditionOperator = 'AND' | 'OR' | 'NOT';
type ComparisonOperator = 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'greater_than_or_equal' | 'less_than_or_equal' | 'contains' | 'not_contains' | 'starts_with' | 'ends_with' | 'between' | 'not_between' | 'in' | 'not_in' | 'regex';
type PolicyDecision = 'allow' | 'deny' | 'require_mfa' | 'require_approval' | 'restricted_access';
type SecondaryAction = 'log_audit' | 'email_risk_team' | 'send_notification' | 'increment_risk_score' | 'require_approval' | 'trigger_alert';

interface SingleCondition {
  field: string;
  operator: ComparisonOperator;
  value: any;
  negate?: boolean;
}

interface PolicyCondition {
  operator: ConditionOperator;
  conditions: SingleCondition[];
}

interface PolicyAction {
  primary: PolicyDecision;
  message?: string;
  secondary_actions: SecondaryAction[];
  restrictions?: Record<string, any>;
}

interface PolicyFormData {
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

interface PolicyTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  template_data: any;
  usage_count: number;
}

const POLICY_TYPE_OPTIONS: { value: PolicyType; label: string; icon: React.ComponentType<any>; color: string }[] = [
  { value: 'time_based', label: 'Time-Based', icon: ClockIcon, color: 'blue' },
  { value: 'location_based', label: 'Location-Based', icon: MapPinIcon, color: 'green' },
  { value: 'risk_based', label: 'Risk-Based', icon: AlertTriangleIcon, color: 'red' },
  { value: 'device_based', label: 'Device-Based', icon: SmartphoneIcon, color: 'purple' },
  { value: 'behavioral', label: 'Behavioral', icon: BrainIcon, color: 'orange' },
  { value: 'compliance', label: 'Compliance', icon: ShieldIcon, color: 'indigo' },
  { value: 'custom', label: 'Custom', icon: SettingsIcon, color: 'gray' },
];

const CONDITION_FIELDS = [
  { value: 'user.id', label: 'User ID', type: 'string' },
  { value: 'user.email', label: 'User Email', type: 'string' },
  { value: 'user.tier', label: 'User Tier', type: 'string' },
  { value: 'user.account_age_days', label: 'Account Age (Days)', type: 'number' },
  { value: 'user.risk_score', label: 'User Risk Score', type: 'number' },
  { value: 'environment.time.hour', label: 'Time of Day (Hour)', type: 'number' },
  { value: 'environment.time.day_of_week', label: 'Day of Week', type: 'number' },
  { value: 'environment.is_business_hours', label: 'Business Hours', type: 'boolean' },
  { value: 'environment.is_weekend', label: 'Weekend', type: 'boolean' },
  { value: 'device.trust_score', label: 'Device Trust Score', type: 'number' },
  { value: 'device.is_managed', label: 'Managed Device', type: 'boolean' },
  { value: 'device.type', label: 'Device Type', type: 'string' },
  { value: 'network.type', label: 'Network Type', type: 'string' },
  { value: 'network.is_trusted', label: 'Trusted Network', type: 'boolean' },
  { value: 'location.country', label: 'Country', type: 'string' },
  { value: 'location.is_vpn', label: 'VPN Connection', type: 'boolean' },
  { value: 'action', label: 'Action', type: 'string' },
];

const COMPARISON_OPERATORS: { value: ComparisonOperator; label: string; types: string[] }[] = [
  { value: 'equals', label: 'Equals', types: ['string', 'number', 'boolean'] },
  { value: 'not_equals', label: 'Not Equals', types: ['string', 'number', 'boolean'] },
  { value: 'greater_than', label: 'Greater Than', types: ['number'] },
  { value: 'less_than', label: 'Less Than', types: ['number'] },
  { value: 'greater_than_or_equal', label: 'Greater Than or Equal', types: ['number'] },
  { value: 'less_than_or_equal', label: 'Less Than or Equal', types: ['number'] },
  { value: 'contains', label: 'Contains', types: ['string'] },
  { value: 'not_contains', label: 'Not Contains', types: ['string'] },
  { value: 'starts_with', label: 'Starts With', types: ['string'] },
  { value: 'ends_with', label: 'Ends With', types: ['string'] },
  { value: 'between', label: 'Between', types: ['number'] },
  { value: 'not_between', label: 'Not Between', types: ['number'] },
  { value: 'in', label: 'In List', types: ['string', 'number'] },
  { value: 'not_in', label: 'Not In List', types: ['string', 'number'] },
];

export default function PolicyBuilder() {
  const [formData, setFormData] = useState<PolicyFormData>({
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
  });
  
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
      const response = await fetch('/api/v1/admin/policies/templates');
      if (response.ok) {
        const data = await response.json();
        setTemplates(data.templates || []);
      }
    } catch (error) {
      console.error('Error loading templates:', error);
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
        action: formData.target_actions[0] || 'epsx:test:action',
        simulate_context: {
          time_of_day: 14,
          day_of_week: 2,
          device_trust_score: 75,
          location_country: 'US',
          network_type: 'office',
          risk_score: 25,
        }
      };

      const response = await fetch('/api/v1/admin/policies/evaluate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(testContext),
      });

      if (response.ok) {
        const data = await response.json();
        setTestResults(data.evaluation);
        toast({
          title: "Test Complete",
          description: `Policy decision: ${data.evaluation.decision}`,
        });
      }
    } catch (error) {
      console.error('Error testing policy:', error);
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
      
      if (!formData.name || formData.target_actions.length === 0 || formData.conditions.conditions.length === 0) {
        toast({
          title: "Validation Error",
          description: "Please fill in all required fields",
          variant: "destructive",
        });
        return;
      }

      const response = await fetch('/api/v1/admin/policies', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(formData),
      });

      if (response.ok) {
        const data = await response.json();
        toast({
          title: "Success",
          description: `Policy "${formData.name}" created successfully`,
        });
        
        // Reset form
        setFormData({
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
        });
      } else {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create policy');
      }
    } catch (error) {
      console.error('Error saving policy:', error);
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to save policy",
        variant: "destructive",
      });
    } finally {
      setSaving(false);
    }
  };

  const selectedPolicyType = POLICY_TYPE_OPTIONS.find(opt => opt.value === formData.policy_type);
  const IconComponent = selectedPolicyType?.icon || SettingsIcon;

  return (
    <div className="space-y-6 sm:space-y-8">
      {/* Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="h-12 w-12 bg-gradient-to-br from-purple-400 to-blue-500 rounded-2xl flex items-center justify-center">
            <ShieldIcon className="h-6 w-6 text-white" />
          </div>
          <div>
            <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600 bg-clip-text text-transparent">Dynamic Policy Builder</h2>
            <p className="text-sm text-gray-600 dark:text-gray-400">Create conditional access policies</p>
          </div>
        </div>
        
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
          <Button 
            variant="outline" 
            size="sm"
            onClick={() => setShowTemplates(!showTemplates)}
            className="min-h-[44px] rounded-2xl border-2"
          >
            <FileTextIcon className="h-4 w-4 mr-2" />
            Templates
          </Button>
          
          <Button 
            size="sm"
            onClick={handleTestPolicy}
            disabled={formData.conditions.conditions.length === 0}
            className="min-h-[44px] rounded-2xl bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600"
          >
            <PlayIcon className="h-4 w-4 mr-2" />
            Test Policy
          </Button>
          
          <Button 
            onClick={handleSavePolicy}
            disabled={saving || !formData.name}
            className="min-h-[44px] rounded-2xl bg-gradient-to-r from-purple-500 to-blue-500 hover:from-purple-600 hover:to-blue-600"
          >
            <SaveIcon className="h-4 w-4 mr-2" />
            {saving ? 'Saving...' : 'Save Policy'}
          </Button>
        </div>
      </div>

      {/* Templates Panel */}
      {showTemplates && (
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-indigo-400/20 to-purple-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-blue-600 via-indigo-600 to-purple-600 bg-clip-text text-transparent">Policy Templates</h3>
              <Button variant="ghost" size="sm" onClick={() => setShowTemplates(false)} className="h-10 w-10 rounded-2xl">
                <XIcon className="h-4 w-4" />
              </Button>
            </div>
            
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              {templates.map(template => (
                <div key={template.id} className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-4 shadow-xl border border-white/20 cursor-pointer hover:scale-[1.02]">
                  <div className="flex items-start justify-between mb-2">
                    <h4 className="font-semibold text-gray-900 dark:text-white">{template.name}</h4>
                    <Badge variant="outline" className="text-xs">{template.category}</Badge>
                  </div>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">{template.description}</p>
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-gray-500 dark:text-gray-400">Used {template.usage_count} times</span>
                    <Button size="sm" variant="outline" className="rounded-xl min-h-[36px]">Use Template</Button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Policy Configuration */}
      <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-indigo-400/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
          <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600 bg-clip-text text-transparent mb-4 sm:mb-6">Policy Configuration</h3>
          
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
            <div>
              <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Policy Name *</label>
              <Input
                placeholder="e.g., Trading Hours Control"
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                className="rounded-2xl border-2 min-h-[44px]"
              />
            </div>
            
            <div>
              <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Policy Type *</label>
              <select
                className="w-full px-3 py-3 border-2 border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white min-h-[44px]"
                value={formData.policy_type}
                onChange={(e) => setFormData(prev => ({ ...prev, policy_type: e.target.value as PolicyType }))}
              >
                {POLICY_TYPE_OPTIONS.map(option => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Priority</label>
              <Input
                type="number"
                placeholder="100"
                value={formData.priority || ''}
                onChange={(e) => setFormData(prev => ({ ...prev, priority: parseInt(e.target.value) || 100 }))}
                className="rounded-2xl border-2 min-h-[44px]"
              />
            </div>
          </div>
          
          <div>
            <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Description</label>
            <textarea
              className="w-full px-3 py-3 border-2 border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              rows={3}
              placeholder="Describe what this policy does..."
              value={formData.description || ''}
              onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
            />
          </div>
        </div>
      </div>

      {/* Target Actions */}
      <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-teal-400/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-4 gap-3">
            <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-green-600 via-emerald-600 to-teal-600 bg-clip-text text-transparent">Target Actions</h3>
            <Button size="sm" onClick={addTargetAction} className="min-h-[44px] rounded-2xl bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600 w-full sm:w-auto">
              <PlusIcon className="h-4 w-4 mr-2" />
              Add Action
            </Button>
          </div>
          
          <div className="space-y-3">
            {formData.target_actions.map((action, index) => (
              <div key={index} className="flex items-center gap-3 p-3 sm:p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl">
                <span className="font-mono text-sm flex-1 text-gray-900 dark:text-gray-100">{action}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => removeTargetAction(index)}
                  className="h-10 w-10 p-0 text-red-600 hover:bg-red-100 dark:hover:bg-red-900/20 rounded-xl"
                >
                  <TrashIcon className="h-4 w-4" />
                </Button>
              </div>
            ))}
            
            {formData.target_actions.length === 0 && (
              <div className="text-center py-8 sm:py-12 text-gray-500 dark:text-gray-400">
                <div className="h-16 w-16 bg-gradient-to-br from-gray-200 to-gray-300 dark:from-gray-700 dark:to-gray-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                  <ShieldIcon className="h-8 w-8 text-gray-400" />
                </div>
                <p className="text-sm font-medium">No target actions defined</p>
                <p className="text-xs">Add actions this policy should apply to</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Conditions Builder */}
      <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-orange-400/20 via-yellow-400/20 to-amber-400/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-4 gap-3">
            <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-orange-600 via-yellow-600 to-amber-600 bg-clip-text text-transparent">Policy Conditions</h3>
            <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
              <select
                className="px-3 py-2 border-2 border-gray-300 dark:border-gray-600 rounded-2xl text-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-white min-h-[44px]"
                value={formData.conditions.operator}
                onChange={(e) => setFormData(prev => ({
                  ...prev,
                  conditions: {
                    ...prev.conditions,
                    operator: e.target.value as ConditionOperator
                  }
                }))}
              >
                <option value="AND">ALL conditions must be met (AND)</option>
                <option value="OR">ANY condition must be met (OR)</option>
                <option value="NOT">NO conditions must be met (NOT)</option>
              </select>
              
              <Button size="sm" onClick={addCondition} className="min-h-[44px] rounded-2xl bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 w-full sm:w-auto">
                <PlusIcon className="h-4 w-4 mr-2" />
                Add Condition
              </Button>
            </div>
          </div>
          
          <div className="space-y-4">
            {formData.conditions.conditions.map((condition, index) => {
              const field = CONDITION_FIELDS.find(f => f.value === condition.field);
              const availableOperators = COMPARISON_OPERATORS.filter(op => 
                field ? op.types.includes(field.type) : true
              );
              
              return (
                <div key={index} className="p-4 border-2 border-gray-200 dark:border-gray-700 rounded-2xl bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-800 dark:to-gray-700">
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
                    <div>
                      <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Field</label>
                      <select
                        className="w-full px-3 py-3 border-2 border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white min-h-[44px]"
                        value={condition.field}
                        onChange={(e) => updateCondition(index, { field: e.target.value })}
                      >
                        {CONDITION_FIELDS.map(field => (
                          <option key={field.value} value={field.value}>
                            {field.label}
                          </option>
                        ))}
                      </select>
                    </div>
                    
                    <div>
                      <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Operator</label>
                      <select
                        className="w-full px-3 py-3 border-2 border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white min-h-[44px]"
                        value={condition.operator}
                        onChange={(e) => updateCondition(index, { operator: e.target.value as ComparisonOperator })}
                      >
                        {availableOperators.map(op => (
                          <option key={op.value} value={op.value}>
                            {op.label}
                          </option>
                        ))}
                      </select>
                    </div>
                    
                    <div>
                      <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Value</label>
                      <Input
                        placeholder="Enter value..."
                        value={condition.value}
                        onChange={(e) => updateCondition(index, { value: e.target.value })}
                        className="rounded-2xl border-2 min-h-[44px]"
                      />
                    </div>
                    
                    <div className="flex flex-col justify-between gap-2">
                      <div>
                        <label className="flex items-center gap-3 p-2 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-700">
                          <input
                            type="checkbox"
                            checked={condition.negate || false}
                            onChange={(e) => updateCondition(index, { negate: e.target.checked })}
                            className="h-5 w-5 rounded border-gray-300 text-orange-600 focus:ring-orange-500"
                          />
                          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Negate</span>
                        </label>
                      </div>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => removeCondition(index)}
                        className="h-10 w-10 p-0 text-red-600 hover:bg-red-100 dark:hover:bg-red-900/20 rounded-xl self-end"
                      >
                        <TrashIcon className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </div>
              );
            })}
            
            {formData.conditions.conditions.length === 0 && (
              <div className="text-center py-8 sm:py-12 text-gray-500 dark:text-gray-400">
                <div className="h-16 w-16 bg-gradient-to-br from-orange-200 to-yellow-200 dark:from-orange-800 dark:to-yellow-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                  <AlertTriangleIcon className="h-8 w-8 text-orange-600 dark:text-orange-400" />
                </div>
                <p className="text-sm font-medium">No conditions defined</p>
                <p className="text-xs">Add conditions to control when this policy applies</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Actions & Responses */}
      <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-red-400/20 via-pink-400/20 to-rose-400/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
          <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-red-600 via-pink-600 to-rose-600 bg-clip-text text-transparent mb-4 sm:mb-6">Actions & Responses</h3>
          
          <div className="space-y-4 sm:space-y-6">
            <div>
              <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Primary Action</label>
              <select
                className="w-full px-3 py-3 border-2 border-gray-300 dark:border-gray-600 rounded-2xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white min-h-[44px]"
                value={formData.actions.primary}
                onChange={(e) => setFormData(prev => ({
                  ...prev,
                  actions: {
                    ...prev.actions,
                    primary: e.target.value as PolicyDecision
                  }
                }))}
              >
                <option value="allow">Allow</option>
                <option value="deny">Deny</option>
                <option value="require_mfa">Require MFA</option>
                <option value="require_approval">Require Approval</option>
                <option value="restricted_access">Restricted Access</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Message</label>
              <Input
                placeholder="e.g., High-value trade requires approval"
                value={formData.actions.message || ''}
                onChange={(e) => setFormData(prev => ({
                  ...prev,
                  actions: {
                    ...prev.actions,
                    message: e.target.value
                  }
                }))}
                className="rounded-2xl border-2 min-h-[44px]"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Test Results */}
      {testResults && (
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-teal-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
            <div className="flex items-center gap-3 mb-4">
              <div className="h-10 w-10 bg-gradient-to-br from-green-500 to-emerald-500 rounded-2xl flex items-center justify-center">
                <CheckCircleIcon className="h-5 w-5 text-white" />
              </div>
              <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-green-600 via-emerald-600 to-teal-600 bg-clip-text text-transparent">Test Results</h3>
            </div>
            
            <div className="space-y-4">
              <div className="flex flex-col sm:flex-row sm:items-center gap-2">
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">Decision:</span>
                <Badge 
                  variant={testResults.decision === 'allow' ? 'default' : 
                          testResults.decision === 'deny' ? 'destructive' : 'secondary'}
                  className="text-sm px-3 py-1"
                >
                  {testResults.decision}
                </Badge>
              </div>
              
              <div>
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">Reason:</span>
                <p className="text-sm text-gray-600 dark:text-gray-400 mt-2 p-3 bg-gray-50 dark:bg-gray-800 rounded-xl">{testResults.final_decision_reason}</p>
              </div>
              
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">Evaluation Time:</span>
                <span className="text-sm bg-blue-100 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 px-2 py-1 rounded-lg">{testResults.evaluation_time_ms}ms</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}