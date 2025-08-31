'use client'

import { useState, useCallback, useMemo, useRef, useEffect } from 'react'
import { 
  Plus,
  Minus,
  Copy,
  Trash2,
  Settings,
  Play,
  Pause,
  RotateCcw,
  Save,
  Download,
  Upload,
  Eye,
  Code,
  Layers,
  Link,
  Unlink,
  ChevronDown,
  ChevronRight,
  Clock,
  User,
  Shield,
  Building,
  Tag,
  Filter,
  Zap,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Target,
  Workflow,
  GitBranch,
  Database,
  Calendar
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Switch } from '@/components/ui/switch'
import { Slider } from '@/components/ui/slider'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { useToast } from '@/components/ui/use-toast'
import { format, addMinutes, addHours, addDays } from 'date-fns'

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

interface PermissionNode {
  id: string
  type: 'permission' | 'group' | 'condition'
  operator?: 'AND' | 'OR'
  platform?: string
  resource?: string
  action?: string
  duration?: number // minutes
  timestamp?: number // unix timestamp
  conditions?: PermissionCondition[]
  children?: PermissionNode[]
  isExpanded?: boolean
  position?: { x: number; y: number }
  metadata?: Record<string, any>
}

interface PermissionCondition {
  id: string
  type: 'time_range' | 'user_role' | 'platform_access' | 'usage_limit' | 'ip_restriction' | 'custom'
  operator: 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'contains' | 'in_range'
  field: string
  value: any
  isActive: boolean
}

interface ValidationResult {
  isValid: boolean
  errors: string[]
  warnings: string[]
  suggestions: string[]
}

interface TestProfile {
  id: string
  name: string
  email: string
  role: string
  platform: string[]
  currentTime: Date
  metadata: Record<string, any>
}

interface PermissionBuilderProps {
  className?: string
  onSave?: (permission: PermissionNode) => void
  onTest?: (permission: PermissionNode, profile: TestProfile) => Promise<boolean>
}

// ============================================================================
// MOCK DATA
// ============================================================================

const PLATFORMS = ['epsx', 'admin', 'epsx-pay', 'epsx-token'] as const
const COMMON_RESOURCES = ['analytics', 'users', 'dashboard', 'reports', 'billing', 'rankings', 'realtime'] as const
const COMMON_ACTIONS = ['view', 'create', 'update', 'delete', 'manage', 'export', 'configure'] as const

const DURATION_PRESETS = [
  { label: '15 minutes', minutes: 15 },
  { label: '1 hour', minutes: 60 },
  { label: '4 hours', minutes: 240 },
  { label: '8 hours', minutes: 480 },
  { label: '1 day', minutes: 1440 },
  { label: '3 days', minutes: 4320 },
  { label: '1 week', minutes: 10080 }
] as const

const TEST_PROFILES: TestProfile[] = [
  {
    id: 'profile-1',
    name: 'John Doe',
    email: 'john@example.com',
    role: 'admin',
    platform: ['epsx', 'admin'],
    currentTime: new Date(),
    metadata: { department: 'IT', level: 'senior' }
  },
  {
    id: 'profile-2',
    name: 'Jane Smith',
    email: 'jane@example.com',
    role: 'analyst',
    platform: ['epsx'],
    currentTime: new Date(),
    metadata: { department: 'Analytics', level: 'junior' }
  }
]

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function PermissionBuilder({ className = '', onSave, onTest }: PermissionBuilderProps) {
  const { toast } = useToast()
  
  // State management
  const [rootNode, setRootNode] = useState<PermissionNode>({
    id: 'root',
    type: 'group',
    operator: 'AND',
    children: [],
    isExpanded: true
  })
  
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
  const [validation, setValidation] = useState<ValidationResult>({
    isValid: true,
    errors: [],
    warnings: [],
    suggestions: []
  })
  
  const [showTestDialog, setShowTestDialog] = useState(false)
  const [selectedTestProfile, setSelectedTestProfile] = useState<TestProfile | null>(null)
  const [testResult, setTestResult] = useState<boolean | null>(null)
  const [isBuilderMode, setIsBuilderMode] = useState(true)
  const [showAdvancedOptions, setShowAdvancedOptions] = useState(false)

  // Validation
  const validatePermissionTree = useCallback((node: PermissionNode): ValidationResult => {
    const errors: string[] = []
    const warnings: string[] = []
    const suggestions: string[] = []

    const validateNode = (node: PermissionNode, path: string = '') => {
      if (node.type === 'permission') {
        if (!node.platform) errors.push(`${path}: Platform is required`)
        if (!node.resource) errors.push(`${path}: Resource is required`)
        if (!node.action) errors.push(`${path}: Action is required`)
        
        if (node.duration && node.duration < 5) {
          warnings.push(`${path}: Duration less than 5 minutes might be too short`)
        }
        
        if (node.duration && node.duration > 10080) {
          warnings.push(`${path}: Duration more than 1 week might be too long`)
        }
      }
      
      if (node.type === 'group' && (!node.children || node.children.length === 0)) {
        warnings.push(`${path}: Empty group detected`)
      }
      
      if (node.children) {
        node.children.forEach((child, index) => {
          validateNode(child, `${path}.${index}`)
        })
      }
    }

    validateNode(node)

    // Add suggestions
    if (node.children && node.children.length === 1) {
      suggestions.push('Consider if a single permission needs to be in a group')
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      suggestions
    }
  }, [])

  // Update validation when tree changes
  useEffect(() => {
    const result = validatePermissionTree(rootNode)
    setValidation(result)
  }, [rootNode, validatePermissionTree])

  // Tree manipulation functions
  const addNode = useCallback((parentId: string, nodeType: PermissionNode['type']) => {
    const newNode: PermissionNode = {
      id: `node-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: nodeType,
      ...(nodeType === 'permission' ? {
        platform: 'epsx',
        resource: '',
        action: '',
        duration: 240
      } : nodeType === 'group' ? {
        operator: 'AND',
        children: [],
        isExpanded: true
      } : {
        conditions: []
      })
    }

    const updateTree = (node: PermissionNode): PermissionNode => {
      if (node.id === parentId) {
        return {
          ...node,
          children: [...(node.children || []), newNode]
        }
      }
      
      if (node.children) {
        return {
          ...node,
          children: node.children.map(updateTree)
        }
      }
      
      return node
    }

    setRootNode(updateTree(rootNode))
    setSelectedNodeId(newNode.id)
  }, [rootNode])

  const updateNode = useCallback((nodeId: string, updates: Partial<PermissionNode>) => {
    const updateTree = (node: PermissionNode): PermissionNode => {
      if (node.id === nodeId) {
        return { ...node, ...updates }
      }
      
      if (node.children) {
        return {
          ...node,
          children: node.children.map(updateTree)
        }
      }
      
      return node
    }

    setRootNode(updateTree(rootNode))
  }, [rootNode])

  const deleteNode = useCallback((nodeId: string) => {
    const removeFromTree = (node: PermissionNode): PermissionNode => {
      if (node.children) {
        return {
          ...node,
          children: node.children
            .filter(child => child.id !== nodeId)
            .map(removeFromTree)
        }
      }
      return node
    }

    setRootNode(removeFromTree(rootNode))
    if (selectedNodeId === nodeId) {
      setSelectedNodeId(null)
    }
  }, [rootNode, selectedNodeId])

  const duplicateNode = useCallback((nodeId: string) => {
    const findAndDuplicate = (node: PermissionNode): PermissionNode | null => {
      if (node.id === nodeId) {
        return {
          ...node,
          id: `node-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          children: node.children?.map(child => ({ ...child, id: `node-${Date.now()}-${Math.random().toString(36).substr(2, 9)}` }))
        }
      }
      
      if (node.children) {
        for (const child of node.children) {
          const result = findAndDuplicate(child)
          if (result) return result
        }
      }
      
      return null
    }

    const duplicate = findAndDuplicate(rootNode)
    if (duplicate) {
      // Add duplicate as sibling
      const addSibling = (node: PermissionNode): PermissionNode => {
        if (node.children) {
          const childIndex = node.children.findIndex(child => child.id === nodeId)
          if (childIndex !== -1) {
            const newChildren = [...node.children]
            newChildren.splice(childIndex + 1, 0, duplicate)
            return { ...node, children: newChildren }
          }
          
          return {
            ...node,
            children: node.children.map(addSibling)
          }
        }
        return node
      }

      setRootNode(addSibling(rootNode))
    }
  }, [rootNode])

  // Generate permission string
  const generatePermissionString = useCallback((node: PermissionNode): string => {
    if (node.type === 'permission' && node.platform && node.resource && node.action) {
      let permission = `${node.platform}:${node.resource}:${node.action}`
      
      if (node.timestamp) {
        permission += `:${node.timestamp}`
      } else if (node.duration) {
        const expiryTimestamp = Math.floor((Date.now() + node.duration * 60 * 1000) / 1000)
        permission += `:${expiryTimestamp}`
      }
      
      return permission
    }
    
    if (node.type === 'group' && node.children) {
      const childPermissions = node.children
        .map(generatePermissionString)
        .filter(p => p.length > 0)
      
      if (childPermissions.length === 0) return ''
      if (childPermissions.length === 1) return childPermissions[0]
      
      const operator = node.operator === 'OR' ? ' OR ' : ' AND '
      return `(${childPermissions.join(operator)})`
    }
    
    return ''
  }, [])

  // Test permission against profile
  const testPermission = useCallback(async (profile: TestProfile) => {
    if (!onTest) {
      // Simulate test result
      const result = Math.random() > 0.3
      setTestResult(result)
      
      toast({
        title: 'Test Complete',
        description: `Permission ${result ? 'granted' : 'denied'} for ${profile.name}`,
        variant: result ? 'default' : 'destructive'
      })
      return
    }
    
    try {
      const result = await onTest(rootNode, profile)
      setTestResult(result)
      
      toast({
        title: 'Test Complete',
        description: `Permission ${result ? 'granted' : 'denied'} for ${profile.name}`,
        variant: result ? 'default' : 'destructive'
      })
    } catch (error) {
      toast({
        title: 'Test Error',
        description: 'Failed to test permission',
        variant: 'destructive'
      })
    }
  }, [rootNode, onTest, toast])

  const selectedNode = useMemo(() => {
    const findNode = (node: PermissionNode): PermissionNode | null => {
      if (node.id === selectedNodeId) return node
      if (node.children) {
        for (const child of node.children) {
          const result = findNode(child)
          if (result) return result
        }
      }
      return null
    }
    
    return findNode(rootNode)
  }, [rootNode, selectedNodeId])

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-xl font-semibold">Permission Builder</h3>
          <p className="text-muted-foreground">
            Build complex permissions with embedded timestamps and conditional logic
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Switch
            id="builder-mode"
            checked={isBuilderMode}
            onCheckedChange={setIsBuilderMode}
          />
          <Label htmlFor="builder-mode" className="text-sm">Visual Builder</Label>
          
          <Button variant="outline" size="sm" onClick={() => setShowTestDialog(true)}>
            <Play className="h-4 w-4 mr-2" />
            Test
          </Button>
          
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
          
          <Button size="sm" onClick={() => onSave?.(rootNode)}>
            <Save className="h-4 w-4 mr-2" />
            Save
          </Button>
        </div>
      </div>

      {/* Validation Status */}
      {(!validation.isValid || validation.warnings.length > 0) && (
        <Alert variant={validation.isValid ? 'default' : 'destructive'}>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            <div className="space-y-2">
              {validation.errors.map((error, index) => (
                <div key={index} className="text-red-600">• {error}</div>
              ))}
              {validation.warnings.map((warning, index) => (
                <div key={index} className="text-yellow-600">⚠ {warning}</div>
              ))}
              {validation.suggestions.map((suggestion, index) => (
                <div key={index} className="text-blue-600">💡 {suggestion}</div>
              ))}
            </div>
          </AlertDescription>
        </Alert>
      )}

      {/* Main Interface */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Tree Builder */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Workflow className="h-5 w-5" />
                Permission Tree
              </CardTitle>
              <CardDescription>
                Build your permission structure using groups and conditions
              </CardDescription>
            </CardHeader>
            <CardContent>
              {isBuilderMode ? (
                <div className="space-y-4">
                  <div className="flex gap-2 mb-4">
                    <Button 
                      size="sm" 
                      onClick={() => addNode('root', 'permission')}
                    >
                      <Plus className="h-4 w-4 mr-2" />
                      Add Permission
                    </Button>
                    <Button 
                      size="sm" 
                      variant="outline"
                      onClick={() => addNode('root', 'group')}
                    >
                      <GitBranch className="h-4 w-4 mr-2" />
                      Add Group
                    </Button>
                    <Button 
                      size="sm" 
                      variant="outline"
                      onClick={() => addNode('root', 'condition')}
                    >
                      <Filter className="h-4 w-4 mr-2" />
                      Add Condition
                    </Button>
                  </div>
                  
                  <ScrollArea className="h-[500px]">
                    <PermissionTreeNode
                      node={rootNode}
                      selectedNodeId={selectedNodeId}
                      onNodeSelect={setSelectedNodeId}
                      onNodeUpdate={updateNode}
                      onNodeDelete={deleteNode}
                      onNodeDuplicate={duplicateNode}
                      onAddChild={addNode}
                      level={0}
                    />
                  </ScrollArea>
                </div>
              ) : (
                <div className="space-y-4">
                  <Label>Raw Permission String</Label>
                  <Textarea
                    value={generatePermissionString(rootNode)}
                    readOnly
                    rows={8}
                    className="font-mono text-sm"
                  />
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Properties Panel */}
        <div className="space-y-6">
          {selectedNode && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="h-5 w-5" />
                  Properties
                </CardTitle>
                <CardDescription>
                  Configure the selected {selectedNode.type}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <PermissionNodeProperties
                  node={selectedNode}
                  onUpdate={(updates) => updateNode(selectedNode.id, updates)}
                />
              </CardContent>
            </Card>
          )}

          {/* Permission Preview */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Eye className="h-5 w-5" />
                Generated Permission
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="font-mono text-sm p-3 bg-gray-100 dark:bg-gray-800 rounded">
                {generatePermissionString(rootNode) || 'No permission configured'}
              </div>
            </CardContent>
          </Card>

          {/* Quick Actions */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Zap className="h-5 w-5" />
                Quick Actions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <Button 
                size="sm" 
                variant="outline" 
                className="w-full"
                onClick={() => {
                  setRootNode({
                    id: 'root',
                    type: 'group',
                    operator: 'AND',
                    children: [],
                    isExpanded: true
                  })
                  setSelectedNodeId(null)
                }}
              >
                <RotateCcw className="h-4 w-4 mr-2" />
                Reset Tree
              </Button>
              
              <Button 
                size="sm" 
                variant="outline" 
                className="w-full"
                onClick={() => setShowAdvancedOptions(!showAdvancedOptions)}
              >
                <Code className="h-4 w-4 mr-2" />
                {showAdvancedOptions ? 'Hide' : 'Show'} Advanced
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Test Dialog */}
      <Dialog open={showTestDialog} onOpenChange={setShowTestDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Test Permission</DialogTitle>
            <DialogDescription>
              Test your permission against different user profiles
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4">
            <div>
              <Label>Test Profile</Label>
              <Select
                value={selectedTestProfile?.id || ''}
                onValueChange={(value) => {
                  const profile = TEST_PROFILES.find(p => p.id === value)
                  setSelectedTestProfile(profile || null)
                }}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a test profile" />
                </SelectTrigger>
                <SelectContent>
                  {TEST_PROFILES.map((profile) => (
                    <SelectItem key={profile.id} value={profile.id}>
                      {profile.name} ({profile.role})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {selectedTestProfile && (
              <div className="p-3 border rounded space-y-2">
                <div><strong>Name:</strong> {selectedTestProfile.name}</div>
                <div><strong>Role:</strong> {selectedTestProfile.role}</div>
                <div><strong>Platforms:</strong> {selectedTestProfile.platform.join(', ')}</div>
              </div>
            )}

            {testResult !== null && (
              <Alert variant={testResult ? 'default' : 'destructive'}>
                {testResult ? (
                  <CheckCircle className="h-4 w-4" />
                ) : (
                  <XCircle className="h-4 w-4" />
                )}
                <AlertDescription>
                  Permission {testResult ? 'granted' : 'denied'} for {selectedTestProfile?.name}
                </AlertDescription>
              </Alert>
            )}

            <div className="flex gap-2">
              <Button
                onClick={() => selectedTestProfile && testPermission(selectedTestProfile)}
                disabled={!selectedTestProfile}
                className="flex-1"
              >
                <Play className="h-4 w-4 mr-2" />
                Run Test
              </Button>
              <Button variant="outline" onClick={() => setShowTestDialog(false)}>
                Close
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}

// ============================================================================
// TREE NODE COMPONENT
// ============================================================================

interface PermissionTreeNodeProps {
  node: PermissionNode
  selectedNodeId: string | null
  onNodeSelect: (id: string) => void
  onNodeUpdate: (id: string, updates: Partial<PermissionNode>) => void
  onNodeDelete: (id: string) => void
  onNodeDuplicate: (id: string) => void
  onAddChild: (parentId: string, type: PermissionNode['type']) => void
  level: number
}

function PermissionTreeNode({
  node,
  selectedNodeId,
  onNodeSelect,
  onNodeUpdate,
  onNodeDelete,
  onNodeDuplicate,
  onAddChild,
  level
}: PermissionTreeNodeProps) {
  const getNodeIcon = (type: PermissionNode['type']) => {
    switch (type) {
      case 'permission': return Shield
      case 'group': return GitBranch
      case 'condition': return Filter
      default: return Shield
    }
  }

  const getNodeColor = (type: PermissionNode['type']) => {
    switch (type) {
      case 'permission': return 'text-blue-600'
      case 'group': return 'text-green-600'
      case 'condition': return 'text-orange-600'
      default: return 'text-gray-600'
    }
  }

  const Icon = getNodeIcon(node.type)
  const isSelected = selectedNodeId === node.id
  
  return (
    <div className="space-y-2">
      <div 
        className={`
          flex items-center gap-2 p-2 rounded cursor-pointer transition-colors
          ${isSelected ? 'bg-blue-100 dark:bg-blue-900/20' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}
        `}
        style={{ marginLeft: `${level * 20}px` }}
        onClick={() => onNodeSelect(node.id)}
      >
        {node.type === 'group' && node.children && node.children.length > 0 && (
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            onClick={(e) => {
              e.stopPropagation()
              onNodeUpdate(node.id, { isExpanded: !node.isExpanded })
            }}
          >
            {node.isExpanded ? (
              <ChevronDown className="h-3 w-3" />
            ) : (
              <ChevronRight className="h-3 w-3" />
            )}
          </Button>
        )}
        
        <Icon className={`h-4 w-4 ${getNodeColor(node.type)}`} />
        
        <div className="flex-1">
          {node.type === 'permission' && (
            <span className="text-sm font-mono">
              {node.platform}:{node.resource}:{node.action}
              {node.duration && (
                <Badge variant="outline" className="ml-2">
                  {node.duration < 60 ? `${node.duration}m` :
                   node.duration < 1440 ? `${Math.round(node.duration / 60)}h` :
                   `${Math.round(node.duration / 1440)}d`}
                </Badge>
              )}
            </span>
          )}
          
          {node.type === 'group' && (
            <span className="text-sm">
              Group ({node.operator})
              {node.children && (
                <Badge variant="outline" className="ml-2">
                  {node.children.length} items
                </Badge>
              )}
            </span>
          )}
          
          {node.type === 'condition' && (
            <span className="text-sm">
              Condition
              {node.conditions && (
                <Badge variant="outline" className="ml-2">
                  {node.conditions.length} rules
                </Badge>
              )}
            </span>
          )}
        </div>
        
        {isSelected && (
          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0"
              onClick={(e) => {
                e.stopPropagation()
                onNodeDuplicate(node.id)
              }}
            >
              <Copy className="h-3 w-3" />
            </Button>
            
            {node.id !== 'root' && (
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0 text-red-600"
                onClick={(e) => {
                  e.stopPropagation()
                  onNodeDelete(node.id)
                }}
              >
                <Trash2 className="h-3 w-3" />
              </Button>
            )}
          </div>
        )}
      </div>
      
      {node.type === 'group' && node.isExpanded && node.children && (
        <div>
          {node.children.map((child) => (
            <PermissionTreeNode
              key={child.id}
              node={child}
              selectedNodeId={selectedNodeId}
              onNodeSelect={onNodeSelect}
              onNodeUpdate={onNodeUpdate}
              onNodeDelete={onNodeDelete}
              onNodeDuplicate={onNodeDuplicate}
              onAddChild={onAddChild}
              level={level + 1}
            />
          ))}
          
          {isSelected && (
            <div className="flex gap-1 mt-2" style={{ marginLeft: `${(level + 1) * 20}px` }}>
              <Button
                size="sm"
                variant="outline"
                onClick={() => onAddChild(node.id, 'permission')}
              >
                <Plus className="h-3 w-3 mr-1" />
                Permission
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => onAddChild(node.id, 'group')}
              >
                <Plus className="h-3 w-3 mr-1" />
                Group
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => onAddChild(node.id, 'condition')}
              >
                <Plus className="h-3 w-3 mr-1" />
                Condition
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// PROPERTIES PANEL COMPONENT
// ============================================================================

interface PermissionNodePropertiesProps {
  node: PermissionNode
  onUpdate: (updates: Partial<PermissionNode>) => void
}

function PermissionNodeProperties({ node, onUpdate }: PermissionNodePropertiesProps) {
  if (node.type === 'permission') {
    return (
      <div className="space-y-4">
        <div>
          <Label>Platform</Label>
          <Select 
            value={node.platform || 'epsx'} 
            onValueChange={(value) => onUpdate({ platform: value })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PLATFORMS.map((platform) => (
                <SelectItem key={platform} value={platform}>
                  {platform}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <Label>Resource</Label>
          <Input
            value={node.resource || ''}
            onChange={(e) => onUpdate({ resource: e.target.value })}
            placeholder="e.g., analytics, users"
          />
          <div className="flex flex-wrap gap-1 mt-1">
            {COMMON_RESOURCES.map((resource) => (
              <Button
                key={resource}
                variant="ghost"
                size="sm"
                className="h-6 text-xs"
                onClick={() => onUpdate({ resource })}
              >
                {resource}
              </Button>
            ))}
          </div>
        </div>

        <div>
          <Label>Action</Label>
          <Input
            value={node.action || ''}
            onChange={(e) => onUpdate({ action: e.target.value })}
            placeholder="e.g., view, manage"
          />
          <div className="flex flex-wrap gap-1 mt-1">
            {COMMON_ACTIONS.map((action) => (
              <Button
                key={action}
                variant="ghost"
                size="sm"
                className="h-6 text-xs"
                onClick={() => onUpdate({ action })}
              >
                {action}
              </Button>
            ))}
          </div>
        </div>

        <div>
          <Label>Duration: {Math.round((node.duration || 0) / 60 * 100) / 100} hours</Label>
          <Slider
            value={[node.duration || 240]}
            onValueChange={(value) => onUpdate({ duration: value[0] })}
            min={5}
            max={10080} // 1 week
            step={5}
          />
          <div className="grid grid-cols-3 gap-1 mt-2">
            {DURATION_PRESETS.slice(0, 6).map((preset) => (
              <Button
                key={preset.label}
                variant="outline"
                size="sm"
                className="text-xs"
                onClick={() => onUpdate({ duration: preset.minutes })}
              >
                {preset.label}
              </Button>
            ))}
          </div>
        </div>

        <div>
          <Label>Custom Timestamp</Label>
          <Input
            type="datetime-local"
            value={node.timestamp ? format(new Date(node.timestamp * 1000), "yyyy-MM-dd'T'HH:mm") : ''}
            onChange={(e) => {
              const timestamp = e.target.value ? Math.floor(new Date(e.target.value).getTime() / 1000) : undefined
              onUpdate({ timestamp })
            }}
          />
          <p className="text-xs text-muted-foreground mt-1">
            Leave empty to use duration-based expiry
          </p>
        </div>
      </div>
    )
  }

  if (node.type === 'group') {
    return (
      <div className="space-y-4">
        <div>
          <Label>Operator</Label>
          <Select 
            value={node.operator || 'AND'} 
            onValueChange={(value: 'AND' | 'OR') => onUpdate({ operator: value })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="AND">AND (All must match)</SelectItem>
              <SelectItem value="OR">OR (Any can match)</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="text-sm text-muted-foreground">
          <p>
            <strong>AND:</strong> All child permissions must be satisfied
          </p>
          <p>
            <strong>OR:</strong> Any child permission can be satisfied
          </p>
        </div>
      </div>
    )
  }

  if (node.type === 'condition') {
    return (
      <div className="space-y-4">
        <div>
          <Label>Conditions</Label>
          <p className="text-sm text-muted-foreground">
            Add conditions to restrict when this permission applies
          </p>
        </div>

        <Button size="sm" variant="outline">
          <Plus className="h-4 w-4 mr-2" />
          Add Condition
        </Button>
        
        {/* This would expand to show condition configuration */}
      </div>
    )
  }

  return (
    <div className="text-sm text-muted-foreground">
      Select a node to configure its properties
    </div>
  )
}