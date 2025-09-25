'use client'

import React, { useState, useEffect } from 'react'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { 
  Plus, 
  Trash2, 
  Edit, 
  Copy, 
  Eye, 
  Settings, 
  Users, 
  Workflow,
  BarChart3,
  Clock,
  Shield,
  Zap
} from 'lucide-react'

interface DynamicRule {
  id: string
  name: string
  description: string
  logic_operator: 'AND' | 'OR' | 'NOT' | 'XOR'
  conditions: RuleCondition[]
  actions: RuleActions
  is_active: boolean
  priority: number
  created_at: string
  updated_at: string
}

interface RuleCondition {
  type: 'behavioral' | 'temporal' | 'contextual'
  field: string
  operator: 'equals' | 'greater_than' | 'less_than' | 'contains' | 'in_range'
  value: string | number | boolean
  weight: number
}

interface RuleActions {
  assign_groups: string[]
  remove_groups: string[]
  set_permissions: string[]
  notify_admin: boolean
  log_event: boolean
}

interface GroupTemplate {
  id: string
  name: string
  description: string
  parameters: TemplateParameter[]
  permission_patterns: string[]
  auto_assignment_enabled: boolean
  evaluation_conditions: Record<string, any>
}

interface TemplateParameter {
  name: string
  type: 'string' | 'number' | 'boolean' | 'array'
  required: boolean
  default_value?: any
  validation_rules?: string[]
}

interface GroupHierarchy {
  id: string
  parent_id?: string
  name: string
  level: number
  children: GroupHierarchy[]
  permissions: string[]
  inheritance_type: 'full' | 'partial' | 'none'
}

export default function DynamicGroupManagement() {
  const [activeTab, setActiveTab] = useState('rules')
  const [rules, setRules] = useState<DynamicRule[]>([])
  const [templates, setTemplates] = useState<GroupTemplate[]>([])
  const [hierarchies, setHierarchies] = useState<GroupHierarchy[]>([])
  const [selectedRule, setSelectedRule] = useState<DynamicRule | null>(null)
  const [isRuleBuilderOpen, setIsRuleBuilderOpen] = useState(false)

  const [newRule, setNewRule] = useState<Partial<DynamicRule>>({
    name: '',
    description: '',
    logic_operator: 'AND',
    conditions: [],
    actions: {
      assign_groups: [],
      remove_groups: [],
      set_permissions: [],
      notify_admin: false,
      log_event: true
    },
    is_active: true,
    priority: 100
  })

  useEffect(() => {
    loadRules()
    loadTemplates()
    loadHierarchies()
  }, [])

  const loadRules = async () => {
    // Mock data - replace with actual API call
    setRules([
      {
        id: '1',
        name: 'High Activity Premium Users',
        description: 'Auto-assign premium permissions for highly active users',
        logic_operator: 'AND',
        conditions: [
          {
            type: 'behavioral',
            field: 'daily_activity_score',
            operator: 'greater_than',
            value: 80,
            weight: 0.7
          },
          {
            type: 'behavioral',
            field: 'subscription_tier',
            operator: 'equals',
            value: 'premium',
            weight: 1.0
          }
        ],
        actions: {
          assign_groups: ['premium_analytics', 'advanced_features'],
          remove_groups: [],
          set_permissions: ['epsx:analytics:advanced', 'epsx:trading:premium'],
          notify_admin: false,
          log_event: true
        },
        is_active: true,
        priority: 90,
        created_at: '2024-01-15T10:30:00Z',
        updated_at: '2024-01-20T14:15:00Z'
      }
    ])
  }

  const loadTemplates = async () => {
    // Mock data - replace with actual API call
    setTemplates([
      {
        id: '1',
        name: 'Tier-based Analytics Access',
        description: 'Template for tier-based analytics permissions',
        parameters: [
          {
            name: 'tier',
            type: 'string',
            required: true,
            validation_rules: ['basic', 'premium', 'enterprise']
          },
          {
            name: 'max_queries_per_day',
            type: 'number',
            required: false,
            default_value: 100
          }
        ],
        permission_patterns: [
          'epsx:analytics:{tier}',
          'epsx:data:read:{tier}'
        ],
        auto_assignment_enabled: true,
        evaluation_conditions: {
          subscription_tier: '{tier}',
          account_status: 'active'
        }
      }
    ])
  }

  const loadHierarchies = async () => {
    // Mock data - replace with actual API call
    setHierarchies([
      {
        id: '1',
        name: 'Trading Permissions',
        level: 0,
        permissions: ['epsx:trading:view'],
        inheritance_type: 'full',
        children: [
          {
            id: '2',
            parent_id: '1',
            name: 'Basic Trading',
            level: 1,
            permissions: ['epsx:trading:basic'],
            inheritance_type: 'full',
            children: [
              {
                id: '3',
                parent_id: '2',
                name: 'Advanced Trading',
                level: 2,
                permissions: ['epsx:trading:advanced', 'epsx:trading:api'],
                inheritance_type: 'partial',
                children: []
              }
            ]
          }
        ]
      }
    ])
  }

  const addCondition = () => {
    const newCondition: RuleCondition = {
      type: 'behavioral',
      field: '',
      operator: 'equals',
      value: '',
      weight: 1.0
    }
    setNewRule(prev => ({
      ...prev,
      conditions: [...(prev.conditions || []), newCondition]
    }))
  }

  const updateCondition = (index: number, updates: Partial<RuleCondition>) => {
    setNewRule(prev => ({
      ...prev,
      conditions: prev.conditions?.map((condition, i) => 
        i === index ? { ...condition, ...updates } : condition
      ) || []
    }))
  }

  const removeCondition = (index: number) => {
    setNewRule(prev => ({
      ...prev,
      conditions: prev.conditions?.filter((_, i) => i !== index) || []
    }))
  }

  const saveRule = async () => {
    if (!newRule.name || !newRule.conditions?.length) {
      alert('Please provide a name and at least one condition')
      return
    }

    // API call to save rule
    console.log('Saving rule:', newRule)
    
    setIsRuleBuilderOpen(false)
    setNewRule({
      name: '',
      description: '',
      logic_operator: 'AND',
      conditions: [],
      actions: {
        assign_groups: [],
        remove_groups: [],
        set_permissions: [],
        notify_admin: false,
        log_event: true
      },
      is_active: true,
      priority: 100
    })
    loadRules()
  }

  const RuleBuilder = () => (
    <Card className="p-6">
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h3 className="text-xl font-semibold">Visual Rule Builder</h3>
          <Button 
            variant="outline" 
            onClick={() => setIsRuleBuilderOpen(false)}
          >
            Cancel
          </Button>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label htmlFor="rule-name">Rule Name</Label>
            <Input
              id="rule-name"
              value={newRule.name || ''}
              onChange={(e) => setNewRule(prev => ({ ...prev, name: e.target.value }))}
              placeholder="Enter rule name"
            />
          </div>
          <div>
            <Label htmlFor="priority">Priority (1-100)</Label>
            <Input
              id="priority"
              type="number"
              min="1"
              max="100"
              value={newRule.priority || 100}
              onChange={(e) => setNewRule(prev => ({ ...prev, priority: parseInt(e.target.value) }))}
            />
          </div>
        </div>

        <div>
          <Label htmlFor="description">Description</Label>
          <Textarea
            id="description"
            value={newRule.description || ''}
            onChange={(e) => setNewRule(prev => ({ ...prev, description: e.target.value }))}
            placeholder="Describe what this rule does"
            rows={2}
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Label htmlFor="logic-operator">Logic Operator</Label>
            <Select 
              value={newRule.logic_operator || 'AND'}
              onValueChange={(value: 'AND' | 'OR' | 'NOT' | 'XOR') => 
                setNewRule(prev => ({ ...prev, logic_operator: value }))
              }
            >
              <SelectTrigger className="w-24">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="AND">AND</SelectItem>
                <SelectItem value="OR">OR</SelectItem>
                <SelectItem value="NOT">NOT</SelectItem>
                <SelectItem value="XOR">XOR</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-center space-x-2">
            <Switch 
              id="is-active"
              checked={newRule.is_active || false}
              onCheckedChange={(checked) => 
                setNewRule(prev => ({ ...prev, is_active: checked }))
              }
            />
            <Label htmlFor="is-active">Active</Label>
          </div>
        </div>

        <div>
          <div className="flex items-center justify-between mb-4">
            <h4 className="text-lg font-medium">Conditions</h4>
            <Button onClick={addCondition} size="sm">
              <Plus className="h-4 w-4 mr-1" />
              Add Condition
            </Button>
          </div>
          
          {newRule.conditions?.map((condition, index) => (
            <Card key={index} className="p-4 mb-3">
              <div className="grid grid-cols-5 gap-3 items-end">
                <div>
                  <Label>Type</Label>
                  <Select 
                    value={condition.type}
                    onValueChange={(value: 'behavioral' | 'temporal' | 'contextual') =>
                      updateCondition(index, { type: value })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="behavioral">Behavioral</SelectItem>
                      <SelectItem value="temporal">Temporal</SelectItem>
                      <SelectItem value="contextual">Contextual</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div>
                  <Label>Field</Label>
                  <Input
                    value={condition.field}
                    onChange={(e) => updateCondition(index, { field: e.target.value })}
                    placeholder="Field name"
                  />
                </div>
                
                <div>
                  <Label>Operator</Label>
                  <Select 
                    value={condition.operator}
                    onValueChange={(value: any) => updateCondition(index, { operator: value })}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="equals">Equals</SelectItem>
                      <SelectItem value="greater_than">Greater Than</SelectItem>
                      <SelectItem value="less_than">Less Than</SelectItem>
                      <SelectItem value="contains">Contains</SelectItem>
                      <SelectItem value="in_range">In Range</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div>
                  <Label>Value</Label>
                  <Input
                    value={String(condition.value)}
                    onChange={(e) => updateCondition(index, { value: e.target.value })}
                    placeholder="Value"
                  />
                </div>
                
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => removeCondition(index)}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </Card>
          ))}
        </div>

        <div>
          <h4 className="text-lg font-medium mb-4">Actions</h4>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Assign Groups</Label>
              <Input
                placeholder="Group names (comma-separated)"
                value={newRule.actions?.assign_groups?.join(', ') || ''}
                onChange={(e) => setNewRule(prev => ({
                  ...prev,
                  actions: {
                    ...prev.actions!,
                    assign_groups: e.target.value.split(',').map(g => g.trim()).filter(Boolean)
                  }
                }))}
              />
            </div>
            <div>
              <Label>Set Permissions</Label>
              <Input
                placeholder="Permissions (comma-separated)"
                value={newRule.actions?.set_permissions?.join(', ') || ''}
                onChange={(e) => setNewRule(prev => ({
                  ...prev,
                  actions: {
                    ...prev.actions!,
                    set_permissions: e.target.value.split(',').map(p => p.trim()).filter(Boolean)
                  }
                }))}
              />
            </div>
          </div>
          
          <div className="flex items-center space-x-6 mt-4">
            <div className="flex items-center space-x-2">
              <Switch 
                id="notify-admin"
                checked={newRule.actions?.notify_admin || false}
                onCheckedChange={(checked) => 
                  setNewRule(prev => ({
                    ...prev,
                    actions: { ...prev.actions!, notify_admin: checked }
                  }))
                }
              />
              <Label htmlFor="notify-admin">Notify Admin</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Switch 
                id="log-event"
                checked={newRule.actions?.log_event || false}
                onCheckedChange={(checked) => 
                  setNewRule(prev => ({
                    ...prev,
                    actions: { ...prev.actions!, log_event: checked }
                  }))
                }
              />
              <Label htmlFor="log-event">Log Event</Label>
            </div>
          </div>
        </div>

        <div className="flex justify-end space-x-2">
          <Button variant="outline" onClick={() => setIsRuleBuilderOpen(false)}>
            Cancel
          </Button>
          <Button onClick={saveRule}>
            <Zap className="h-4 w-4 mr-1" />
            Create Rule
          </Button>
        </div>
      </div>
    </Card>
  )

  const RulesTab = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Dynamic Rules</h2>
          <p className="text-gray-600">Create and manage intelligent group assignment rules</p>
        </div>
        <Button onClick={() => setIsRuleBuilderOpen(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Create Rule
        </Button>
      </div>

      {isRuleBuilderOpen && <RuleBuilder />}

      <div className="grid gap-4">
        {rules.map((rule) => (
          <Card key={rule.id} className="p-6">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center space-x-2 mb-2">
                  <h3 className="text-lg font-semibold">{rule.name}</h3>
                  <Badge variant={rule.is_active ? 'default' : 'secondary'}>
                    {rule.is_active ? 'Active' : 'Inactive'}
                  </Badge>
                  <Badge variant="outline">
                    Priority: {rule.priority}
                  </Badge>
                </div>
                <p className="text-gray-600 mb-3">{rule.description}</p>
                
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <span className="font-medium">Logic:</span> {rule.logic_operator}
                  </div>
                  <div>
                    <span className="font-medium">Conditions:</span> {rule.conditions.length}
                  </div>
                  <div>
                    <span className="font-medium">Actions:</span> {rule.actions.assign_groups.length} groups
                  </div>
                </div>
              </div>
              
              <div className="flex items-center space-x-2">
                <Button variant="outline" size="sm">
                  <Eye className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm">
                  <Edit className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm">
                  <Copy className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm">
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  )

  const TemplatesTab = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Group Templates</h2>
          <p className="text-gray-600">Reusable patterns for consistent group creation</p>
        </div>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          Create Template
        </Button>
      </div>

      <div className="grid gap-4">
        {templates.map((template) => (
          <Card key={template.id} className="p-6">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h3 className="text-lg font-semibold mb-2">{template.name}</h3>
                <p className="text-gray-600 mb-3">{template.description}</p>
                
                <div className="flex flex-wrap gap-2 mb-3">
                  {template.parameters.map((param, index) => (
                    <Badge key={index} variant="outline">
                      {param.name}: {param.type}
                      {param.required && <span className="text-red-500">*</span>}
                    </Badge>
                  ))}
                </div>
                
                <div className="text-sm">
                  <span className="font-medium">Permission Patterns:</span>
                  <div className="mt-1 space-x-2">
                    {template.permission_patterns.map((pattern, index) => (
                      <Badge key={index} variant="secondary">{pattern}</Badge>
                    ))}
                  </div>
                </div>
              </div>
              
              <div className="flex items-center space-x-2">
                <Button variant="outline" size="sm">
                  <Settings className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm">
                  <Copy className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm">
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  )

  const HierarchyTab = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Permission Hierarchies</h2>
          <p className="text-gray-600">Organize groups in tree structures with inheritance</p>
        </div>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          Create Hierarchy
        </Button>
      </div>

      <div className="space-y-4">
        {hierarchies.map((hierarchy) => (
          <Card key={hierarchy.id} className="p-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold">{hierarchy.name}</h3>
                <Badge variant="outline">
                  {hierarchy.inheritance_type} inheritance
                </Badge>
              </div>
              
              <div className="pl-4 border-l-2 border-gray-200">
                {hierarchy.children.map((child) => (
                  <div key={child.id} className="py-2">
                    <div className="flex items-center justify-between">
                      <span className="font-medium">{child.name}</span>
                      <div className="flex space-x-1">
                        {child.permissions.map((perm, index) => (
                          <Badge key={index} variant="secondary" className="text-xs">
                            {perm}
                          </Badge>
                        ))}
                      </div>
                    </div>
                    
                    {child.children.map((grandchild) => (
                      <div key={grandchild.id} className="pl-4 pt-2 border-l border-gray-100">
                        <div className="flex items-center justify-between">
                          <span className="text-sm">{grandchild.name}</span>
                          <div className="flex space-x-1">
                            {grandchild.permissions.map((perm, index) => (
                              <Badge key={index} variant="outline" className="text-xs">
                                {perm}
                              </Badge>
                            ))}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  )

  const AnalyticsTab = () => (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Group Analytics</h2>
        <p className="text-gray-600">Monitor and analyze group performance</p>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <Users className="h-5 w-5 text-blue-500" />
            <span className="text-sm text-gray-600">Total Groups</span>
          </div>
          <div className="text-2xl font-bold">24</div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <Workflow className="h-5 w-5 text-green-500" />
            <span className="text-sm text-gray-600">Active Rules</span>
          </div>
          <div className="text-2xl font-bold">8</div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <BarChart3 className="h-5 w-5 text-purple-500" />
            <span className="text-sm text-gray-600">Templates</span>
          </div>
          <div className="text-2xl font-bold">5</div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <Clock className="h-5 w-5 text-orange-500" />
            <span className="text-sm text-gray-600">Auto-Assignments (24h)</span>
          </div>
          <div className="text-2xl font-bold">156</div>
        </Card>
      </div>
    </div>
  )

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-3xl font-bold">Dynamic Group Management</h1>
        <p className="text-gray-600">Advanced intelligent group system with rules, templates, and hierarchies</p>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="rules">
            <Workflow className="h-4 w-4 mr-2" />
            Rules
          </TabsTrigger>
          <TabsTrigger value="templates">
            <Settings className="h-4 w-4 mr-2" />
            Templates
          </TabsTrigger>
          <TabsTrigger value="hierarchy">
            <Shield className="h-4 w-4 mr-2" />
            Hierarchies
          </TabsTrigger>
          <TabsTrigger value="analytics">
            <BarChart3 className="h-4 w-4 mr-2" />
            Analytics
          </TabsTrigger>
        </TabsList>

        <TabsContent value="rules">
          <RulesTab />
        </TabsContent>

        <TabsContent value="templates">
          <TemplatesTab />
        </TabsContent>

        <TabsContent value="hierarchy">
          <HierarchyTab />
        </TabsContent>

        <TabsContent value="analytics">
          <AnalyticsTab />
        </TabsContent>
      </Tabs>
    </div>
  )
}