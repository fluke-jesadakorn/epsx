'use client';

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '@epsx/ui';
import { Input } from '@epsx/ui';
import { Label } from '@epsx/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import { Textarea } from '@/components/ui/textarea';
import { Plus, X, Parentheses, Save, Download, Copy, Eye, Trash2 } from 'lucide-react';

interface ConditionNode {
  id: string;
  type: 'condition' | 'group';
  field?: string;
  operator?: string;
  value?: string | string[];
  logicalOperator?: 'AND' | 'OR';
  children?: ConditionNode[];
  parentId?: string;
  negated?: boolean;
}

interface ConditionTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  condition: ConditionNode;
  usageCount: number;
  tags: string[];
}

interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
  complexity: number;
}

const FIELD_TYPES = {
  'user.email': 'string',
  'user.registration_date': 'datetime',
  'user.subscription_type': 'enum',
  'user.department': 'enum',
  'user.role_level': 'enum',
  'user.account_type': 'enum',
  'user.payment_verified': 'boolean',
  'user.trial_days_remaining': 'number',
  'user.last_login': 'datetime',
  'user.tier': 'enum',
  'user.age': 'number',
  'user.country': 'string',
  'user.signup_source': 'enum',
  'user.feature_flags': 'array'
};

const FIELD_OPTIONS = Object.keys(FIELD_TYPES).map(field => ({
  value: field,
  label: field.replace('user.', '').replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
  type: FIELD_TYPES[field as keyof typeof FIELD_TYPES]
}));

const OPERATOR_OPTIONS = {
  string: [
    { value: '==', label: 'Equals', description: 'Exact match' },
    { value: '!=', label: 'Not Equals', description: 'Does not match' },
    { value: 'contains', label: 'Contains', description: 'Contains substring' },
    { value: 'starts_with', label: 'Starts With', description: 'Begins with text' },
    { value: 'ends_with', label: 'Ends With', description: 'Ends with text' },
    { value: 'regex', label: 'Regex Match', description: 'Regular expression' },
    { value: 'in', label: 'In List', description: 'Value in list' },
    { value: 'not_in', label: 'Not In List', description: 'Value not in list' }
  ],
  number: [
    { value: '==', label: 'Equals', description: 'Exact value' },
    { value: '!=', label: 'Not Equals', description: 'Different value' },
    { value: '>', label: 'Greater Than', description: 'Larger than value' },
    { value: '<', label: 'Less Than', description: 'Smaller than value' },
    { value: '>=', label: 'Greater Than or Equal', description: 'Larger or equal' },
    { value: '<=', label: 'Less Than or Equal', description: 'Smaller or equal' },
    { value: 'between', label: 'Between', description: 'Within range' },
    { value: 'in', label: 'In List', description: 'Value in list' }
  ],
  datetime: [
    { value: '==', label: 'On Date', description: 'Exact date' },
    { value: '>', label: 'After', description: 'Later than date' },
    { value: '<', label: 'Before', description: 'Earlier than date' },
    { value: '>=', label: 'On or After', description: 'Same or later' },
    { value: '<=', label: 'On or Before', description: 'Same or earlier' },
    { value: 'between', label: 'Between Dates', description: 'Within date range' },
    { value: 'days_ago', label: 'Days Ago', description: 'X days before now' },
    { value: 'hours_ago', label: 'Hours Ago', description: 'X hours before now' }
  ],
  boolean: [
    { value: '==', label: 'Is', description: 'Boolean value' },
    { value: '!=', label: 'Is Not', description: 'Opposite boolean' }
  ],
  enum: [
    { value: '==', label: 'Equals', description: 'Exact match' },
    { value: '!=', label: 'Not Equals', description: 'Does not match' },
    { value: 'in', label: 'In List', description: 'Value in list' },
    { value: 'not_in', label: 'Not In List', description: 'Value not in list' }
  ],
  array: [
    { value: 'contains', label: 'Contains', description: 'Array contains value' },
    { value: 'not_contains', label: 'Does Not Contain', description: 'Array missing value' },
    { value: 'contains_any', label: 'Contains Any', description: 'Contains any of values' },
    { value: 'contains_all', label: 'Contains All', description: 'Contains all values' },
    { value: 'empty', label: 'Is Empty', description: 'Array is empty' },
    { value: 'not_empty', label: 'Is Not Empty', description: 'Array has values' }
  ]
};

const CONDITION_TEMPLATES: ConditionTemplate[] = [
  {
    id: 'template_new_user',
    name: 'New User (24h)',
    description: 'User registered within last 24 hours',
    category: 'User Status',
    condition: {
      id: 'cond_1',
      type: 'condition',
      field: 'user.registration_date',
      operator: 'hours_ago',
      value: '24'
    },
    usageCount: 156,
    tags: ['onboarding', 'new', 'time-based']
  },
  {
    id: 'template_premium_verified',
    name: 'Verified Premium User',
    description: 'Premium subscription with verified payment',
    category: 'Subscription',
    condition: {
      id: 'group_1',
      type: 'group',
      logicalOperator: 'AND',
      children: [
        {
          id: 'cond_2',
          type: 'condition',
          field: 'user.subscription_type',
          operator: '==',
          value: 'premium'
        },
        {
          id: 'cond_3',
          type: 'condition',
          field: 'user.payment_verified',
          operator: '==',
          value: 'true',
          logicalOperator: 'AND'
        }
      ]
    },
    usageCount: 89,
    tags: ['premium', 'verified', 'payment']
  },
  {
    id: 'template_admin_it',
    name: 'IT Department Admin',
    description: 'IT department staff with manager+ role',
    category: 'Role-based',
    condition: {
      id: 'group_2',
      type: 'group',
      logicalOperator: 'AND',
      children: [
        {
          id: 'cond_4',
          type: 'condition',
          field: 'user.department',
          operator: '==',
          value: 'IT'
        },
        {
          id: 'cond_5',
          type: 'condition',
          field: 'user.role_level',
          operator: 'in',
          value: ['manager', 'director', 'vp'],
          logicalOperator: 'AND'
        }
      ]
    },
    usageCount: 34,
    tags: ['admin', 'IT', 'role']
  },
  {
    id: 'template_trial_active',
    name: 'Active Trial User',
    description: 'Trial user with remaining days',
    category: 'Trial',
    condition: {
      id: 'group_3',
      type: 'group',
      logicalOperator: 'AND',
      children: [
        {
          id: 'cond_6',
          type: 'condition',
          field: 'user.account_type',
          operator: '==',
          value: 'trial'
        },
        {
          id: 'cond_7',
          type: 'condition',
          field: 'user.trial_days_remaining',
          operator: '>',
          value: '0',
          logicalOperator: 'AND'
        }
      ]
    },
    usageCount: 78,
    tags: ['trial', 'active', 'time-limited']
  }
];

const ConditionBuilder: React.FC = () => {
  const [rootCondition, setRootCondition] = useState<ConditionNode>({
    id: 'root',
    type: 'group',
    logicalOperator: 'AND',
    children: []
  });

  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [savedConditions, setSavedConditions] = useState<ConditionNode[]>([]);
  const [activeTemplate, setActiveTemplate] = useState<string>('');

  const generateId = () => Math.random().toString(36).substr(2, 9);

  const addCondition = (parentId: string = 'root') => {
    const newCondition: ConditionNode = {
      id: generateId(),
      type: 'condition',
      field: '',
      operator: '==',
      value: '',
      parentId
    };

    updateConditionTree(parentId, (node) => {
      if (node.type === 'group') {
        const children = node.children || [];
        if (children.length > 0) {
          newCondition.logicalOperator = node.logicalOperator || 'AND';
        }
        return {
          ...node,
          children: [...children, newCondition]
        };
      }
      return node;
    });
  };

  const addGroup = (parentId: string = 'root') => {
    const newGroup: ConditionNode = {
      id: generateId(),
      type: 'group',
      logicalOperator: 'AND',
      children: [],
      parentId
    };

    updateConditionTree(parentId, (node) => {
      if (node.type === 'group') {
        const children = node.children || [];
        if (children.length > 0) {
          newGroup.logicalOperator = node.logicalOperator || 'AND';
        }
        return {
          ...node,
          children: [...children, newGroup]
        };
      }
      return node;
    });
  };

  const updateConditionTree = (targetId: string, updater: (node: ConditionNode) => ConditionNode) => {
    const updateNode = (node: ConditionNode): ConditionNode => {
      if (node.id === targetId) {
        return updater(node);
      }
      if (node.children) {
        return {
          ...node,
          children: node.children.map(updateNode)
        };
      }
      return node;
    };

    setRootCondition(updateNode);
  };

  const removeNode = (nodeId: string) => {
    const removeFromNode = (node: ConditionNode): ConditionNode => {
      if (node.children) {
        return {
          ...node,
          children: node.children
            .filter(child => child.id !== nodeId)
            .map(removeFromNode)
        };
      }
      return node;
    };

    setRootCondition(removeFromNode);
  };

  const updateCondition = (nodeId: string, updates: Partial<ConditionNode>) => {
    updateConditionTree(nodeId, (node) => ({ ...node, ...updates }));
  };

  const getFieldType = (field: string): keyof typeof OPERATOR_OPTIONS => {
    return FIELD_TYPES[field as keyof typeof FIELD_TYPES] as keyof typeof OPERATOR_OPTIONS || 'string';
  };

  const getOperatorOptions = (field: string) => {
    const fieldType = getFieldType(field);
    return OPERATOR_OPTIONS[fieldType] || OPERATOR_OPTIONS.string;
  };

  const validateCondition = (node: ConditionNode): ValidationResult => {
    const errors: string[] = [];
    const warnings: string[] = [];
    let complexity = 0;

    const validateNode = (n: ConditionNode, depth: number = 0) => {
      complexity += depth + 1;

      if (n.type === 'condition') {
        if (!n.field) errors.push(`Condition ${n.id}: Field is required`);
        if (!n.operator) errors.push(`Condition ${n.id}: Operator is required`);
        if (!n.value && n.value !== false && n.value !== 0) {
          errors.push(`Condition ${n.id}: Value is required`);
        }

        if (n.field && n.operator) {
          const fieldType = getFieldType(n.field);
          const validOperators = OPERATOR_OPTIONS[fieldType].map(op => op.value);
          if (!validOperators.includes(n.operator)) {
            errors.push(`Condition ${n.id}: Invalid operator for field type`);
          }
        }

        if (depth > 5) {
          warnings.push(`Condition ${n.id}: Very deep nesting may impact performance`);
        }
      } else if (n.type === 'group') {
        if (!n.children || n.children.length === 0) {
          warnings.push(`Group ${n.id}: Empty group`);
        } else if (n.children.length === 1) {
          warnings.push(`Group ${n.id}: Single child group is unnecessary`);
        }

        n.children?.forEach(child => validateNode(child, depth + 1));
      }
    };

    validateNode(node);

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      complexity
    };
  };

  const validateCurrentCondition = () => {
    const result = validateCondition(rootCondition);
    setValidationResult(result);
  };

  const renderConditionNode = (node: ConditionNode, depth: number = 0, index: number = 0): React.ReactNode => {
    const isSelected = selectedNode === node.id;
    const hasParent = depth > 0;

    return (
      <div
        key={node.id}
        className={`border rounded-lg p-4 space-y-3 ${
          isSelected ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800'
        } ${depth > 0 ? 'ml-6' : ''}`}
        onClick={(e) => {
          e.stopPropagation();
          setSelectedNode(node.id);
        }}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {hasParent && index > 0 && (
              <Select
                value={node.logicalOperator}
                onValueChange={(value: 'AND' | 'OR') => updateCondition(node.id, { logicalOperator: value })}
              >
                <SelectTrigger className="w-20">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="AND">AND</SelectItem>
                  <SelectItem value="OR">OR</SelectItem>
                </SelectContent>
              </Select>
            )}
            <Badge variant={node.type === 'group' ? 'default' : 'secondary'}>
              {node.type === 'group' ? 'Group' : 'Condition'}
            </Badge>
            {node.negated && <Badge variant="destructive">NOT</Badge>}
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={(e) => {
                e.stopPropagation();
                updateCondition(node.id, { negated: !node.negated });
              }}
              title="Toggle negation"
            >
              NOT
            </Button>
            {node.id !== 'root' && (
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  removeNode(node.id);
                }}
              >
                <X className="w-4 h-4" />
              </Button>
            )}
          </div>
        </div>

        {node.type === 'condition' ? (
          <div className="grid grid-cols-3 gap-3">
            <div>
              <Label>Field</Label>
              <Select
                value={node.field}
                onValueChange={(value) => updateCondition(node.id, { field: value, operator: '==', value: '' })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select field" />
                </SelectTrigger>
                <SelectContent>
                  {FIELD_OPTIONS.map(option => (
                    <SelectItem key={option.value} value={option.value}>
                      <div>
                        <div>{option.label}</div>
                        <div className="text-xs text-gray-500">{option.type}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Operator</Label>
              <Select
                value={node.operator}
                onValueChange={(value) => updateCondition(node.id, { operator: value })}
                disabled={!node.field}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select operator" />
                </SelectTrigger>
                <SelectContent>
                  {node.field && getOperatorOptions(node.field).map(option => (
                    <SelectItem key={option.value} value={option.value}>
                      <div>
                        <div>{option.label}</div>
                        <div className="text-xs text-gray-500">{option.description}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Value</Label>
              {node.operator === 'in' || node.operator === 'not_in' || node.operator === 'contains_any' || node.operator === 'contains_all' ? (
                <Textarea
                  value={Array.isArray(node.value) ? node.value.join(', ') : node.value}
                  onChange={(e) => updateCondition(node.id, { 
                    value: e.target.value.split(',').map(v => v.trim()).filter(v => v) 
                  })}
                  placeholder="Enter values separated by commas"
                  rows={2}
                />
              ) : node.operator === 'between' ? (
                <div className="space-y-1">
                  <Input
                    placeholder="From"
                    value={Array.isArray(node.value) ? node.value[0] || '' : ''}
                    onChange={(e) => {
                      const current = Array.isArray(node.value) ? node.value : ['', ''];
                      updateCondition(node.id, { value: [e.target.value, current[1] || ''] });
                    }}
                  />
                  <Input
                    placeholder="To"
                    value={Array.isArray(node.value) ? node.value[1] || '' : ''}
                    onChange={(e) => {
                      const current = Array.isArray(node.value) ? node.value : ['', ''];
                      updateCondition(node.id, { value: [current[0] || '', e.target.value] });
                    }}
                  />
                </div>
              ) : getFieldType(node.field || '') === 'boolean' ? (
                <Select
                  value={node.value as string}
                  onValueChange={(value) => updateCondition(node.id, { value })}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select value" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="true">True</SelectItem>
                    <SelectItem value="false">False</SelectItem>
                  </SelectContent>
                </Select>
              ) : (
                <Input
                  value={node.value as string}
                  onChange={(e) => updateCondition(node.id, { value: e.target.value })}
                  placeholder="Enter value"
                  type={getFieldType(node.field || '') === 'number' ? 'number' : 'text'}
                />
              )}
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Label>Group Operator:</Label>
                <Select
                  value={node.logicalOperator}
                  onValueChange={(value: 'AND' | 'OR') => updateCondition(node.id, { logicalOperator: value })}
                >
                  <SelectTrigger className="w-20">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="AND">AND</SelectItem>
                    <SelectItem value="OR">OR</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex gap-1">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={(e) => {
                    e.stopPropagation();
                    addCondition(node.id);
                  }}
                >
                  <Plus className="w-4 h-4 mr-1" />
                  Condition
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={(e) => {
                    e.stopPropagation();
                    addGroup(node.id);
                  }}
                >
                  <Parentheses className="w-4 h-4 mr-1" />
                  Group
                </Button>
              </div>
            </div>

            {node.children && node.children.length > 0 ? (
              <div className="space-y-3">
                {node.children.map((child, index) => (
                  <div key={child.id || index}>
                    {renderConditionNode(child, depth + 1, index)}
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-4 text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 rounded border-2 border-dashed dark:border-gray-600">
                <p>Empty group. Add conditions or subgroups.</p>
              </div>
            )}
          </div>
        )}
      </div>
    );
  };

  const generateConditionText = (node: ConditionNode): string => {
    if (node.type === 'condition') {
      const negation = node.negated ? 'NOT ' : '';
      const field = node.field || '[field]';
      const operator = node.operator || '[operator]';
      const value = Array.isArray(node.value) 
        ? `[${node.value.join(', ')}]`
        : `"${node.value || '[value]'}"`;
      return `${negation}${field} ${operator} ${value}`;
    }

    if (node.children && node.children.length > 0) {
      const negation = node.negated ? 'NOT ' : '';
      const childTexts = node.children.map(generateConditionText);
      const operator = ` ${node.logicalOperator || 'AND'} `;
      return `${negation}(${childTexts.join(operator)})`;
    }

    return '';
  };

  const saveCondition = () => {
    if (rootCondition.children && rootCondition.children.length > 0) {
      setSavedConditions(prev => [...prev, { ...rootCondition, id: generateId() }]);
      alert('Condition saved successfully!');
    }
  };

  const loadTemplate = (templateId: string) => {
    const template = CONDITION_TEMPLATES.find(t => t.id === templateId);
    if (template) {
      setRootCondition({
        id: 'root',
        type: 'group',
        logicalOperator: 'AND',
        children: [{ ...template.condition, id: generateId() }]
      });
      setActiveTemplate(templateId);
    }
  };

  const exportCondition = () => {
    const dataStr = JSON.stringify(rootCondition, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);
    const exportFileDefaultName = 'condition_builder_export.json';
    
    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Advanced Condition Builder</h1>
          <p className="text-gray-600">Create complex logical conditions with AND/OR groups and nested logic</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={validateCurrentCondition}>
            <Eye className="w-4 h-4 mr-2" />
            Validate
          </Button>
          <Button variant="outline" onClick={exportCondition}>
            <Download className="w-4 h-4 mr-2" />
            Export
          </Button>
          <Button onClick={saveCondition} disabled={!rootCondition.children?.length}>
            <Save className="w-4 h-4 mr-2" />
            Save
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Main Builder */}
        <div className="lg:col-span-3 space-y-6">
          <Card>
            <CardHeader>
              <div className="flex justify-between items-center">
                <div>
                  <CardTitle>Condition Tree</CardTitle>
                  <CardDescription>Build complex conditions using drag-and-drop interface</CardDescription>
                </div>
                <div className="flex gap-2">
                  <Button size="sm" onClick={() => addCondition()}>
                    <Plus className="w-4 h-4 mr-1" />
                    Add Condition
                  </Button>
                  <Button size="sm" variant="outline" onClick={() => addGroup()}>
                    <Parentheses className="w-4 h-4 mr-1" />
                    Add Group
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              {rootCondition.children && rootCondition.children.length > 0 ? (
                <div className="space-y-4" onClick={() => setSelectedNode(null)}>
                  {renderConditionNode(rootCondition)}
                </div>
              ) : (
                <div className="text-center py-12 text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 rounded border-2 border-dashed dark:border-gray-600">
                  <Parentheses className="w-12 h-12 mx-auto mb-4 text-gray-400 dark:text-gray-500" />
                  <p className="text-lg font-medium mb-2">No conditions defined</p>
                  <p className="mb-4">Start by adding your first condition or use a template</p>
                  <div className="flex gap-2 justify-center">
                    <Button onClick={() => addCondition()}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add First Condition
                    </Button>
                    <Button variant="outline" onClick={() => addGroup()}>
                      <Parentheses className="w-4 h-4 mr-2" />
                      Add Group
                    </Button>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Generated Expression */}
          <Card>
            <CardHeader>
              <CardTitle>Generated Expression</CardTitle>
              <CardDescription>Human-readable representation of your condition logic</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-gray-100 dark:bg-gray-700 p-4 rounded-lg font-mono text-sm">
                {rootCondition.children && rootCondition.children.length > 0 
                  ? generateConditionText(rootCondition)
                  : 'No conditions defined'
                }
              </div>
            </CardContent>
          </Card>

          {/* Validation Results */}
          {validationResult && (
            <Card>
              <CardHeader>
                <CardTitle>Validation Results</CardTitle>
                <CardDescription>Condition validation and complexity analysis</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className={`p-3 rounded-lg ${
                  validationResult.isValid 
                    ? 'bg-green-50 border border-green-200' 
                    : 'bg-red-50 border border-red-200'
                }`}>
                  <h4 className={`font-medium ${
                    validationResult.isValid ? 'text-green-800' : 'text-red-800'
                  }`}>
                    {validationResult.isValid ? 'Valid Condition' : 'Invalid Condition'}
                  </h4>
                  <p className={`text-sm ${
                    validationResult.isValid ? 'text-green-600' : 'text-red-600'
                  }`}>
                    Complexity Score: {validationResult.complexity}
                  </p>
                </div>

                {validationResult.errors.length > 0 && (
                  <div className="space-y-2">
                    <h5 className="font-medium text-red-800">Errors:</h5>
                    {validationResult.errors.map((error, index) => (
                      <p key={index} className="text-sm text-red-600 bg-red-50 p-2 rounded">
                        • {error}
                      </p>
                    ))}
                  </div>
                )}

                {validationResult.warnings.length > 0 && (
                  <div className="space-y-2">
                    <h5 className="font-medium text-yellow-800">Warnings:</h5>
                    {validationResult.warnings.map((warning, index) => (
                      <p key={index} className="text-sm text-yellow-600 bg-yellow-50 p-2 rounded">
                        • {warning}
                      </p>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Condition Templates</CardTitle>
              <CardDescription>Pre-built condition patterns</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {CONDITION_TEMPLATES.map(template => (
                <div
                  key={template.id}
                  className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                    activeTemplate === template.id
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-gray-300'
                  }`}
                  onClick={() => loadTemplate(template.id)}
                >
                  <div className="flex justify-between items-start mb-2">
                    <h4 className="font-medium">{template.name}</h4>
                    <Badge variant="outline">{template.category}</Badge>
                  </div>
                  <p className="text-sm text-gray-600 mb-2">{template.description}</p>
                  <div className="flex justify-between items-center">
                    <div className="flex flex-wrap gap-1">
                      {template.tags.slice(0, 2).map(tag => (
                        <Badge key={tag} variant="secondary" className="text-xs">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                    <span className="text-xs text-gray-500">{template.usageCount} uses</span>
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
              <CardDescription>Common condition operations</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <Button
                variant="outline"
                className="w-full justify-start"
                onClick={() => {
                  setRootCondition({
                    id: 'root',
                    type: 'group',
                    logicalOperator: 'AND',
                    children: []
                  });
                  setSelectedNode(null);
                  setValidationResult(null);
                }}
              >
                <Trash2 className="w-4 h-4 mr-2" />
                Clear All
              </Button>
              
              <Button
                variant="outline"
                className="w-full justify-start"
                onClick={() => {
                  const copy = JSON.parse(JSON.stringify(rootCondition));
                  copy.id = generateId();
                  setSavedConditions(prev => [...prev, copy]);
                }}
                disabled={!rootCondition.children?.length}
              >
                <Copy className="w-4 h-4 mr-2" />
                Duplicate Current
              </Button>
            </CardContent>
          </Card>

          {savedConditions.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Saved Conditions</CardTitle>
                <CardDescription>Previously saved condition sets</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {savedConditions.slice(-5).map(condition => (
                  <div
                    key={condition.id}
                    className="p-3 border rounded-lg cursor-pointer hover:bg-gray-50"
                    onClick={() => setRootCondition(condition)}
                  >
                    <div className="text-sm font-mono text-gray-600 truncate">
                      {generateConditionText(condition)}
                    </div>
                    <div className="text-xs text-gray-500 mt-1">
                      {condition.children?.length || 0} condition(s)
                    </div>
                  </div>
                ))}
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
};

export default ConditionBuilder;