/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any */
'use client';

import {
  AlertTriangleIcon,
  BrainIcon,
  CheckCircleIcon,
  ClockIcon,
  FileTextIcon,
  MapPinIcon,
  PlusIcon,
  SaveIcon,
  SettingsIcon,
  ShieldIcon,
  SmartphoneIcon,
  TrashIcon,
  XIcon,
  PlayIcon
} from 'lucide-react';
import React from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

import type { PolicyFormData, PolicyTemplate, SingleCondition } from './hooks/use-policy-builder';

type PolicyType = 'time_based' | 'location_based' | 'risk_based' | 'device_based' | 'behavioral' | 'compliance' | 'custom';
type ComparisonOperator = 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'greater_than_or_equal' | 'less_than_or_equal' | 'contains' | 'not_contains' | 'starts_with' | 'ends_with' | 'between' | 'not_between' | 'in' | 'not_in' | 'regex';
type ConditionOperator = 'AND' | 'OR' | 'NOT';
type PolicyDecision = 'allow' | 'deny' | 'require_mfa' | 'require_approval' | 'restricted_access';

export const POLICY_TYPE_OPTIONS: { value: PolicyType; label: string; icon: React.ComponentType<any>; color: string }[] = [
  { value: 'time_based', label: 'Time-Based', icon: ClockIcon, color: 'blue' },
  { value: 'location_based', label: 'Location-Based', icon: MapPinIcon, color: 'green' },
  { value: 'risk_based', label: 'Risk-Based', icon: AlertTriangleIcon, color: 'red' },
  { value: 'device_based', label: 'Device-Based', icon: SmartphoneIcon, color: 'purple' },
  { value: 'behavioral', label: 'Behavioral', icon: BrainIcon, color: 'orange' },
  { value: 'compliance', label: 'Compliance', icon: ShieldIcon, color: 'indigo' },
  { value: 'custom', label: 'Custom', icon: SettingsIcon, color: 'gray' },
];

export const CONDITION_FIELDS = [
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

export const COMPARISON_OPERATORS: { value: ComparisonOperator; label: string; types: string[] }[] = [
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

interface HeaderProps {
  showTemplates: boolean;
  setShowTemplates: (value: boolean) => void;
  onTest: () => void;
  onSave: () => void;
  saving: boolean;
  formData: PolicyFormData;
}

export function PolicyBuilderHeader({ showTemplates, setShowTemplates, onTest, onSave, saving, formData }: HeaderProps) {
  return (
    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
      <div className="flex items-center gap-3">
        <div className="h-12 w-12 bg-gradient-to-br from-purple-400 to-blue-500 rounded-2xl flex items-center justify-center">
          <ShieldIcon className="h-6 w-6 text-white" />
        </div>
        <div>
          <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">Dynamic Policy Builder</h2>
          <p className="text-sm text-muted-foreground">Create conditional access policies</p>
        </div>
      </div>

      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
        <Button
          variant="glass"
          size="sm"
          onClick={() => setShowTemplates(!showTemplates)}
          className="min-h-[44px] rounded-2xl border-2"
        >
          <FileTextIcon className="h-4 w-4 mr-2" />
          Templates
        </Button>

        <Button
          size="sm"
          variant="admin"
          onClick={onTest}
          disabled={formData.conditions.conditions.length === 0}
        >
          <PlayIcon className="h-4 w-4 mr-2" />
          Test Policy
        </Button>

        <Button
          onClick={onSave}
          disabled={saving ?? !formData.name}
          variant="admin"
        >
          <SaveIcon className="h-4 w-4 mr-2" />
          {saving ? 'Saving...' : 'Save Policy'}
        </Button>
      </div>
    </div>
  );
}

interface TemplatesPanelProps {
  templates: PolicyTemplate[];
  onClose: () => void;
}

export function TemplatesPanel({ templates, onClose }: TemplatesPanelProps) {
  return (
    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-500/20 via-orange-500/20 to-purple-500/20 p-0.5">
      <div className="relative bg-white dark:bg-card backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent">Policy Templates</h3>
          <Button variant="ghost" size="sm" onClick={onClose} className="h-10 w-10 rounded-2xl">
            <XIcon className="h-4 w-4" />
          </Button>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {templates.map(template => (
            <div key={template.id} className="bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm rounded-2xl p-4 shadow-xl border border-gray-200 dark:border-border cursor-pointer hover:scale-[1.02] hover:border-purple-500/30 transition-all">
              <div className="flex items-start justify-between mb-2">
                <h4 className="font-semibold text-foreground">{template.name}</h4>
                <Badge variant="glass" className="text-xs">{template.category}</Badge>
              </div>
              <p className="text-sm text-muted-foreground mb-3">{template.description}</p>
              <div className="flex items-center justify-between">
                <span className="text-xs text-muted-foreground">Used {template.usage_count} times</span>
                <Button size="sm" variant="glass" className="rounded-xl min-h-[36px]">Use Template</Button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

interface ConfigurationSectionProps {
  formData: PolicyFormData;
  setFormData: React.Dispatch<React.SetStateAction<PolicyFormData>>;
}

export function PolicyConfiguration({ formData, setFormData }: ConfigurationSectionProps) {
  return (
    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-500/20 via-orange-500/20 to-purple-500/20 p-0.5">
      <div className="relative bg-white dark:bg-card backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
        <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent mb-4 sm:mb-6">Policy Configuration</h3>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
          <div>
            <label className="block text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">Policy Name *</label>
            <Input
              placeholder="e.g., Business Hours Control"
              value={formData.name}
              onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
              className="rounded-2xl border-2 min-h-[44px]"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold mb-2 text-foreground">Policy Type *</label>
            <select
              className="w-full px-3 py-3 border-2 border-gray-300 dark:border-white/20 rounded-2xl bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm text-foreground min-h-[44px]"
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
              value={formData.priority ?? ''}
              onChange={(e) => setFormData(prev => ({ ...prev, priority: parseInt(e.target.value) ?? 100 }))}
              className="rounded-2xl border-2 min-h-[44px]"
            />
          </div>
        </div>

        <div>
          <label className="block text-sm font-semibold mb-2 text-foreground">Description</label>
          <textarea
            className="w-full px-3 py-3 border-2 border-gray-300 dark:border-white/20 rounded-2xl bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm text-foreground"
            rows={3}
            placeholder="Describe what this policy does..."
            value={formData.description ?? ''}
            onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
          />
        </div>
      </div>
    </div>
  );
}

interface TargetActionsProps {
  actions: string[];
  onAdd: () => void;
  onRemove: (index: number) => void;
}

export function TargetActions({ actions, onAdd, onRemove }: TargetActionsProps) {
  return (
    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-500/20 via-orange-500/20 to-purple-500/20 p-0.5">
      <div className="relative bg-white dark:bg-card backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-4 gap-3">
          <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent">Target Actions</h3>
          <Button size="sm" onClick={onAdd} className="min-h-[44px] rounded-2xl bg-gradient-to-r from-purple-500 to-orange-500 hover:shadow-xl hover:shadow-purple-500/30 w-full sm:w-auto">
            <PlusIcon className="h-4 w-4 mr-2" />
            Add Action
          </Button>
        </div>

        <div className="space-y-3">
          {actions.map((action, index) => (
            <div key={index} className="flex items-center gap-3 p-3 sm:p-4 bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm border border-gray-200 dark:border-border rounded-2xl">
              <span className="font-mono text-sm flex-1 text-foreground">{action}</span>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => onRemove(index)}
                className="h-10 w-10 p-0 text-red-400 hover:bg-red-500/10 rounded-xl"
              >
                <TrashIcon className="h-4 w-4" />
              </Button>
            </div>
          ))}

          {actions.length === 0 && (
            <div className="text-center py-8 sm:py-12 text-muted-foreground">
              <div className="h-16 w-16 bg-gradient-to-br from-purple-500/10 to-orange-500/10 border border-purple-500/20 rounded-2xl flex items-center justify-center mx-auto mb-4">
                <ShieldIcon className="h-8 w-8 text-purple-400" />
              </div>
              <p className="text-sm font-medium">No target actions defined</p>
              <p className="text-xs">Add actions this policy should apply to</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface ConditionsBuilderProps {
  formData: PolicyFormData;
  setFormData: React.Dispatch<React.SetStateAction<PolicyFormData>>;
  onAdd: () => void;
  onRemove: (index: number) => void;
  onUpdate: (index: number, updates: Partial<SingleCondition>) => void;
}

export function ConditionsBuilder({ formData, setFormData, onAdd, onRemove, onUpdate }: ConditionsBuilderProps) {
  return (
    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-500/20 via-orange-500/20 to-purple-500/20 p-0.5">
      <div className="relative bg-white dark:bg-card backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-4 gap-3">
          <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent">Policy Conditions</h3>
          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
            <select
              className="px-3 py-2 border-2 border-gray-300 dark:border-white/20 rounded-2xl text-sm bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm text-foreground min-h-[44px]"
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

            <Button size="sm" onClick={onAdd} className="min-h-[44px] rounded-2xl bg-gradient-to-r from-purple-500 to-orange-500 hover:shadow-xl hover:shadow-purple-500/30 w-full sm:w-auto">
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
              <div key={index} className="p-4 border-2 border-gray-200 dark:border-border rounded-2xl bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm">
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
                  <div>
                    <label className="block text-sm font-semibold mb-2 text-foreground">Field</label>
                    <select
                      className="w-full px-3 py-3 border-2 border-gray-300 dark:border-white/20 rounded-2xl bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm text-foreground min-h-[44px]"
                      value={condition.field}
                      onChange={(e) => onUpdate(index, { field: e.target.value })}
                    >
                      {CONDITION_FIELDS.map(field => (
                        <option key={field.value} value={field.value}>
                          {field.label}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-semibold mb-2 text-foreground">Operator</label>
                    <select
                      className="w-full px-3 py-3 border-2 border-gray-300 dark:border-white/20 rounded-2xl bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm text-foreground min-h-[44px]"
                      value={condition.operator}
                      onChange={(e) => onUpdate(index, { operator: e.target.value as ComparisonOperator })}
                    >
                      {availableOperators.map(op => (
                        <option key={op.value} value={op.value}>
                          {op.label}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-semibold mb-2 text-foreground">Value</label>
                    <Input
                      variant="glass"
                      placeholder="Enter value..."
                      value={condition.value}
                      onChange={(e) => onUpdate(index, { value: e.target.value })}
                      className="rounded-2xl min-h-[44px]"
                    />
                  </div>

                  <div className="flex flex-col justify-between gap-2">
                    <div>
                      <label className="flex items-center gap-3 p-2 rounded-xl hover:bg-gray-100 dark:hover:bg-white/5">
                        <input
                          type="checkbox"
                          checked={condition.negate ?? false}
                          onChange={(e) => onUpdate(index, { negate: e.target.checked })}
                          className="h-5 w-5 rounded border-gray-300 dark:border-white/20 text-purple-500 focus:ring-purple-500/50"
                        />
                        <span className="text-sm font-medium text-foreground">Negate</span>
                      </label>
                    </div>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => onRemove(index)}
                      className="h-10 w-10 p-0 text-red-400 hover:bg-red-500/10 rounded-xl self-end"
                    >
                      <TrashIcon className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </div>
            );
          })}

          {formData.conditions.conditions.length === 0 && (
            <div className="text-center py-8 sm:py-12 text-muted-foreground">
              <div className="h-16 w-16 bg-gradient-to-br from-purple-500/10 to-orange-500/10 border border-purple-500/20 rounded-2xl flex items-center justify-center mx-auto mb-4">
                <AlertTriangleIcon className="h-8 w-8 text-purple-400" />
              </div>
              <p className="text-sm font-medium">No conditions defined</p>
              <p className="text-xs">Add conditions to control when this policy applies</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface ActionsResponsesProps {
  formData: PolicyFormData;
  setFormData: React.Dispatch<React.SetStateAction<PolicyFormData>>;
}

export function ActionsResponses({ formData, setFormData }: ActionsResponsesProps) {
  return (
    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-500/20 via-orange-500/20 to-purple-500/20 p-0.5">
      <div className="relative bg-white dark:bg-card backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
        <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent mb-4 sm:mb-6">Actions & Responses</h3>

        <div className="space-y-4 sm:space-y-6">
          <div>
            <label className="block text-sm font-semibold mb-2 text-foreground">Primary Action</label>
            <select
              className="w-full px-3 py-3 border-2 border-gray-300 dark:border-white/20 rounded-2xl bg-white dark:bg-white/[0.04] dark:bg-slate-800/50 backdrop-blur-sm text-foreground min-h-[44px]"
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
            <label className="block text-sm font-semibold mb-2 text-foreground">Message</label>
            <Input
              variant="glass"
              placeholder="e.g., High-value trade requires approval"
              value={formData.actions.message ?? ''}
              onChange={(e) => setFormData(prev => ({
                ...prev,
                actions: {
                  ...prev.actions,
                  message: e.target.value
                }
              }))}
              className="rounded-2xl min-h-[44px]"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

interface TestResultsProps {
  testResults: any;
}

export function TestResults({ testResults }: TestResultsProps) {
  if (!testResults) {return null;}

  return (
    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-500/20 via-teal-500/20 to-emerald-500/20 p-0.5">
      <div className="relative bg-white dark:bg-card backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
        <div className="flex items-center gap-3 mb-4">
          <div className="h-10 w-10 bg-gradient-to-br from-emerald-500 to-teal-500 rounded-2xl flex items-center justify-center">
            <CheckCircleIcon className="h-5 w-5 text-white" />
          </div>
          <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent">Test Results</h3>
        </div>

        <div className="space-y-4">
          <div className="flex flex-col sm:flex-row sm:items-center gap-2">
            <span className="text-sm font-semibold text-foreground">Decision:</span>
            <Badge
              variant={testResults.decision === 'allow' ? 'pancake' :
                testResults.decision === 'deny' ? 'destructive' : 'glass'}
              className="text-sm px-3 py-1"
            >
              {testResults.decision}
            </Badge>
          </div>

          <div>
            <span className="text-sm font-semibold text-foreground">Reason:</span>
            <p className="text-sm text-muted-foreground mt-2 p-3 bg-white dark:bg-white/[0.04] backdrop-blur-sm border border-gray-200 dark:border-border rounded-xl">{testResults.final_decision_reason}</p>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold text-foreground">Evaluation Time:</span>
            <span className="text-sm bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-2 py-1 rounded-lg">{testResults.evaluation_time_ms}ms</span>
          </div>
        </div>
      </div>
    </div>
  );
}
