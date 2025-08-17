"use client";

import React, { useState, useMemo } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@epsx/ui';
import { 
  ChevronRight, 
  ChevronDown, 
  Folder, 
  FolderOpen, 
  Shield, 
  Search,
  Filter,
  Check,
  X,
  Plus,
  Minus
} from 'lucide-react';

interface PermissionNode {
  id: string;
  name: string;
  type: 'resource' | 'action' | 'permission';
  children?: PermissionNode[];
  granted: boolean;
  inherited: boolean;
  source?: string;
  description?: string;
  riskLevel: 'low' | 'medium' | 'high';
  category: string;
}

interface InteractivePermissionTreeViewProps {
  permissions: PermissionNode[];
  onPermissionChange?: (nodeId: string, granted: boolean) => void;
  onBulkPermissionChange?: (nodeIds: string[], granted: boolean) => void;
  className?: string;
  readonly?: boolean;
  showSearch?: boolean;
  showBulkActions?: boolean;
  highlightChanges?: boolean;
}

interface TreeNodeProps {
  node: PermissionNode;
  level: number;
  searchTerm: string;
  expandedNodes: Set<string>;
  selectedNodes: Set<string>;
  onToggleExpand: (nodeId: string) => void;
  onToggleSelect: (nodeId: string) => void;
  onPermissionChange?: (nodeId: string, granted: boolean) => void;
  readonly: boolean;
  highlightChanges: boolean;
  pendingChanges: Map<string, boolean>;
}

function TreeNode({
  node,
  level,
  searchTerm,
  expandedNodes,
  selectedNodes,
  onToggleExpand,
  onToggleSelect,
  onPermissionChange,
  readonly,
  highlightChanges,
  pendingChanges
}: TreeNodeProps) {
  const isExpanded = expandedNodes.has(node.id);
  const isSelected = selectedNodes.has(node.id);
  const hasChildren = node.children && node.children.length > 0;
  const hasPendingChange = pendingChanges.has(node.id);
  const pendingValue = pendingChanges.get(node.id);
  const effectiveGranted = hasPendingChange ? pendingValue! : node.granted;

  const shouldShow = useMemo(() => {
    if (!searchTerm) return true;
    
    const searchLower = searchTerm.toLowerCase();
    return (
      node.name.toLowerCase().includes(searchLower) ||
      node.description?.toLowerCase().includes(searchLower) ||
      node.category.toLowerCase().includes(searchLower)
    );
  }, [node, searchTerm]);

  const handlePermissionToggle = () => {
    if (readonly || node.inherited) return;
    onPermissionChange?.(node.id, !effectiveGranted);
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'low': return 'text-green-600 bg-green-100';
      case 'medium': return 'text-orange-600 bg-orange-100';
      case 'high': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getPermissionIcon = (granted: boolean, inherited: boolean, pending?: boolean) => {
    if (pending !== undefined) {
      return pending ? 
        <Check className="h-4 w-4 text-green-500" /> : 
        <X className="h-4 w-4 text-red-500" />;
    }
    
    if (inherited) {
      return granted ? 
        <Check className="h-4 w-4 text-blue-500" /> : 
        <X className="h-4 w-4 text-gray-400" />;
    }
    
    return granted ? 
      <Check className="h-4 w-4 text-green-500" /> : 
      <X className="h-4 w-4 text-red-500" />;
  };

  if (!shouldShow && !hasChildren) return null;

  return (
    <div>
      <div 
        className={`flex items-center py-2 px-2 hover:bg-gray-50 rounded-lg group ${
          isSelected ? 'bg-blue-50 border border-blue-200' : ''
        } ${hasPendingChange && highlightChanges ? 'bg-yellow-50 border-l-4 border-yellow-400' : ''}`}
        style={{ marginLeft: `${level * 20}px` }}
      >
        {/* Expand/Collapse */}
        <div className="w-6 h-6 flex items-center justify-center">
          {hasChildren ? (
            <button 
              onClick={() => onToggleExpand(node.id)}
              className="hover:bg-gray-200 rounded p-1"
            >
              {isExpanded ? 
                <ChevronDown className="h-4 w-4" /> : 
                <ChevronRight className="h-4 w-4" />
              }
            </button>
          ) : (
            <div className="w-4 h-4" />
          )}
        </div>

        {/* Selection checkbox */}
        <div className="w-6 h-6 flex items-center justify-center">
          <input
            type="checkbox"
            checked={isSelected}
            onChange={() => onToggleSelect(node.id)}
            className="rounded border-gray-300"
          />
        </div>

        {/* Icon */}
        <div className="w-6 h-6 flex items-center justify-center mr-2">
          {node.type === 'resource' ? (
            hasChildren ? (
              isExpanded ? <FolderOpen className="h-4 w-4 text-blue-500" /> : <Folder className="h-4 w-4 text-blue-500" />
            ) : (
              <Shield className="h-4 w-4 text-purple-500" />
            )
          ) : (
            <Shield className="h-4 w-4 text-green-500" />
          )}
        </div>

        {/* Permission status */}
        <div className="w-6 h-6 flex items-center justify-center mr-3">
          <button
            onClick={handlePermissionToggle}
            disabled={readonly || node.inherited}
            className={`rounded p-1 transition-colors ${
              !readonly && !node.inherited ? 'hover:bg-gray-200' : 'cursor-not-allowed'
            }`}
          >
            {getPermissionIcon(effectiveGranted, node.inherited, hasPendingChange ? pendingValue : undefined)}
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className={`font-medium ${
              hasPendingChange && highlightChanges ? 'text-yellow-800' : 'text-gray-900'
            }`}>
              {node.name}
            </span>
            
            <Badge className={getRiskColor(node.riskLevel)} variant="secondary">
              {node.riskLevel}
            </Badge>
            
            <Badge variant="outline" className="text-xs">
              {node.category}
            </Badge>
            
            {node.inherited && (
              <Badge variant="secondary" className="text-xs text-blue-600 bg-blue-100">
                inherited from {node.source}
              </Badge>
            )}
            
            {hasPendingChange && highlightChanges && (
              <Badge variant="secondary" className="text-xs text-yellow-600 bg-yellow-100">
                pending change
              </Badge>
            )}
          </div>
          
          {node.description && (
            <p className="text-sm text-gray-600 mt-1 truncate">
              {node.description}
            </p>
          )}
        </div>

        {/* Quick actions */}
        <div className="opacity-0 group-hover:opacity-100 transition-opacity">
          {!readonly && !node.inherited && (
            <div className="flex gap-1">
              <Button
                size="sm"
                variant="ghost"
                onClick={() => onPermissionChange?.(node.id, true)}
                className="h-6 w-6 p-0"
              >
                <Plus className="h-3 w-3" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => onPermissionChange?.(node.id, false)}
                className="h-6 w-6 p-0"
              >
                <Minus className="h-3 w-3" />
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <div>
          {node.children!.map(child => (
            <TreeNode
              key={child.id}
              node={child}
              level={level + 1}
              searchTerm={searchTerm}
              expandedNodes={expandedNodes}
              selectedNodes={selectedNodes}
              onToggleExpand={onToggleExpand}
              onToggleSelect={onToggleSelect}
              onPermissionChange={onPermissionChange}
              readonly={readonly}
              highlightChanges={highlightChanges}
              pendingChanges={pendingChanges}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function InteractivePermissionTreeView({
  permissions,
  onPermissionChange,
  onBulkPermissionChange,
  className = '',
  readonly = false,
  showSearch = true,
  showBulkActions = true,
  highlightChanges = true
}: InteractivePermissionTreeViewProps) {
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [selectedNodes, setSelectedNodes] = useState<Set<string>>(new Set());
  const [pendingChanges, setPendingChanges] = useState<Map<string, boolean>>(new Map());

  const handleToggleExpand = (nodeId: string) => {
    setExpandedNodes(prev => {
      const newSet = new Set(prev);
      if (newSet.has(nodeId)) {
        newSet.delete(nodeId);
      } else {
        newSet.add(nodeId);
      }
      return newSet;
    });
  };

  const handleToggleSelect = (nodeId: string) => {
    setSelectedNodes(prev => {
      const newSet = new Set(prev);
      if (newSet.has(nodeId)) {
        newSet.delete(nodeId);
      } else {
        newSet.add(nodeId);
      }
      return newSet;
    });
  };

  const handlePermissionChange = (nodeId: string, granted: boolean) => {
    setPendingChanges(prev => {
      const newMap = new Map(prev);
      newMap.set(nodeId, granted);
      return newMap;
    });
    onPermissionChange?.(nodeId, granted);
  };

  const handleBulkGrant = () => {
    const nodeIds = Array.from(selectedNodes);
    nodeIds.forEach(nodeId => {
      setPendingChanges(prev => {
        const newMap = new Map(prev);
        newMap.set(nodeId, true);
        return newMap;
      });
    });
    onBulkPermissionChange?.(nodeIds, true);
    setSelectedNodes(new Set());
  };

  const handleBulkRevoke = () => {
    const nodeIds = Array.from(selectedNodes);
    nodeIds.forEach(nodeId => {
      setPendingChanges(prev => {
        const newMap = new Map(prev);
        newMap.set(nodeId, false);
        return newMap;
      });
    });
    onBulkPermissionChange?.(nodeIds, false);
    setSelectedNodes(new Set());
  };

  const expandAll = () => {
    const getAllNodeIds = (nodes: PermissionNode[]): string[] => {
      const ids: string[] = [];
      nodes.forEach(node => {
        ids.push(node.id);
        if (node.children) {
          ids.push(...getAllNodeIds(node.children));
        }
      });
      return ids;
    };
    setExpandedNodes(new Set(getAllNodeIds(permissions)));
  };

  const collapseAll = () => {
    setExpandedNodes(new Set());
  };

  const clearSelection = () => {
    setSelectedNodes(new Set());
  };

  const selectAll = () => {
    const getAllNodeIds = (nodes: PermissionNode[]): string[] => {
      const ids: string[] = [];
      nodes.forEach(node => {
        ids.push(node.id);
        if (node.children) {
          ids.push(...getAllNodeIds(node.children));
        }
      });
      return ids;
    };
    setSelectedNodes(new Set(getAllNodeIds(permissions)));
  };

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Controls */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Permission Tree View
            </CardTitle>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={expandAll}>
                Expand All
              </Button>
              <Button variant="outline" size="sm" onClick={collapseAll}>
                Collapse All
              </Button>
            </div>
          </div>
        </CardHeader>
        
        {(showSearch || showBulkActions) && (
          <CardContent className="pt-0">
            <div className="flex gap-4 items-center">
              {showSearch && (
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <Input
                    placeholder="Search permissions..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-10"
                  />
                </div>
              )}
              
              {showBulkActions && (
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={selectedNodes.size > 0 ? clearSelection : selectAll}
                  >
                    {selectedNodes.size > 0 ? 'Clear Selection' : 'Select All'}
                    {selectedNodes.size > 0 && (
                      <Badge variant="secondary" className="ml-2">
                        {selectedNodes.size}
                      </Badge>
                    )}
                  </Button>
                  
                  {!readonly && selectedNodes.size > 0 && (
                    <>
                      <Button
                        size="sm"
                        onClick={handleBulkGrant}
                        className="bg-green-600 hover:bg-green-700"
                      >
                        Grant Selected
                      </Button>
                      <Button
                        size="sm"
                        variant="destructive"
                        onClick={handleBulkRevoke}
                      >
                        Revoke Selected
                      </Button>
                    </>
                  )}
                </div>
              )}
            </div>
          </CardContent>
        )}
      </Card>

      {/* Tree */}
      <Card>
        <CardContent className="p-4">
          {pendingChanges.size > 0 && highlightChanges && (
            <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
              <div className="flex items-center gap-2">
                <Filter className="h-4 w-4 text-yellow-600" />
                <span className="text-sm font-medium text-yellow-800">
                  {pendingChanges.size} pending change{pendingChanges.size !== 1 ? 's' : ''}
                </span>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setPendingChanges(new Map())}
                  className="ml-auto"
                >
                  Clear Changes
                </Button>
              </div>
            </div>
          )}
          
          <div className="space-y-1 max-h-96 overflow-y-auto">
            {permissions.map(node => (
              <TreeNode
                key={node.id}
                node={node}
                level={0}
                searchTerm={searchTerm}
                expandedNodes={expandedNodes}
                selectedNodes={selectedNodes}
                onToggleExpand={handleToggleExpand}
                onToggleSelect={handleToggleSelect}
                onPermissionChange={handlePermissionChange}
                readonly={readonly}
                highlightChanges={highlightChanges}
                pendingChanges={pendingChanges}
              />
            ))}
          </div>

          {permissions.length === 0 && (
            <div className="text-center py-8 text-gray-500">
              <Shield className="h-12 w-12 mx-auto mb-4 text-gray-400" />
              <p>No permissions available</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}