'use client';

import React, { useState, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { DragDropContext, Droppable, Draggable, DropResult } from '@hello-pangea/dnd';
import { Plus, X, Move, Play, Save, Download, Copy } from 'lucide-react';

interface Condition {
  id: string;
  field: string;
  operator: string;
  value: string;
  logicalOperator?: 'AND' | 'OR';
}

interface AssignmentRule {
  id: string;
  name: string;
  description: string;
  conditions: Condition[];
  targetProfile: string;
  priority: number;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

interface RuleTemplate {
  id: string;
  name: string;
  description: string;
  conditions: Condition[];
  targetProfile: string;
  category: string;
  usageCount: number;
}

interface TestResult {
  passed: boolean;
  matchedUsers: number;
  errors: string[];
  warnings: string[];
  executionTime: number;
}

const FIELD_OPTIONS = [
  { value: 'user.email', label: 'Email' },
  { value: 'user.registration_date', label: 'Registration Date' },
  { value: 'user.subscription_type', label: 'Subscription Type' },
  { value: 'user.department', label: 'Department' },
  { value: 'user.role_level', label: 'Role Level' },
  { value: 'user.account_type', label: 'Account Type' },
  { value: 'user.payment_verified', label: 'Payment Verified' },
  { value: 'user.trial_days_remaining', label: 'Trial Days Remaining' },
  { value: 'user.last_login', label: 'Last Login' },
  { value: 'user.tier', label: 'Current Tier' }
];

const OPERATOR_OPTIONS = [
  { value: '==', label: 'Equals' },
  { value: '!=', label: 'Not Equals' },
  { value: '>', label: 'Greater Than' },
  { value: '<', label: 'Less Than' },
  { value: '>=', label: 'Greater Than or Equal' },
  { value: '<=', label: 'Less Than or Equal' },
  { value: 'contains', label: 'Contains' },
  { value: 'starts_with', label: 'Starts With' },
  { value: 'ends_with', label: 'Ends With' },
  { value: 'in', label: 'In List' },
  { value: 'not_in', label: 'Not In List' }
];

const PROFILE_OPTIONS = [
  { value: 'bronze_package', label: 'Bronze Package' },
  { value: 'silver_package', label: 'Silver Package' },
  { value: 'gold_package', label: 'Gold Package' },
  { value: 'admin_profile', label: 'Admin Profile' },
  { value: 'trial_package', label: 'Trial Package' }
];

const RULE_TEMPLATES: RuleTemplate[] = [
  {
    id: 'template_1',
    name: 'New User Bronze Assignment',
    description: 'Assign Bronze package to new users',
    conditions: [
      { id: 'cond_1', field: 'user.registration_date', operator: '<', value: '24h' },
      { id: 'cond_2', field: 'user.tier', operator: '==', value: 'null', logicalOperator: 'AND' }
    ],
    targetProfile: 'bronze_package',
    category: 'Onboarding',
    usageCount: 245
  },
  {
    id: 'template_2',
    name: 'Premium User Upgrade',
    description: 'Auto-upgrade verified premium users',
    conditions: [
      { id: 'cond_3', field: 'user.payment_verified', operator: '==', value: 'true' },
      { id: 'cond_4', field: 'user.subscription_type', operator: '==', value: 'premium', logicalOperator: 'AND' }
    ],
    targetProfile: 'gold_package',
    category: 'Upgrades',
    usageCount: 89
  },
  {
    id: 'template_3',
    name: 'Admin Role Assignment',
    description: 'Assign admin roles to IT managers',
    conditions: [
      { id: 'cond_5', field: 'user.department', operator: '==', value: 'IT' },
      { id: 'cond_6', field: 'user.role_level', operator: '>=', value: 'manager', logicalOperator: 'AND' }
    ],
    targetProfile: 'admin_profile',
    category: 'Administration',
    usageCount: 12
  }
];

const VisualRuleBuilder: React.FC = () => {
  const [currentRule, setCurrentRule] = useState<AssignmentRule>({
    id: '',
    name: '',
    description: '',
    conditions: [],
    targetProfile: '',
    priority: 1,
    isActive: true,
    createdAt: '',
    updatedAt: ''
  });

  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [isTestingRule, setIsTestingRule] = useState(false);
  const [savedRules, setSavedRules] = useState<AssignmentRule[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<string>('');

  const generateId = () => Math.random().toString(36).substr(2, 9);

  const addCondition = () => {
    const newCondition: Condition = {
      id: generateId(),
      field: '',
      operator: '==',
      value: '',
      logicalOperator: currentRule.conditions.length > 0 ? 'AND' : undefined
    };
    setCurrentRule(prev => ({
      ...prev,
      conditions: [...prev.conditions, newCondition]
    }));
  };

  const updateCondition = (id: string, updates: Partial<Condition>) => {
    setCurrentRule(prev => ({
      ...prev,
      conditions: prev.conditions.map(condition =>
        condition.id === id ? { ...condition, ...updates } : condition
      )
    }));
  };

  const removeCondition = (id: string) => {
    setCurrentRule(prev => ({
      ...prev,
      conditions: prev.conditions.filter(condition => condition.id !== id)
    }));
  };

  const onDragEnd = useCallback((result: DropResult) => {
    if (!result.destination) return;

    const items = Array.from(currentRule.conditions);
    const [reorderedItem] = items.splice(result.source.index, 1);
    items.splice(result.destination.index, 0, reorderedItem);

    setCurrentRule(prev => ({
      ...prev,
      conditions: items
    }));
  }, [currentRule.conditions]);

  const testRule = async () => {
    setIsTestingRule(true);
    try {
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Mock test results
      const mockResult: TestResult = {
        passed: currentRule.conditions.length > 0 && currentRule.targetProfile !== '',
        matchedUsers: Math.floor(Math.random() * 500) + 50,
        errors: currentRule.conditions.length === 0 ? ['At least one condition is required'] : [],
        warnings: currentRule.targetProfile === '' ? ['Target profile not selected'] : [],
        executionTime: Math.floor(Math.random() * 200) + 50
      };

      setTestResult(mockResult);
    } catch (error) {
      console.error('Error testing rule:', error);
    } finally {
      setIsTestingRule(false);
    }
  };

  const saveRule = () => {
    if (!currentRule.name || currentRule.conditions.length === 0) {
      alert('Please provide a rule name and at least one condition');
      return;
    }

    const ruleToSave: AssignmentRule = {
      ...currentRule,
      id: currentRule.id || generateId(),
      createdAt: currentRule.createdAt || new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };

    setSavedRules(prev => {
      const existingIndex = prev.findIndex(rule => rule.id === ruleToSave.id);
      if (existingIndex >= 0) {
        const updated = [...prev];
        updated[existingIndex] = ruleToSave;
        return updated;
      }
      return [...prev, ruleToSave];
    });

    alert('Rule saved successfully!');
  };

  const loadTemplate = (templateId: string) => {
    const template = RULE_TEMPLATES.find(t => t.id === templateId);
    if (template) {
      setCurrentRule({
        id: '',
        name: template.name,
        description: template.description,
        conditions: template.conditions.map(c => ({ ...c, id: generateId() })),
        targetProfile: template.targetProfile,
        priority: 1,
        isActive: true,
        createdAt: '',
        updatedAt: ''
      });
    }
  };

  const exportRule = () => {
    const dataStr = JSON.stringify(currentRule, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);
    const exportFileDefaultName = `rule_${currentRule.name.replace(/\s+/g, '_').toLowerCase()}.json`;
    
    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  const generateRuleCode = () => {
    const conditions = currentRule.conditions.map((condition, index) => {
      const logicalOp = index > 0 ? ` ${condition.logicalOperator} ` : '';
      return `${logicalOp}${condition.field} ${condition.operator} ${condition.value}`;
    }).join('');

    return `
// Auto-generated assignment rule: ${currentRule.name}
if (${conditions}) {
  assignPermissionProfile(user, "${currentRule.targetProfile}");
}`;
  };

  const duplicateRule = () => {
    setCurrentRule(prev => ({
      ...prev,
      id: '',
      name: `${prev.name} (Copy)`,
      createdAt: '',
      updatedAt: ''
    }));
  };

  const clearRule = () => {
    setCurrentRule({
      id: '',
      name: '',
      description: '',
      conditions: [],
      targetProfile: '',
      priority: 1,
      isActive: true,
      createdAt: '',
      updatedAt: ''
    });
    setTestResult(null);
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Visual Rule Builder</h1>
          <p className="text-gray-600">Create and manage auto-assignment rules with drag-and-drop interface</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={duplicateRule} disabled={!currentRule.name}>
            <Copy className="w-4 h-4 mr-2" />
            Duplicate
          </Button>
          <Button variant="outline" onClick={exportRule} disabled={!currentRule.name}>
            <Download className="w-4 h-4 mr-2" />
            Export
          </Button>
          <Button variant="outline" onClick={clearRule}>
            Clear
          </Button>
          <Button onClick={saveRule} disabled={!currentRule.name || currentRule.conditions.length === 0}>
            <Save className="w-4 h-4 mr-2" />
            Save Rule
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Rule Builder */}
        <div className="lg:col-span-2 space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Rule Configuration</CardTitle>
              <CardDescription>Define the basic properties of your assignment rule</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="ruleName">Rule Name</Label>
                  <Input
                    id="ruleName"
                    value={currentRule.name}
                    onChange={(e) => setCurrentRule(prev => ({ ...prev, name: e.target.value }))}
                    placeholder="Enter rule name"
                  />
                </div>
                <div>
                  <Label htmlFor="priority">Priority</Label>
                  <Input
                    id="priority"
                    type="number"
                    min="1"
                    max="10"
                    value={currentRule.priority}
                    onChange={(e) => setCurrentRule(prev => ({ ...prev, priority: parseInt(e.target.value) || 1 }))}
                  />
                </div>
              </div>

              <div>
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  value={currentRule.description}
                  onChange={(e) => setCurrentRule(prev => ({ ...prev, description: e.target.value }))}
                  placeholder="Describe what this rule does"
                  rows={3}
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="targetProfile">Target Profile</Label>
                  <Select value={currentRule.targetProfile} onValueChange={(value) => setCurrentRule(prev => ({ ...prev, targetProfile: value }))}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select target profile" />
                    </SelectTrigger>
                    <SelectContent>
                      {PROFILE_OPTIONS.map(option => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-center space-x-2">
                  <Switch
                    id="isActive"
                    checked={currentRule.isActive}
                    onCheckedChange={(checked) => setCurrentRule(prev => ({ ...prev, isActive: checked }))}
                  />
                  <Label htmlFor="isActive">Active</Label>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <div className="flex justify-between items-center">
                <div>
                  <CardTitle>Conditions</CardTitle>
                  <CardDescription>Define the conditions that trigger this rule</CardDescription>
                </div>
                <Button onClick={addCondition} size="sm">
                  <Plus className="w-4 h-4 mr-2" />
                  Add Condition
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              {currentRule.conditions.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  <p>No conditions defined. Click &quot;Add Condition&quot; to get started.</p>
                </div>
              ) : (
                <DragDropContext onDragEnd={onDragEnd}>
                  <Droppable droppableId="conditions">
                    {(provided) => (
                      <div {...provided.droppableProps} ref={provided.innerRef} className="space-y-3">
                        {currentRule.conditions.map((condition, index) => (
                          <Draggable key={condition.id} draggableId={condition.id} index={index}>
                            {(provided) => (
                              <div
                                ref={provided.innerRef}
                                {...provided.draggableProps}
                                className="border rounded-lg p-4 bg-white space-y-3"
                              >
                                <div className="flex items-center justify-between">
                                  <div className="flex items-center gap-2">
                                    <div {...provided.dragHandleProps} className="cursor-move">
                                      <Move className="w-4 h-4 text-gray-400" />
                                    </div>
                                    {index > 0 && (
                                      <Select
                                        value={condition.logicalOperator}
                                        onValueChange={(value: 'AND' | 'OR') => updateCondition(condition.id, { logicalOperator: value })}
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
                                    <Badge variant="outline">Condition {index + 1}</Badge>
                                  </div>
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => removeCondition(condition.id)}
                                  >
                                    <X className="w-4 h-4" />
                                  </Button>
                                </div>

                                <div className="grid grid-cols-3 gap-3">
                                  <div>
                                    <Label>Field</Label>
                                    <Select
                                      value={condition.field}
                                      onValueChange={(value) => updateCondition(condition.id, { field: value })}
                                    >
                                      <SelectTrigger>
                                        <SelectValue placeholder="Select field" />
                                      </SelectTrigger>
                                      <SelectContent>
                                        {FIELD_OPTIONS.map(option => (
                                          <SelectItem key={option.value} value={option.value}>
                                            {option.label}
                                          </SelectItem>
                                        ))}
                                      </SelectContent>
                                    </Select>
                                  </div>

                                  <div>
                                    <Label>Operator</Label>
                                    <Select
                                      value={condition.operator}
                                      onValueChange={(value) => updateCondition(condition.id, { operator: value })}
                                    >
                                      <SelectTrigger>
                                        <SelectValue />
                                      </SelectTrigger>
                                      <SelectContent>
                                        {OPERATOR_OPTIONS.map(option => (
                                          <SelectItem key={option.value} value={option.value}>
                                            {option.label}
                                          </SelectItem>
                                        ))}
                                      </SelectContent>
                                    </Select>
                                  </div>

                                  <div>
                                    <Label>Value</Label>
                                    <Input
                                      value={condition.value}
                                      onChange={(e) => updateCondition(condition.id, { value: e.target.value })}
                                      placeholder="Enter value"
                                    />
                                  </div>
                                </div>
                              </div>
                            )}
                          </Draggable>
                        ))}
                        {provided.placeholder}
                      </div>
                    )}
                  </Droppable>
                </DragDropContext>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Rule Preview</CardTitle>
              <CardDescription>Preview of the generated rule logic</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-gray-100 p-3 rounded font-mono text-sm">
                {currentRule.conditions.length > 0 ? (
                  currentRule.conditions.map((condition, index) => (
                    <div key={condition.id}>
                      {index > 0 && <span className="text-blue-600">{condition.logicalOperator} </span>}
                      <span className="text-green-600">{condition.field}</span>
                      <span className="text-purple-600"> {condition.operator} </span>
                      <span className="text-orange-600">&quot;{condition.value}&quot;</span>
                    </div>
                  ))
                ) : (
                  <span className="text-gray-500">No conditions defined</span>
                )}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Test Rule</CardTitle>
              <CardDescription>Test your rule against the user database</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <Button
                onClick={testRule}
                disabled={isTestingRule || currentRule.conditions.length === 0}
                className="w-full"
              >
                {isTestingRule ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                    Testing...
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4 mr-2" />
                    Test Rule
                  </>
                )}
              </Button>

              {testResult && (
                <div className="space-y-3">
                  <div className={`p-3 rounded-lg ${testResult.passed ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'}`}>
                    <h4 className={`font-medium ${testResult.passed ? 'text-green-800' : 'text-red-800'}`}>
                      {testResult.passed ? 'Test Passed' : 'Test Failed'}
                    </h4>
                    <p className={`text-sm ${testResult.passed ? 'text-green-600' : 'text-red-600'}`}>
                      {testResult.passed 
                        ? `Would match ${testResult.matchedUsers} users`
                        : 'Rule has validation errors'
                      }
                    </p>
                  </div>

                  {testResult.errors.length > 0 && (
                    <div className="space-y-1">
                      <h5 className="text-sm font-medium text-red-800">Errors:</h5>
                      {testResult.errors.map((error, index) => (
                        <p key={index} className="text-sm text-red-600">• {error}</p>
                      ))}
                    </div>
                  )}

                  {testResult.warnings.length > 0 && (
                    <div className="space-y-1">
                      <h5 className="text-sm font-medium text-yellow-800">Warnings:</h5>
                      {testResult.warnings.map((warning, index) => (
                        <p key={index} className="text-sm text-yellow-600">• {warning}</p>
                      ))}
                    </div>
                  )}

                  <p className="text-xs text-gray-600">
                    Execution time: {testResult.executionTime}ms
                  </p>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Rule Templates</CardTitle>
              <CardDescription>Start with a pre-built template</CardDescription>
            </CardHeader>
            <CardContent>
              <Select value={selectedTemplate} onValueChange={setSelectedTemplate}>
                <SelectTrigger>
                  <SelectValue placeholder="Select a template" />
                </SelectTrigger>
                <SelectContent>
                  {RULE_TEMPLATES.map(template => (
                    <SelectItem key={template.id} value={template.id}>
                      {template.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              {selectedTemplate && (
                <div className="mt-3 space-y-2">
                  {(() => {
                    const template = RULE_TEMPLATES.find(t => t.id === selectedTemplate);
                    return template ? (
                      <>
                        <p className="text-sm text-gray-600">{template.description}</p>
                        <div className="flex justify-between items-center">
                          <Badge variant="outline">{template.category}</Badge>
                          <span className="text-xs text-gray-500">{template.usageCount} uses</span>
                        </div>
                        <Button
                          size="sm"
                          onClick={() => loadTemplate(selectedTemplate)}
                          className="w-full"
                        >
                          Load Template
                        </Button>
                      </>
                    ) : null;
                  })()}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Generated Code Preview */}
      <Card>
        <CardHeader>
          <CardTitle>Generated Code</CardTitle>
          <CardDescription>Auto-generated code representation of your rule</CardDescription>
        </CardHeader>
        <CardContent>
          <pre className="bg-gray-100 p-4 rounded-lg text-sm overflow-x-auto">
            <code>{generateRuleCode()}</code>
          </pre>
        </CardContent>
      </Card>

      {/* Saved Rules */}
      {savedRules.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Saved Rules</CardTitle>
            <CardDescription>Previously created assignment rules</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {savedRules.map(rule => (
                <div key={rule.id} className="flex items-center justify-between p-3 border rounded-lg">
                  <div>
                    <h4 className="font-medium">{rule.name}</h4>
                    <p className="text-sm text-gray-600">{rule.description}</p>
                    <div className="flex gap-2 mt-1">
                      <Badge variant="outline">{rule.targetProfile}</Badge>
                      <Badge className={rule.isActive ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'}>
                        {rule.isActive ? 'Active' : 'Inactive'}
                      </Badge>
                    </div>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setCurrentRule(rule)}
                  >
                    Edit
                  </Button>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default VisualRuleBuilder;