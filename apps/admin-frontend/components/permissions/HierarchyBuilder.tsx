/**
 * Hierarchy Builder Component
 * Visual tool for building and managing permission hierarchies
 */

'use client'

import { 
  GitBranch, Plus, Trash2, Edit, Save, X, ChevronDown, ChevronRight,
  Shield, Users, Key, AlertTriangle, CheckCircle, Info
} from 'lucide-react'
import React, { useState, useCallback, useMemo } from 'react'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { 
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter 
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { useToast } from '@/components/ui/use-toast'
import { adminCardVariants, adminButtonVariants } from '@/design-system'
import { cn } from '@/lib/shared'

export interface PermissionNode {
  id: string;
  name: string;
  description?: string;
  permissions: string[];
  children: PermissionNode[];
  parent?: string;
  priority: number;
  isSystemNode: boolean;
}

interface HierarchyBuilderProps {
  initialHierarchy?: PermissionNode[];
  onSave?: (hierarchy: PermissionNode[]) => void;
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.initialHierarchy
 * @param root0.onSave
 * @param root0.className
 */
export function HierarchyBuilder({ 
  initialHierarchy = [], 
  onSave,
  className 
}: HierarchyBuilderProps) {
  const { toast } = useToast()
  const [hierarchy, setHierarchy] = useState<PermissionNode[]>(initialHierarchy)
  const [selectedNode, setSelectedNode] = useState<PermissionNode | null>(null)
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set())
  const [editingNode, setEditingNode] = useState<PermissionNode | null>(null)
  const [showNodeEditor, setShowNodeEditor] = useState(false)

  // Available permissions for assignment
  const availablePermissions = useMemo(() => [
    'admin:*:*',
    'admin:users:view',
    'admin:users:manage',
    'admin:users:create',
    'admin:users:delete',
    'admin:permissions:view',
    'admin:permissions:manage',
    'admin:analytics:view',
    'admin:analytics:manage',
    'admin:system:view',
    'admin:system:manage',
    'epsx:*:*',
    'epsx:analytics:view',
    'epsx:analytics:premium',
    'epsx:api:access',
    'epsx:data:export',
    'web3:*:*',
    'web3:wallet:connect',
    'web3:transactions:view',
    'web3:permissions:manage'
  ], [])

  const toggleNodeExpansion = useCallback((nodeId: string) => {
    setExpandedNodes(prev => {
      const newSet = new Set(prev)
      if (newSet.has(nodeId)) {
        newSet.delete(nodeId)
      } else {
        newSet.add(nodeId)
      }
      return newSet
    })
  }, [])

  const createNewNode = useCallback((): PermissionNode => ({
    id: crypto.randomUUID(),
    name: '',
    description: '',
    permissions: [],
    children: [],
    priority: 1,
    isSystemNode: false
  }), [])

  const addRootNode = useCallback(() => {
    const newNode = createNewNode()
    setEditingNode(newNode)
    setShowNodeEditor(true)
  }, [createNewNode])

  const addChildNode = useCallback((parentNode: PermissionNode) => {
    const newNode = createNewNode()
    newNode.parent = parentNode.id
    setEditingNode(newNode)
    setShowNodeEditor(true)
  }, [createNewNode])

  const editNode = useCallback((node: PermissionNode) => {
    setEditingNode({ ...node })
    setShowNodeEditor(true)
  }, [])

  const deleteNode = useCallback((nodeToDelete: PermissionNode) => {
    const deleteFromHierarchy = (nodes: PermissionNode[]): PermissionNode[] => {
      return nodes
        .filter(node => node.id !== nodeToDelete.id)
        .map(node => ({
          ...node,
          children: deleteFromHierarchy(node.children)
        }))
    }

    setHierarchy(deleteFromHierarchy(hierarchy))
    
    if (selectedNode?.id === nodeToDelete.id) {
      setSelectedNode(null)
    }
    
    toast({
      title: 'Node Deleted',
      description: `Permission node "${nodeToDelete.name}" has been deleted.`
    })
  }, [hierarchy, selectedNode, toast])

  const saveNode = useCallback((savedNode: PermissionNode) => {
    if (!savedNode.name.trim()) {
      toast({
        title: 'Invalid Node',
        description: 'Node name is required',
        variant: 'destructive'
      })
      return
    }

    const updateHierarchy = (nodes: PermissionNode[]): PermissionNode[] => {
      // If this is a new node (no existing node with this ID)
      const existingNode = findNodeInHierarchy(nodes, savedNode.id)
      if (!existingNode) {
        if (savedNode.parent) {
          // Add as child to parent
          return nodes.map(node => ({
            ...node,
            children: addChildToNode(node, savedNode)
          }))
        } else {
          // Add as root node
          return [...nodes, savedNode]
        }
      }

      // Update existing node
      return nodes.map(node => {
        if (node.id === savedNode.id) {
          return { ...savedNode }
        }
        return {
          ...node,
          children: updateHierarchy(node.children)
        }
      })
    }

    const addChildToNode = (node: PermissionNode, childNode: PermissionNode): PermissionNode[] => {
      if (node.id === childNode.parent) {
        return [...node.children, childNode]
      }
      return node.children.map(child => ({
        ...child,
        children: addChildToNode(child, childNode)
      }))
    }

    setHierarchy(updateHierarchy(hierarchy))
    setShowNodeEditor(false)
    setEditingNode(null)
    
    toast({
      title: 'Node Saved',
      description: `Permission node "${savedNode.name}" has been saved.`
    })
  }, [hierarchy, toast])

  const findNodeInHierarchy = (nodes: PermissionNode[], nodeId: string): PermissionNode | null => {
    for (const node of nodes) {
      if (node.id === nodeId) {return node}
      const found = findNodeInHierarchy(node.children, nodeId)
      if (found) {return found}
    }
    return null
  }

  const handleSaveHierarchy = useCallback(() => {
    onSave?.(hierarchy)
    toast({
      title: 'Hierarchy Saved',
      description: 'Permission hierarchy has been saved successfully.'
    })
  }, [hierarchy, onSave, toast])

  const getNodeDepth = useCallback((nodeId: string, nodes: PermissionNode[] = hierarchy, depth = 0): number => {
    for (const node of nodes) {
      if (node.id === nodeId) {return depth}
      const childDepth = getNodeDepth(nodeId, node.children, depth + 1)
      if (childDepth !== -1) {return childDepth}
    }
    return -1
  }, [hierarchy])

  const renderNode = useCallback((node: PermissionNode, depth = 0) => {
    const isExpanded = expandedNodes.has(node.id)
    const hasChildren = node.children.length > 0
    const isSelected = selectedNode?.id === node.id

    return (
      <div key={node.id} className="space-y-2">
        <Card 
          className={cn(
            adminCardVariants({ variant: 'default' }),
            'cursor-pointer border-l-4',
            isSelected ? 'border-l-blue-500 bg-blue-50' : 'border-l-gray-300',
            node.isSystemNode ? 'border-l-yellow-500' : ''
          )}
          style={{ marginLeft: `${depth * 24}px` }}
          onClick={() => setSelectedNode(node)}
        >
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 flex-1">
                {hasChildren ? (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 w-6 p-0"
                    onClick={(e) => {
                      e.stopPropagation()
                      toggleNodeExpansion(node.id)
                    }}
                  >
                    {isExpanded ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                  </Button>
                ) : (
                  <div className="w-6" />
                )}
                
                <Shield className="h-4 w-4 text-blue-600" />
                
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <h4 className="font-medium text-sm truncate">{node.name}</h4>
                    {node.isSystemNode && (
                      <Badge variant="secondary" className="text-xs">System</Badge>
                    )}
                    <Badge variant="outline" className="text-xs">
                      {node.permissions.length} perms
                    </Badge>
                  </div>
                  {node.description && (
                    <p className="text-xs text-gray-600 truncate mt-1">
                      {node.description}
                    </p>
                  )}
                </div>
              </div>

              <div className="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-8 w-8 p-0"
                  onClick={(e) => {
                    e.stopPropagation()
                    addChildNode(node)
                  }}
                >
                  <Plus className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-8 w-8 p-0"
                  onClick={(e) => {
                    e.stopPropagation()
                    editNode(node)
                  }}
                >
                  <Edit className="h-4 w-4" />
                </Button>
                {!node.isSystemNode && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0 text-red-600 hover:text-red-700"
                    onClick={(e) => {
                      e.stopPropagation()
                      deleteNode(node)
                    }}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        {isExpanded && hasChildren && (
          <div className="space-y-2">
            {node.children.map(child => renderNode(child, depth + 1))}
          </div>
        )}
      </div>
    )
  }, [expandedNodes, selectedNode, toggleNodeExpansion, addChildNode, editNode, deleteNode])

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">Permission Hierarchy</h1>
          <p className="text-sm text-gray-600 mt-1">
            Build and manage permission inheritance structures
          </p>
        </div>
        <div className="flex gap-2">
          <Button 
            onClick={addRootNode}
            className={adminButtonVariants({ variant: 'secondary', size: 'sm' })}
          >
            <Plus className="h-4 w-4 mr-2" />
            Add Root Node
          </Button>
          <Button 
            onClick={handleSaveHierarchy}
            className={adminButtonVariants({ variant: 'primary', size: 'sm' })}
          >
            <Save className="h-4 w-4 mr-2" />
            Save Hierarchy
          </Button>
        </div>
      </div>

      {/* Hierarchy Tree */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-4">
          <Card className={adminCardVariants({ variant: 'default' })}>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <GitBranch className="h-5 w-5" />
                Hierarchy Tree
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {hierarchy.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  <GitBranch className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>No permission nodes created yet.</p>
                  <p className="text-sm">Click "Add Root Node" to get started.</p>
                </div>
              ) : (
                hierarchy.map(node => renderNode(node))
              )}
            </CardContent>
          </Card>
        </div>

        {/* Node Details Panel */}
        <div className="space-y-4">
          <Card className={adminCardVariants({ variant: 'default' })}>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="h-5 w-5" />
                Node Details
              </CardTitle>
            </CardHeader>
            <CardContent>
              {selectedNode ? (
                <div className="space-y-4">
                  <div>
                    <Label className="text-sm font-medium">Name</Label>
                    <p className="text-sm text-gray-900">{selectedNode.name}</p>
                  </div>
                  
                  {selectedNode.description && (
                    <div>
                      <Label className="text-sm font-medium">Description</Label>
                      <p className="text-sm text-gray-600">{selectedNode.description}</p>
                    </div>
                  )}
                  
                  <div>
                    <Label className="text-sm font-medium">Priority</Label>
                    <p className="text-sm text-gray-900">{selectedNode.priority}</p>
                  </div>
                  
                  <div>
                    <Label className="text-sm font-medium">Permissions ({selectedNode.permissions.length})</Label>
                    <div className="space-y-1 mt-2">
                      {selectedNode.permissions.map((permission, index) => (
                        <Badge key={index} variant="outline" className="text-xs">
                          {permission}
                        </Badge>
                      ))}
                      {selectedNode.permissions.length === 0 && (
                        <p className="text-xs text-gray-500">No permissions assigned</p>
                      )}
                    </div>
                  </div>
                  
                  <div>
                    <Label className="text-sm font-medium">Children</Label>
                    <p className="text-sm text-gray-900">{selectedNode.children.length} child nodes</p>
                  </div>
                </div>
              ) : (
                <p className="text-sm text-gray-500">Select a node to view details</p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Node Editor Dialog */}
      <Dialog open={showNodeEditor} onOpenChange={setShowNodeEditor}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>
              {editingNode?.name ? 'Edit Permission Node' : 'Create Permission Node'}
            </DialogTitle>
          </DialogHeader>
          {editingNode && (
            <NodeEditor
              node={editingNode}
              availablePermissions={availablePermissions}
              onSave={saveNode}
              onCancel={() => {
                setShowNodeEditor(false)
                setEditingNode(null)
              }}
            />
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}

// Node Editor Component
interface NodeEditorProps {
  node: PermissionNode;
  availablePermissions: string[];
  onSave: (node: PermissionNode) => void;
  onCancel: () => void;
}

function NodeEditor({ node, availablePermissions, onSave, onCancel }: NodeEditorProps) {
  const [formData, setFormData] = useState(node)
  const [selectedPermission, setSelectedPermission] = useState('')

  const addPermission = useCallback(() => {
    if (selectedPermission && !formData.permissions.includes(selectedPermission)) {
      setFormData(prev => ({
        ...prev,
        permissions: [...prev.permissions, selectedPermission]
      }))
      setSelectedPermission('')
    }
  }, [selectedPermission, formData.permissions])

  const removePermission = useCallback((permission: string) => {
    setFormData(prev => ({
      ...prev,
      permissions: prev.permissions.filter(p => p !== permission)
    }))
  }, [])

  const handleSave = useCallback(() => {
    if (!formData.name.trim()) {return}
    onSave(formData)
  }, [formData, onSave])

  return (
    <div className="space-y-4">
      <div>
        <Label htmlFor="name">Node Name *</Label>
        <Input
          id="name"
          value={formData.name}
          onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
          placeholder="Enter node name"
        />
      </div>

      <div>
        <Label htmlFor="description">Description</Label>
        <Textarea
          id="description"
          value={formData.description || ''}
          onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
          placeholder="Optional description"
          rows={3}
        />
      </div>

      <div>
        <Label htmlFor="priority">Priority Level</Label>
        <Select
          value={formData.priority.toString()}
          onValueChange={(value) => setFormData(prev => ({ ...prev, priority: parseInt(value) }))}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {[...Array(10)].map((_, i) => (
              <SelectItem key={i + 1} value={(i + 1).toString()}>
                {i + 1}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div>
        <Label>Permissions</Label>
        <div className="flex gap-2 mt-2">
          <Select value={selectedPermission} onValueChange={setSelectedPermission}>
            <SelectTrigger className="flex-1">
              <SelectValue placeholder="Select permission to add" />
            </SelectTrigger>
            <SelectContent>
              {availablePermissions
                .filter(p => !formData.permissions.includes(p))
                .map(permission => (
                  <SelectItem key={permission} value={permission}>
                    {permission}
                  </SelectItem>
                ))}
            </SelectContent>
          </Select>
          <Button 
            onClick={addPermission} 
            disabled={!selectedPermission}
            size="sm"
          >
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        
        <div className="space-y-1 mt-3">
          {formData.permissions.map((permission, index) => (
            <div key={index} className="flex items-center justify-between p-2 bg-gray-50 rounded">
              <span className="text-sm">{permission}</span>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0 text-red-600"
                onClick={() => removePermission(permission)}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          ))}
          {formData.permissions.length === 0 && (
            <p className="text-sm text-gray-500">No permissions assigned</p>
          )}
        </div>
      </div>

      <DialogFooter>
        <Button variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={handleSave} disabled={!formData.name.trim()}>
          Save Node
        </Button>
      </DialogFooter>
    </div>
  )
}

export default HierarchyBuilder