'use client'

import React, { useState, useCallback } from 'react'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { 
  Plus, 
  Trash2, 
  Play, 
  Save, 
  Copy,
  Move,
  Link,
  GitBranch,
  Zap,
  Target,
  Clock,
  User,
  Settings,
  ChevronDown,
  ChevronRight
} from 'lucide-react'

interface RuleNode {
  id: string
  type: 'condition' | 'group' | 'action'
  operator?: 'AND' | 'OR' | 'NOT' | 'XOR'
  children?: RuleNode[]
  data?: {
    field?: string
    operator?: string
    value?: any
    weight?: number
    conditionType?: 'behavioral' | 'temporal' | 'contextual'
    actionType?: 'assign_group' | 'remove_group' | 'set_permission' | 'notify'
  }
  collapsed?: boolean
}

interface RuleTemplate {
  id: string
  name: string
  description: string
  category: string
  template: RuleNode
  popular?: boolean
}

const RULE_TEMPLATES: RuleTemplate[] = [
  {
    id: 'high_activity',
    name: 'High Activity User',
    description: 'Users with high engagement scores',
    category: 'Behavioral',
    popular: true,
    template: {
      id: 'root',
      type: 'group',
      operator: 'AND',
      children: [
        {
          id: 'cond1',
          type: 'condition',
          data: {
            field: 'daily_activity_score',
            operator: 'greater_than',
            value: 80,
            weight: 1.0,
            conditionType: 'behavioral'
          }
        },
        {
          id: 'cond2',
          type: 'condition',
          data: {
            field: 'sessions_per_week',
            operator: 'greater_than',
            value: 5,
            weight: 0.8,
            conditionType: 'behavioral'
          }
        }
      ]
    }
  },
  {
    id: 'business_hours',
    name: 'Business Hours Access',
    description: 'Restrict access to business hours',
    category: 'Temporal',
    template: {
      id: 'root',
      type: 'group',
      operator: 'AND',
      children: [
        {
          id: 'time1',
          type: 'condition',
          data: {
            field: 'current_hour',
            operator: 'in_range',
            value: [9, 17],
            weight: 1.0,
            conditionType: 'temporal'
          }
        },
        {
          id: 'time2',
          type: 'condition',
          data: {
            field: 'is_weekend',
            operator: 'equals',
            value: false,
            weight: 1.0,
            conditionType: 'temporal'
          }
        }
      ]
    }
  },
  {
    id: 'mobile_premium',
    name: 'Mobile Premium Users',
    description: 'Premium features for mobile users',
    category: 'Contextual',
    template: {
      id: 'root',
      type: 'group',
      operator: 'AND',
      children: [
        {
          id: 'device',
          type: 'condition',
          data: {
            field: 'device_type',
            operator: 'equals',
            value: 'mobile',
            weight: 1.0,
            conditionType: 'contextual'
          }
        },
        {
          id: 'sub',
          type: 'condition',
          data: {
            field: 'subscription_tier',
            operator: 'equals',
            value: 'premium',
            weight: 1.0,
            conditionType: 'behavioral'
          }
        }
      ]
    }
  }
]

const FIELD_OPTIONS = {
  behavioral: [
    'daily_activity_score',
    'weekly_activity_score',
    'sessions_per_day',
    'sessions_per_week',
    'features_used',
    'subscription_tier',
    'account_age_days',
    'total_transactions',
    'last_login_days_ago'
  ],
  temporal: [
    'current_hour',
    'current_day_of_week',
    'is_weekend',
    'is_business_hours',
    'timezone',
    'session_duration_minutes',
    'time_since_last_action'
  ],
  contextual: [
    'device_type',
    'platform',
    'browser',
    'ip_country',
    'is_mobile',
    'screen_width',
    'connection_type',
    'app_version'
  ]
}

const OPERATORS = {
  string: ['equals', 'not_equals', 'contains', 'starts_with', 'ends_with', 'regex'],
  number: ['equals', 'not_equals', 'greater_than', 'less_than', 'greater_equal', 'less_equal', 'in_range'],
  boolean: ['equals', 'not_equals'],
  array: ['contains', 'contains_all', 'contains_any', 'length_equals', 'length_greater', 'length_less']
}

export default function VisualRuleBuilder({ onSave }: { onSave?: (rule: RuleNode) => void }) {
  const [ruleTree, setRuleTree] = useState<RuleNode>({
    id: 'root',
    type: 'group',
    operator: 'AND',
    children: []
  })
  
  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [showTemplates, setShowTemplates] = useState(false)
  const [ruleName, setRuleName] = useState('')
  const [ruleDescription, setRuleDescription] = useState('')

  const generateId = () => Math.random().toString(36).substr(2, 9)

  const updateNode = useCallback((nodeId: string, updates: Partial<RuleNode>) => {
    const updateRecursive = (nodes: RuleNode[]): RuleNode[] => {
      return nodes.map(node => {
        if (node.id === nodeId) {
          return { ...node, ...updates }
        }
        if (node.children) {
          return { ...node, children: updateRecursive(node.children) }
        }
        return node
      })
    }

    if (ruleTree.id === nodeId) {
      setRuleTree({ ...ruleTree, ...updates })
    } else {
      setRuleTree({
        ...ruleTree,
        children: ruleTree.children ? updateRecursive(ruleTree.children) : []
      })
    }
  }, [ruleTree])

  const addNode = useCallback((parentId: string, nodeType: 'condition' | 'group' | 'action') => {
    const newNode: RuleNode = {
      id: generateId(),
      type: nodeType,
      ...(nodeType === 'group' && { operator: 'AND', children: [] }),
      ...(nodeType === 'condition' && {
        data: {
          field: '',
          operator: 'equals',
          value: '',
          weight: 1.0,
          conditionType: 'behavioral'
        }
      }),
      ...(nodeType === 'action' && {
        data: {
          actionType: 'assign_group',
          value: ''
        }
      })
    }

    const addToChildren = (nodes: RuleNode[]): RuleNode[] => {
      return nodes.map(node => {
        if (node.id === parentId) {
          return {
            ...node,
            children: [...(node.children || []), newNode]
          }
        }
        if (node.children) {
          return { ...node, children: addToChildren(node.children) }
        }
        return node
      })
    }

    if (ruleTree.id === parentId) {
      setRuleTree({
        ...ruleTree,
        children: [...(ruleTree.children || []), newNode]
      })
    } else {
      setRuleTree({
        ...ruleTree,
        children: ruleTree.children ? addToChildren(ruleTree.children) : []
      })
    }
  }, [ruleTree])

  const removeNode = useCallback((nodeId: string) => {
    const removeFromChildren = (nodes: RuleNode[]): RuleNode[] => {
      return nodes
        .filter(node => node.id !== nodeId)
        .map(node => ({
          ...node,
          children: node.children ? removeFromChildren(node.children) : undefined
        }))
    }

    setRuleTree({
      ...ruleTree,
      children: ruleTree.children ? removeFromChildren(ruleTree.children) : []
    })
  }, [ruleTree])

  const applyTemplate = (template: RuleTemplate) => {
    setRuleTree({ ...template.template, id: 'root' })
    setRuleName(template.name)
    setRuleDescription(template.description)
    setShowTemplates(false)
  }

  const testRule = async () => {
    // Mock rule testing
    console.log('Testing rule:', ruleTree)
    alert('Rule test completed! Check console for details.')
  }

  const saveRule = async () => {
    if (!ruleName.trim()) {
      alert('Please provide a rule name')
      return
    }
    
    const ruleData = {
      name: ruleName,
      description: ruleDescription,
      tree: ruleTree
    }
    
    console.log('Saving rule:', ruleData)
    onSave?.(ruleTree)
    alert('Rule saved successfully!')
  }

  const RenderNode = ({ node, depth = 0 }: { node: RuleNode; depth?: number }) => {
    const isSelected = selectedNode === node.id
    const hasChildren = node.children && node.children.length > 0

    const getNodeIcon = (type: string) => {
      switch (type) {
        case 'condition':
          return <Target className="h-4 w-4" />
        case 'group':
          return <GitBranch className="h-4 w-4" />
        case 'action':
          return <Zap className="h-4 w-4" />
        default:
          return <Settings className="h-4 w-4" />
      }
    }

    const getConditionTypeColor = (type: string) => {
      switch (type) {
        case 'behavioral':
          return 'bg-blue-100 text-blue-800'
        case 'temporal':
          return 'bg-green-100 text-green-800'
        case 'contextual':
          return 'bg-purple-100 text-purple-800'
        default:
          return 'bg-gray-100 text-gray-800'
      }
    }

    return (
      <div className={`ml-${depth * 6}`}>
        <Card 
          className={`p-3 mb-2 cursor-pointer transition-all ${
            isSelected ? 'ring-2 ring-blue-500 shadow-md' : 'hover:shadow-sm'
          }`}
          onClick={() => setSelectedNode(node.id)}
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              {hasChildren && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="p-1 h-6 w-6"
                  onClick={(e) => {
                    e.stopPropagation()
                    updateNode(node.id, { collapsed: !node.collapsed })
                  }}
                >
                  {node.collapsed ? <ChevronRight className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
                </Button>
              )}
              
              {getNodeIcon(node.type)}
              
              {node.type === 'group' && (
                <Badge variant="outline" className="text-xs">
                  {node.operator}
                </Badge>
              )}
              
              {node.type === 'condition' && node.data?.conditionType && (
                <Badge className={`text-xs ${getConditionTypeColor(node.data.conditionType)}`}>
                  {node.data.conditionType}
                </Badge>
              )}
              
              <span className="text-sm font-medium">
                {node.type === 'condition' && node.data?.field 
                  ? `${node.data.field} ${node.data.operator} ${node.data.value}`
                  : node.type === 'action' && node.data?.actionType
                  ? `${node.data.actionType}: ${node.data.value}`
                  : `${node.type.charAt(0).toUpperCase() + node.type.slice(1)}`
                }
              </span>
            </div>
            
            <div className="flex items-center space-x-1">
              {node.type === 'group' && (
                <>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 w-6 p-1"
                    onClick={(e) => {
                      e.stopPropagation()
                      addNode(node.id, 'condition')
                    }}
                  >
                    <Plus className="h-3 w-3" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 w-6 p-1"
                    onClick={(e) => {
                      e.stopPropagation()
                      addNode(node.id, 'group')
                    }}
                  >
                    <GitBranch className="h-3 w-3" />
                  </Button>
                </>
              )}
              
              {node.id !== 'root' && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 w-6 p-1 text-red-600 hover:text-red-800"
                  onClick={(e) => {
                    e.stopPropagation()
                    removeNode(node.id)
                  }}
                >
                  <Trash2 className="h-3 w-3" />
                </Button>
              )}
            </div>
          </div>
        </Card>
        
        {hasChildren && !node.collapsed && (
          <div className="ml-4 border-l-2 border-gray-200 pl-2">
            {node.children!.map((child) => (
              <RenderNode key={child.id} node={child} depth={depth + 1} />
            ))}
          </div>
        )}
      </div>
    )
  }

  const NodeEditor = () => {
    if (!selectedNode) return null

    const findNode = (nodes: RuleNode[], id: string): RuleNode | null => {
      for (const node of nodes) {
        if (node.id === id) return node
        if (node.children) {
          const found = findNode(node.children, id)
          if (found) return found
        }
      }
      return null
    }

    const node = selectedNode === 'root' ? ruleTree : findNode(ruleTree.children || [], selectedNode)
    if (!node) return null

    return (
      <Card className="p-4">
        <h3 className="text-lg font-semibold mb-4">
          Edit {node.type.charAt(0).toUpperCase() + node.type.slice(1)}
        </h3>
        
        {node.type === 'group' && (
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Logic Operator</label>
              <Select 
                value={node.operator || 'AND'}
                onValueChange={(value: 'AND' | 'OR' | 'NOT' | 'XOR') => 
                  updateNode(node.id, { operator: value })
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="AND">AND (All conditions must match)</SelectItem>
                  <SelectItem value="OR">OR (Any condition can match)</SelectItem>
                  <SelectItem value="NOT">NOT (Invert the result)</SelectItem>
                  <SelectItem value="XOR">XOR (Exactly one condition must match)</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        )}
        
        {node.type === 'condition' && (
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Condition Type</label>
              <Select 
                value={node.data?.conditionType || 'behavioral'}
                onValueChange={(value: 'behavioral' | 'temporal' | 'contextual') => 
                  updateNode(node.id, { 
                    data: { 
                      ...node.data, 
                      conditionType: value,
                      field: '' // Reset field when type changes
                    } 
                  })
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="behavioral">
                    <div className="flex items-center space-x-2">
                      <User className="h-4 w-4" />
                      <span>Behavioral (User activity)</span>
                    </div>
                  </SelectItem>
                  <SelectItem value="temporal">
                    <div className="flex items-center space-x-2">
                      <Clock className="h-4 w-4" />
                      <span>Temporal (Time-based)</span>
                    </div>
                  </SelectItem>
                  <SelectItem value="contextual">
                    <div className="flex items-center space-x-2">
                      <Settings className="h-4 w-4" />
                      <span>Contextual (Environment)</span>
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div>
              <label className="text-sm font-medium">Field</label>
              <Select 
                value={node.data?.field || ''}
                onValueChange={(value) => 
                  updateNode(node.id, { 
                    data: { ...node.data, field: value } 
                  })
                }
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a field" />
                </SelectTrigger>
                <SelectContent>
                  {FIELD_OPTIONS[node.data?.conditionType || 'behavioral']?.map((field) => (
                    <SelectItem key={field} value={field}>
                      {field.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Operator</label>
                <Select 
                  value={node.data?.operator || 'equals'}
                  onValueChange={(value) => 
                    updateNode(node.id, { 
                      data: { ...node.data, operator: value } 
                    })
                  }
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {OPERATORS.number.map((op) => (
                      <SelectItem key={op} value={op}>
                        {op.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <label className="text-sm font-medium">Value</label>
                <Input
                  value={String(node.data?.value || '')}
                  onChange={(e) => 
                    updateNode(node.id, { 
                      data: { ...node.data, value: e.target.value } 
                    })
                  }
                  placeholder="Enter value"
                />
              </div>
            </div>
            
            <div>
              <label className="text-sm font-medium">Weight (0-1)</label>
              <Input
                type="number"
                min="0"
                max="1"
                step="0.1"
                value={node.data?.weight || 1.0}
                onChange={(e) => 
                  updateNode(node.id, { 
                    data: { ...node.data, weight: parseFloat(e.target.value) } 
                  })
                }
              />
            </div>
          </div>
        )}
      </Card>
    )
  }

  const TemplateSelector = () => (
    <Card className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Rule Templates</h3>
        <Button 
          variant="outline" 
          onClick={() => setShowTemplates(false)}
        >
          Close
        </Button>
      </div>
      
      <div className="grid gap-3">
        {RULE_TEMPLATES.map((template) => (
          <Card 
            key={template.id} 
            className="p-3 cursor-pointer hover:shadow-md transition-shadow"
            onClick={() => applyTemplate(template)}
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center space-x-2 mb-1">
                  <h4 className="font-medium">{template.name}</h4>
                  {template.popular && (
                    <Badge variant="secondary" className="text-xs">Popular</Badge>
                  )}
                  <Badge variant="outline" className="text-xs">
                    {template.category}
                  </Badge>
                </div>
                <p className="text-sm text-gray-600">{template.description}</p>
              </div>
              <Copy className="h-4 w-4 text-gray-400" />
            </div>
          </Card>
        ))}
      </div>
    </Card>
  )

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Visual Rule Builder</h2>
          <p className="text-gray-600">Create complex rules with drag-and-drop interface</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" onClick={() => setShowTemplates(true)}>
            <Copy className="h-4 w-4 mr-2" />
            Templates
          </Button>
          <Button variant="outline" onClick={testRule}>
            <Play className="h-4 w-4 mr-2" />
            Test Rule
          </Button>
          <Button onClick={saveRule}>
            <Save className="h-4 w-4 mr-2" />
            Save Rule
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-4 gap-6">
        {/* Rule Tree Visualization */}
        <div className="col-span-2 space-y-4">
          <Card className="p-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold">Rule Structure</h3>
              <div className="flex space-x-2">
                <Button 
                  size="sm" 
                  onClick={() => addNode('root', 'condition')}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  Condition
                </Button>
                <Button 
                  size="sm" 
                  variant="outline"
                  onClick={() => addNode('root', 'group')}
                >
                  <GitBranch className="h-4 w-4 mr-1" />
                  Group
                </Button>
              </div>
            </div>
            
            <div className="space-y-2 max-h-96 overflow-y-auto">
              <RenderNode node={ruleTree} />
            </div>
          </Card>
          
          <Card className="p-4">
            <h3 className="text-lg font-semibold mb-4">Rule Details</h3>
            <div className="space-y-3">
              <div>
                <label className="text-sm font-medium">Rule Name</label>
                <Input
                  value={ruleName}
                  onChange={(e) => setRuleName(e.target.value)}
                  placeholder="Enter rule name"
                />
              </div>
              <div>
                <label className="text-sm font-medium">Description</label>
                <Input
                  value={ruleDescription}
                  onChange={(e) => setRuleDescription(e.target.value)}
                  placeholder="Describe this rule"
                />
              </div>
            </div>
          </Card>
        </div>

        {/* Node Editor */}
        <div className="col-span-2 space-y-4">
          {selectedNode ? (
            <NodeEditor />
          ) : (
            <Card className="p-8 text-center">
              <Target className="h-12 w-12 mx-auto text-gray-400 mb-4" />
              <h3 className="text-lg font-medium text-gray-600 mb-2">
                Select a Node to Edit
              </h3>
              <p className="text-sm text-gray-500">
                Click on any condition or group in the rule tree to configure its properties
              </p>
            </Card>
          )}
          
          {showTemplates && <TemplateSelector />}
        </div>
      </div>
    </div>
  )
}