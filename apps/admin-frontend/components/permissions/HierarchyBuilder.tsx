'use client';

import React, { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/hooks/use-toast';
import { 
  FolderIcon, 
  FileIcon, 
  PlusIcon, 
  TrashIcon, 
  RefreshCwIcon,
  TreePineIcon,
  LinkIcon,
  CheckCircleIcon,
  AlertCircleIcon,
} from 'lucide-react';

interface PermissionHierarchy {
  id: string;
  parent_permission: string;
  child_permission: string;
  inheritance_type: 'automatic' | 'conditional';
  is_active: boolean;
  created_at: string;
}

interface HierarchyNode {
  permission: string;
  children: HierarchyNode[];
  parent?: string;
  type: 'folder' | 'permission';
  level: number;
}

interface HierarchyStats {
  total_hierarchies: number;
  unique_parents: number;
  unique_children: number;
  automatic_count: number;
  conditional_count: number;
}

export default function HierarchyBuilder() {
  const [hierarchies, setHierarchies] = useState<PermissionHierarchy[]>([]);
  const [hierarchyTree, setHierarchyTree] = useState<HierarchyNode[]>([]);
  const [stats, setStats] = useState<HierarchyStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newHierarchy, setNewHierarchy] = useState({
    parent_permission: '',
    child_permission: '',
    inheritance_type: 'automatic' as 'automatic' | 'conditional'
  });
  const { toast } = useToast();

  useEffect(() => {
    loadHierarchyData();
  }, []);

  const loadHierarchyData = async () => {
    try {
      setLoading(true);
      
      // Load hierarchy tree and stats in parallel
      const [treeResponse, statsResponse] = await Promise.all([
        fetch('/api/v1/admin/permissions/hierarchy/tree'),
        fetch('/api/v1/admin/permissions/hierarchy/stats')
      ]);

      if (treeResponse.ok && statsResponse.ok) {
        const treeData = await treeResponse.json();
        const statsData = await statsResponse.json();
        
        setHierarchies(treeData.hierarchies || []);
        setStats(statsData.stats);
        buildHierarchyTree(treeData.hierarchies || []);
      } else {
        throw new Error('Failed to load hierarchy data');
      }
    } catch (error) {
      console.error('Error loading hierarchy data:', error);
      toast({
        title: "Error",
        description: "Failed to load permission hierarchy data",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const buildHierarchyTree = (hierarchies: PermissionHierarchy[]) => {
    const nodes = new Map<string, HierarchyNode>();
    const roots: HierarchyNode[] = [];

    // Create all nodes
    const allPermissions = new Set<string>();
    hierarchies.forEach(h => {
      allPermissions.add(h.parent_permission);
      allPermissions.add(h.child_permission);
    });

    allPermissions.forEach(permission => {
      nodes.set(permission, {
        permission,
        children: [],
        type: permission.includes('*') ? 'folder' : 'permission',
        level: 0
      });
    });

    // Build relationships
    hierarchies.forEach(h => {
      const parent = nodes.get(h.parent_permission);
      const child = nodes.get(h.child_permission);
      
      if (parent && child) {
        parent.children.push(child);
        child.parent = h.parent_permission;
      }
    });

    // Find roots and calculate levels
    nodes.forEach(node => {
      if (!node.parent) {
        roots.push(node);
        calculateLevels(node, 0);
      }
    });

    setHierarchyTree(roots.sort((a, b) => a.permission.localeCompare(b.permission)));
  };

  const calculateLevels = (node: HierarchyNode, level: number) => {
    node.level = level;
    node.children.forEach(child => calculateLevels(child, level + 1));
  };

  const handleAddHierarchy = async () => {
    try {
      if (!newHierarchy.parent_permission || !newHierarchy.child_permission) {
        toast({
          title: "Validation Error",
          description: "Both parent and child permissions are required",
          variant: "destructive",
        });
        return;
      }

      const response = await fetch('/api/v1/admin/permissions/hierarchy', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(newHierarchy),
      });

      if (response.ok) {
        const result = await response.json();
        toast({
          title: "Success",
          description: result.message,
        });
        
        setNewHierarchy({
          parent_permission: '',
          child_permission: '',
          inheritance_type: 'automatic'
        });
        setShowAddForm(false);
        loadHierarchyData();
      } else {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create hierarchy');
      }
    } catch (error) {
      console.error('Error creating hierarchy:', error);
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to create hierarchy",
        variant: "destructive",
      });
    }
  };

  const handleDeleteHierarchy = async (hierarchyId: string, parentPerm: string, childPerm: string) => {
    try {
      const response = await fetch(`/api/v1/admin/permissions/hierarchy/${hierarchyId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        toast({
          title: "Success",
          description: `Removed hierarchy: ${parentPerm} → ${childPerm}`,
        });
        loadHierarchyData();
      } else {
        const error = await response.json();
        throw new Error(error.error || 'Failed to delete hierarchy');
      }
    } catch (error) {
      console.error('Error deleting hierarchy:', error);
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to delete hierarchy",
        variant: "destructive",
      });
    }
  };

  const renderHierarchyNode = (node: HierarchyNode): React.ReactNode => {
    const indent = node.level * 16; // Reduced indent for mobile
    const isFolder = node.type === 'folder';
    
    return (
      <div key={node.permission} className="mb-1">
        <div 
          className="flex items-center gap-2 p-2 sm:p-3 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-700 group"
          style={{ paddingLeft: `${Math.max(indent + 8, 8)}px` }}
        >
          {isFolder ? (
            <FolderIcon className="h-4 w-4 text-blue-600 flex-shrink-0" />
          ) : (
            <FileIcon className="h-4 w-4 text-gray-600 dark:text-gray-400 flex-shrink-0" />
          )}
          
          <span className={`font-mono text-xs sm:text-sm flex-1 min-w-0 truncate ${isFolder ? 'font-semibold text-blue-800 dark:text-blue-200' : 'text-gray-700 dark:text-gray-300'}`}>
            {node.permission}
          </span>
          
          {isFolder && (
            <Badge variant="secondary" className="text-xs rounded-xl flex-shrink-0 hidden sm:inline-flex">
              {node.children.length} children
            </Badge>
          )}
          
          <div className="ml-auto opacity-60 group-hover:opacity-100 flex-shrink-0">
            {/* Find hierarchy entry for delete action */}
            {node.parent && (
              <Button
                size="sm"
                variant="ghost"
                onClick={() => {
                  const hierarchy = hierarchies.find(h => 
                    h.parent_permission === node.parent && 
                    h.child_permission === node.permission
                  );
                  if (hierarchy && node.parent && node.permission) {
                    handleDeleteHierarchy(hierarchy.id, node.parent, node.permission);
                  }
                }}
                className="h-6 w-6 p-0 text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300 rounded-lg"
              >
                <TrashIcon className="h-3 w-3" />
              </Button>
            )}
          </div>
        </div>
        
        {node.children.map(child => renderHierarchyNode(child))}
      </div>
    );
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6 sm:p-8">
            <div className="flex items-center justify-center h-48">
              <RefreshCwIcon className="h-6 w-6 text-gray-400" />
              <span className="ml-2 text-gray-600">Loading hierarchy data...</span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>

      <div className="relative space-y-4 sm:space-y-6">
        {/* Page Header */}
        <div className="text-center mb-6 sm:mb-8">
          <div className="relative inline-block">
            <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
              🌳 Permission Hierarchy Builder
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
          </div>
          <p className="text-sm sm:text-base lg:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Manage permission inheritance relationships across the EPSX platform
          </p>
        </div>

        {/* Header Actions */}
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-4 sm:mb-6">
          <div className="flex items-center gap-3">
            <TreePineIcon className="h-5 w-5 sm:h-6 sm:w-6 text-blue-600" />
            <h2 className="text-lg sm:text-xl font-semibold text-gray-800 dark:text-gray-200">Hierarchy Management</h2>
          </div>
          
          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
            <Button 
              variant="outline" 
              size="sm"
              onClick={loadHierarchyData}
              disabled={loading}
              className="rounded-2xl border-2 border-blue-200 dark:border-blue-700 h-10 sm:h-auto"
            >
              <RefreshCwIcon className="h-4 w-4 mr-2" />
              Refresh
            </Button>
            
            <Button 
              size="sm"
              onClick={() => setShowAddForm(!showAddForm)}
              className="bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 rounded-2xl h-10 sm:h-auto"
            >
              <PlusIcon className="h-4 w-4 mr-2" />
              Add Hierarchy
            </Button>
          </div>
        </div>

        {/* Statistics Cards */}
        {stats && (
          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3 sm:gap-4">
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-cyan-400/20 to-teal-400/20 p-0.5">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-3 sm:p-4">
                <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-br from-blue-300/30 to-cyan-400/30 rounded-full blur-sm"></div>
                <div className="flex items-center gap-2 mb-2">
                  <LinkIcon className="h-4 w-4 text-blue-600" />
                  <div className="min-w-0 flex-1">
                    <p className="text-xs sm:text-sm text-gray-600 dark:text-gray-400 truncate">Total</p>
                    <p className="text-lg sm:text-xl lg:text-2xl font-bold text-blue-600">{stats.total_hierarchies}</p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">Hierarchies</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-teal-400/20 p-0.5">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-3 sm:p-4">
                <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-br from-green-300/30 to-emerald-400/30 rounded-full blur-sm"></div>
                <div className="flex items-center gap-2 mb-2">
                  <FolderIcon className="h-4 w-4 text-green-600" />
                  <div className="min-w-0 flex-1">
                    <p className="text-xs sm:text-sm text-gray-600 dark:text-gray-400 truncate">Parents</p>
                    <p className="text-lg sm:text-xl lg:text-2xl font-bold text-green-600">{stats.unique_parents}</p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">Permissions</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-rose-400/20 p-0.5">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-3 sm:p-4">
                <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-br from-purple-300/30 to-pink-400/30 rounded-full blur-sm"></div>
                <div className="flex items-center gap-2 mb-2">
                  <FileIcon className="h-4 w-4 text-purple-600" />
                  <div className="min-w-0 flex-1">
                    <p className="text-xs sm:text-sm text-gray-600 dark:text-gray-400 truncate">Children</p>
                    <p className="text-lg sm:text-xl lg:text-2xl font-bold text-purple-600">{stats.unique_children}</p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">Permissions</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-400/20 via-green-400/20 to-lime-400/20 p-0.5">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-3 sm:p-4">
                <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-br from-emerald-300/30 to-green-400/30 rounded-full blur-sm"></div>
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircleIcon className="h-4 w-4 text-green-600" />
                  <div className="min-w-0 flex-1">
                    <p className="text-xs sm:text-sm text-gray-600 dark:text-gray-400 truncate">Automatic</p>
                    <p className="text-lg sm:text-xl lg:text-2xl font-bold text-green-600">{stats.automatic_count}</p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">Rules</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-yellow-400/20 via-amber-400/20 to-orange-400/20 p-0.5 col-span-2 sm:col-span-1">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-3 sm:p-4">
                <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-br from-yellow-300/30 to-amber-400/30 rounded-full blur-sm"></div>
                <div className="flex items-center gap-2 mb-2">
                  <AlertCircleIcon className="h-4 w-4 text-yellow-600" />
                  <div className="min-w-0 flex-1">
                    <p className="text-xs sm:text-sm text-gray-600 dark:text-gray-400 truncate">Conditional</p>
                    <p className="text-lg sm:text-xl lg:text-2xl font-bold text-yellow-600">{stats.conditional_count}</p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">Rules</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Add Hierarchy Form */}
        {showAddForm && (
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-4 sm:p-6">
              <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full blur-sm"></div>
              
              <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 to-pink-600 bg-clip-text text-transparent mb-4">
                ➕ Add New Permission Hierarchy
              </h3>
              
              <div className="space-y-4">
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
                  <div>
                    <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">Parent Permission</label>
                    <Input
                      placeholder="e.g., epsx:trading:*"
                      value={newHierarchy.parent_permission}
                      onChange={(e) => setNewHierarchy(prev => ({
                        ...prev,
                        parent_permission: e.target.value
                      }))}
                      className="rounded-2xl border-2 border-blue-200 dark:border-blue-700 h-11"
                    />
                    <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                      Use * for wildcard permissions
                    </p>
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">Child Permission</label>
                    <Input
                      placeholder="e.g., epsx:trading:basic"
                      value={newHierarchy.child_permission}
                      onChange={(e) => setNewHierarchy(prev => ({
                        ...prev,
                        child_permission: e.target.value
                      }))}
                      className="rounded-2xl border-2 border-purple-200 dark:border-purple-700 h-11"
                    />
                    <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                      Specific permission to inherit
                    </p>
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">Inheritance Type</label>
                    <select
                      className="w-full px-3 py-2 border-2 border-orange-200 dark:border-orange-700 rounded-2xl h-11 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300"
                      value={newHierarchy.inheritance_type}
                      onChange={(e) => setNewHierarchy(prev => ({
                        ...prev,
                        inheritance_type: e.target.value as 'automatic' | 'conditional'
                      }))}
                    >
                      <option value="automatic">⚡ Automatic</option>
                      <option value="conditional">🔄 Conditional</option>
                    </select>
                    <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                      How inheritance is applied
                    </p>
                  </div>
                </div>
                
                <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 pt-4">
                  <Button 
                    onClick={handleAddHierarchy}
                    className="bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600 rounded-2xl h-11"
                  >
                    <PlusIcon className="h-4 w-4 mr-2" />
                    Create Hierarchy
                  </Button>
                  
                  <Button 
                    variant="outline" 
                    onClick={() => setShowAddForm(false)}
                    className="rounded-2xl border-2 border-gray-200 dark:border-gray-700 h-11"
                  >
                    Cancel
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Hierarchy Tree */}
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-teal-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-4 sm:p-6">
            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full blur-sm"></div>
            
            <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3 mb-4">
              <div className="flex items-center gap-2">
                <TreePineIcon className="h-5 w-5 text-green-600" />
                <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-green-600 via-emerald-600 to-teal-600 bg-clip-text text-transparent">
                  Permission Inheritance Tree
                </h3>
              </div>
              <Badge variant="outline" className="ml-auto rounded-2xl border-2 border-green-200 dark:border-green-700">
                {hierarchyTree.length} root permissions
              </Badge>
            </div>
            
            {hierarchyTree.length === 0 ? (
              <div className="text-center py-8 sm:py-12 text-gray-500 dark:text-gray-400">
                <TreePineIcon className="h-8 w-8 sm:h-12 sm:w-12 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
                <p className="text-base sm:text-lg font-medium mb-2">No permission hierarchies found</p>
                <p className="text-sm">Create your first hierarchy to see the tree structure</p>
              </div>
            ) : (
              <div className="max-h-64 sm:max-h-96 overflow-y-auto border-2 border-gray-200 dark:border-gray-700 rounded-2xl bg-gray-50 dark:bg-gray-800 p-3 sm:p-4">
                {hierarchyTree.map(node => renderHierarchyNode(node))}
              </div>
            )}
          </div>
        </div>

        {/* Quick Actions Panel */}
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-orange-400/20 via-yellow-400/20 to-amber-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-4 sm:p-6">
            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-orange-400 to-yellow-500 rounded-full blur-sm"></div>
            
            <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-orange-600 via-yellow-600 to-amber-600 bg-clip-text text-transparent mb-4">
              ⚡ Quick Actions
            </h3>
            
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4 mb-4">
              <Button 
                variant="outline" 
                disabled
                className="h-auto p-3 sm:p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-blue-400/10 to-cyan-400/10 border-2 border-blue-200 dark:border-blue-700 rounded-2xl text-gray-700 dark:text-gray-300"
              >
                <LinkIcon className="h-4 w-4 sm:h-5 sm:w-5" />
                <span className="text-xs sm:text-sm font-medium text-center">Test Inheritance</span>
              </Button>
              
              <Button 
                variant="outline" 
                disabled
                className="h-auto p-3 sm:p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-purple-400/10 to-pink-400/10 border-2 border-purple-200 dark:border-purple-700 rounded-2xl text-gray-700 dark:text-gray-300"
              >
                <RefreshCwIcon className="h-4 w-4 sm:h-5 sm:w-5" />
                <span className="text-xs sm:text-sm font-medium text-center">Bulk Assign</span>
              </Button>
              
              <Button 
                variant="outline" 
                disabled
                className="h-auto p-3 sm:p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-green-400/10 to-emerald-400/10 border-2 border-green-200 dark:border-green-700 rounded-2xl text-gray-700 dark:text-gray-300"
              >
                <FileIcon className="h-4 w-4 sm:h-5 sm:w-5" />
                <span className="text-xs sm:text-sm font-medium text-center">Generate Report</span>
              </Button>
              
              <Button 
                variant="outline" 
                disabled
                className="h-auto p-3 sm:p-4 flex flex-col items-center gap-2 bg-gradient-to-r from-yellow-400/10 to-orange-400/10 border-2 border-yellow-200 dark:border-yellow-700 rounded-2xl text-gray-700 dark:text-gray-300"
              >
                <AlertCircleIcon className="h-4 w-4 sm:h-5 sm:w-5" />
                <span className="text-xs sm:text-sm font-medium text-center">Conflict Check</span>
              </Button>
            </div>
            
            <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/10 to-purple-400/10 p-0.5">
              <div className="relative bg-blue-50 dark:bg-blue-900/50 backdrop-blur-xl rounded-2xl p-3 sm:p-4">
                <h4 className="font-bold text-blue-900 dark:text-blue-100 mb-3 text-sm sm:text-base">⚡ Inheritance Performance</h4>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4 text-sm">
                  <div className="flex justify-between items-center">
                    <span className="text-blue-700 dark:text-blue-300">Cache Hit Rate:</span>
                    <span className="font-bold text-blue-900 dark:text-blue-100">94%</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-blue-700 dark:text-blue-300">Avg Resolution:</span>
                    <span className="font-bold text-blue-900 dark:text-blue-100">+12ms</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}