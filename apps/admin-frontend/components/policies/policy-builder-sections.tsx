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

export const POLICY_TYPE_OPTIONS: { value: PolicyType; label: string; icon: React.ComponentType<{ className?: string }>; color: string }[] = [
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
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" />
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 p-5">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-gradient-to-br from-[#7645d9]/10 to-[#1fc7d4]/10 rounded-[14px] text-[#7645d9] border border-[#7645d9]/20">
            <ShieldIcon className="w-4 h-4" />
          </div>
          <div>
            <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-[#7645d9] to-[#1fc7d4] bg-clip-text text-transparent">Dynamic Policy Builder</h2>
            <p className="text-sm text-muted-foreground">Create conditional access policies</p>
          </div>
        </div>

        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
          <Button
            variant="glass"
            size="sm"
            onClick={() => setShowTemplates(!showTemplates)}
            className="min-h-[44px] rounded-xl border border-border/40"
          >
            <FileTextIcon className="h-4 w-4 mr-2" />
            Templates
          </Button>

          <Button
            size="sm"
            onClick={onTest}
            disabled={formData.conditions.conditions.length === 0}
            className="min-h-[44px] rounded-xl bg-gradient-to-r from-[#ed4b9e] to-[#7645d9] text-white border-0"
          >
            <PlayIcon className="h-4 w-4 mr-2" />
            Test Policy
          </Button>

          <Button
            onClick={onSave}
            disabled={saving || formData.name === ''}
            className="rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white border-0"
          >
            <SaveIcon className="h-4 w-4 mr-2" />
            {saving ? 'Saving...' : 'Save Policy'}
          </Button>
        </div>
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
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
      <div className="flex items-center justify-between p-5 border-b border-border/20">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[14px] text-[#ffb237] border border-[#ffb237]/20">
            <FileTextIcon className="w-4 h-4" />
          </div>
          <h3 className="text-xs font-bold text-[#ffb237] uppercase tracking-[0.2em]">Policy Templates</h3>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose} className="h-10 w-10 rounded-xl">
          <XIcon className="h-4 w-4" />
        </Button>
      </div>

      <div className="p-5">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {templates.map(template => (
            <div key={template.id} className="bg-muted/30 rounded-xl p-4 border border-border/40 cursor-pointer hover:scale-[1.02] hover:border-[#ffb237]/30 transition-all">
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
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
      <div className="flex items-center gap-3 p-5 border-b border-border/20">
        <div className="p-2 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-[14px] text-[#1fc7d4] border border-[#1fc7d4]/20">
          <SettingsIcon className="w-4 h-4" />
        </div>
        <h3 className="text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]">Policy Configuration</h3>
      </div>

      <div className="p-5 space-y-4">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          <div>
            <label className="block text-sm font-semibold mb-2 text-foreground">Policy Name *</label>
            <Input
              placeholder="e.g., Business Hours Control"
              value={formData.name}
              onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
              className="rounded-xl border border-border/40 min-h-[44px]"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold mb-2 text-foreground">Policy Type *</label>
            <select
              className="w-full px-3 py-3 border border-border/40 rounded-xl bg-muted/30 text-foreground min-h-[44px]"
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
            <label className="block text-sm font-semibold mb-2 text-foreground">Priority</label>
            <Input
              type="number"
              placeholder="100"
              value={formData.priority ?? ''}
              onChange={(e) => setFormData(prev => ({ ...prev, priority: parseInt(e.target.value, 10) || 100 }))}
              className="rounded-xl border border-border/40 min-h-[44px]"
            />
          </div>
        </div>

        <div>
          <label className="block text-sm font-semibold mb-2 text-foreground">Description</label>
          <textarea
            className="w-full px-3 py-3 border border-border/40 rounded-xl bg-muted/30 text-foreground"
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
  onAdd: (value: string) => void;
  onRemove: (index: number) => void;
}

export function TargetActions({ actions, onAdd, onRemove }: TargetActionsProps) {
  const [newAction, setNewAction] = React.useState('');

  const handleAdd = () => {
    if (newAction.trim() !== '' && !actions.includes(newAction.trim())) {
      onAdd(newAction.trim());
      setNewAction('');
    }
  };

  return (
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]" />
      <div className="flex items-center justify-between p-5 border-b border-border/20">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 rounded-[14px] text-[#31d0aa] border border-[#31d0aa]/20">
            <ShieldIcon className="w-4 h-4" />
          </div>
          <h3 className="text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em]">Target Actions</h3>
        </div>
      </div>

      <div className="p-5 space-y-3">
        <div className="flex gap-2">
          <Input
            variant="glass"
            placeholder="e.g., epsx:trading:execute"
            value={newAction}
            onChange={(e) => setNewAction(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); handleAdd(); } }}
            className="rounded-xl min-h-[44px] flex-1"
          />
          <Button
            size="sm"
            onClick={handleAdd}
            className="min-h-[44px] rounded-xl bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4] text-white border-0"
          >
            <PlusIcon className="h-4 w-4 mr-2" />
            Add
          </Button>
        </div>

        {actions.map((action, index) => (
          <div key={action} className="flex items-center gap-3 p-3 sm:p-4 bg-muted/30 border border-border/40 rounded-xl">
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
            <div className="h-16 w-16 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 border border-[#31d0aa]/20 rounded-2xl flex items-center justify-center mx-auto mb-4">
              <ShieldIcon className="h-8 w-8 text-[#31d0aa]" />
            </div>
            <p className="text-sm font-medium">No target actions defined</p>
            <p className="text-xs">Add actions this policy should apply to</p>
          </div>
        )}
      </div>
    </div>
  );
}

interface ConditionRowProps {
  condition: SingleCondition;
  index: number;
  onRemove: (index: number) => void;
  onUpdate: (index: number, updates: Partial<SingleCondition>) => void;
}

function ConditionRow({ condition, index, onRemove, onUpdate }: ConditionRowProps) {
  const fieldDef = CONDITION_FIELDS.find(f => f.value === condition.field);
  const availableOperators = COMPARISON_OPERATORS.filter(op => fieldDef !== undefined ? op.types.includes(fieldDef.type) : true);
  return (
    <div className="p-4 border border-border/40 rounded-xl bg-muted/30">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
        <div>
          <label className="block text-sm font-semibold mb-2 text-foreground">Field</label>
          <select className="w-full px-3 py-3 border border-border/40 rounded-xl bg-card text-foreground min-h-[44px]" value={condition.field} onChange={(e) => onUpdate(index, { field: e.target.value })}>
            {CONDITION_FIELDS.map(f => <option key={f.value} value={f.value}>{f.label}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-sm font-semibold mb-2 text-foreground">Operator</label>
          <select className="w-full px-3 py-3 border border-border/40 rounded-xl bg-card text-foreground min-h-[44px]" value={condition.operator} onChange={(e) => onUpdate(index, { operator: e.target.value as ComparisonOperator })}>
            {availableOperators.map(op => <option key={op.value} value={op.value}>{op.label}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-sm font-semibold mb-2 text-foreground">Value</label>
          <Input variant="glass" placeholder="Enter value..." value={String(condition.value)} onChange={(e) => onUpdate(index, { value: e.target.value })} className="rounded-xl min-h-[44px]" />
        </div>
        <div className="flex flex-col justify-between gap-2">
          <label className="flex items-center gap-3 p-2 rounded-xl hover:bg-muted/50">
            <input type="checkbox" checked={condition.negate ?? false} onChange={(e) => onUpdate(index, { negate: e.target.checked })} className="h-5 w-5 rounded border-border/40 text-[#7645d9] focus:ring-[#7645d9]/50" />
            <span className="text-sm font-medium text-foreground">Negate</span>
          </label>
          <Button size="sm" variant="ghost" onClick={() => onRemove(index)} className="h-10 w-10 p-0 text-red-400 hover:bg-red-500/10 rounded-xl self-end"><TrashIcon className="h-4 w-4" /></Button>
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
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 p-5 border-b border-border/20">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[14px] text-[#ffb237] border border-[#ffb237]/20">
            <AlertTriangleIcon className="w-4 h-4" />
          </div>
          <h3 className="text-xs font-bold text-[#ffb237] uppercase tracking-[0.2em]">Policy Conditions</h3>
        </div>
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
          <select
            className="px-3 py-2 border border-border/40 rounded-xl text-sm bg-muted/30 text-foreground min-h-[44px]"
            value={formData.conditions.operator}
            onChange={(e) => setFormData(prev => ({ ...prev, conditions: { ...prev.conditions, operator: e.target.value as ConditionOperator } }))}
          >
            <option value="AND">ALL conditions must be met (AND)</option>
            <option value="OR">ANY condition must be met (OR)</option>
            <option value="NOT">NO conditions must be met (NOT)</option>
          </select>
          <Button size="sm" onClick={onAdd} className="min-h-[44px] rounded-xl bg-gradient-to-r from-[#ffb237] to-[#ed4b9e] text-white border-0 w-full sm:w-auto">
            <PlusIcon className="h-4 w-4 mr-2" />Add Condition
          </Button>
        </div>
      </div>

      <div className="p-5 space-y-4">
        {formData.conditions.conditions.map((condition, index) => (
          <ConditionRow key={condition._key} condition={condition} index={index} onRemove={onRemove} onUpdate={onUpdate} />
        ))}

        {formData.conditions.conditions.length === 0 && (
          <div className="text-center py-8 sm:py-12 text-muted-foreground">
            <div className="h-16 w-16 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 border border-[#ffb237]/20 rounded-2xl flex items-center justify-center mx-auto mb-4">
              <AlertTriangleIcon className="h-8 w-8 text-[#ffb237]" />
            </div>
            <p className="text-sm font-medium">No conditions defined</p>
            <p className="text-xs">Add conditions to control when this policy applies</p>
          </div>
        )}
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
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" />
      <div className="flex items-center gap-3 p-5 border-b border-border/20">
        <div className="p-2 bg-gradient-to-br from-[#7645d9]/10 to-[#ed4b9e]/10 rounded-[14px] text-[#7645d9] border border-[#7645d9]/20">
          <ShieldIcon className="w-4 h-4" />
        </div>
        <h3 className="text-xs font-bold text-[#7645d9] uppercase tracking-[0.2em]">Actions & Responses</h3>
      </div>

      <div className="p-5 space-y-4 sm:space-y-6">
        <div>
          <label className="block text-sm font-semibold mb-2 text-foreground">Primary Action</label>
          <select
            className="w-full px-3 py-3 border border-border/40 rounded-xl bg-muted/30 text-foreground min-h-[44px]"
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
            className="rounded-xl min-h-[44px]"
          />
        </div>
      </div>
    </div>
  );
}

interface TestResultsProps {
  testResults: Record<string, unknown> | null;
}

export function TestResults({ testResults }: TestResultsProps) {
  if (testResults === null) {return null;}

  return (
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]" />
      <div className="flex items-center gap-3 p-5 border-b border-border/20">
        <div className="p-2 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 rounded-[14px] text-[#31d0aa] border border-[#31d0aa]/20">
          <CheckCircleIcon className="w-4 h-4" />
        </div>
        <h3 className="text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em]">Test Results</h3>
      </div>

      <div className="p-5 space-y-4">
        <div className="flex flex-col sm:flex-row sm:items-center gap-2">
          <span className="text-sm font-semibold text-foreground">Decision:</span>
          <Badge
            variant={testResults.decision === 'allow' ? 'pancake' :
              testResults.decision === 'deny' ? 'destructive' : 'glass'}
            className="text-sm px-3 py-1"
          >
            {String(testResults.decision ?? '')}
          </Badge>
        </div>

        <div>
          <span className="text-sm font-semibold text-foreground">Reason:</span>
          <p className="text-sm text-muted-foreground mt-2 p-3 bg-muted/30 border border-border/40 rounded-xl">{String(testResults.final_decision_reason ?? '')}</p>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold text-foreground">Evaluation Time:</span>
          <span className="text-sm bg-[#31d0aa]/10 text-[#31d0aa] border border-[#31d0aa]/20 px-2 py-1 rounded-lg">{String(testResults.evaluation_time_ms ?? '')}ms</span>
        </div>
      </div>
    </div>
  );
}
