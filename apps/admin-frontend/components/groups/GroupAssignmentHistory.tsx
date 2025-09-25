/**
 * Group Assignment History Component
 * Comprehensive audit trail viewer for group assignment activities
 * 
 * Features:
 * - Display all group assignment/removal activities
 * - Manual admin assignments vs automatic Web3 assignments
 * - System cleanup activities and expiry tracking
 * - Advanced filtering and search capabilities
 * - Export audit data for compliance
 * - Real-time updates with pagination
 */

'use client';

import { useState, useCallback, useEffect, useMemo } from 'react';
import { 
  History, User, Users, Shield, Clock, Search, Filter,
  Calendar, Download, RefreshCw, Eye, ExternalLink,
  CheckCircle, XCircle, AlertCircle, Zap, Globe,
  ChevronLeft, ChevronRight, MoreHorizontal
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger 
} from '@/components/ui/dialog';
import { Progress } from '@/components/ui/progress';
import { useToast } from '@/components/ui/use-toast';

import { 
  usePermissionGroups,
  useGroupAssignmentHistory,
  useAdminGroupPermissions
} from '@/hooks/useGroupPermissions';
import { groupManagementClient } from '@/lib/api/group-management-client';
import { adminCardVariants, adminButtonVariants } from '@/design-system';
import { cn, formatDateTime, formatRelativeTime } from '@/lib/shared';

interface AssignmentHistoryEntry {
  id: string;
  user_id: string;
  user_email?: string;
  user_name?: string;
  group_id: string;
  group_name?: string;
  operation_type: 'assign' | 'remove' | 'expire' | 'cleanup';
  operation_source: 'manual' | 'web3_automatic' | 'system_cleanup' | 'bulk_operation';
  performed_by?: string;
  performed_by_name?: string;
  reason?: string;
  expires_at?: string;
  metadata?: Record<string, any>;
  created_at: string;
}

interface HistoryFilters {
  operation_type?: string;
  operation_source?: string;
  group_id?: string;
  user_search?: string;
  date_from?: string;
  date_to?: string;
  page: number;
  limit: number;
}

const OPERATION_TYPE_OPTIONS = [
  { value: 'assign', label: 'Assign to Group', icon: <CheckCircle className="h-3 w-3" /> },
  { value: 'remove', label: 'Remove from Group', icon: <XCircle className="h-3 w-3" /> },
  { value: 'expire', label: 'Expired Assignment', icon: <Clock className="h-3 w-3" /> },
  { value: 'cleanup', label: 'System Cleanup', icon: <RefreshCw className="h-3 w-3" /> }
];

const OPERATION_SOURCE_OPTIONS = [
  { value: 'manual', label: 'Manual Assignment', icon: <User className="h-3 w-3" /> },
  { value: 'web3_automatic', label: 'Web3 Auto-Assignment', icon: <Zap className="h-3 w-3" /> },
  { value: 'system_cleanup', label: 'System Cleanup', icon: <RefreshCw className="h-3 w-3" /> },
  { value: 'bulk_operation', label: 'Bulk Operation', icon: <Users className="h-3 w-3" /> }
];

export function GroupAssignmentHistory() {
  const { toast } = useToast();
  
  // State
  const [filters, setFilters] = useState<HistoryFilters>({
    page: 1,
    limit: 50
  });
  const [history, setHistory] = useState<AssignmentHistoryEntry[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [selectedEntry, setSelectedEntry] = useState<AssignmentHistoryEntry | null>(null);
  const [showDetailsDialog, setShowDetailsDialog] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(false);

  // Hooks
  const { groups } = usePermissionGroups();
  const { canViewAuditTrail } = useAdminGroupPermissions();

  // Load history data
  const loadHistory = useCallback(async () => {
    if (!canViewAuditTrail) return;
    
    setLoading(true);
    try {
      const response = await groupManagementClient.getGroupAssignmentHistory({
        ...filters,
        offset: (filters.page - 1) * filters.limit
      });
      
      if (response.success && response.data) {
        setHistory(response.data.history || []);
        setTotalCount(response.data.total || 0);
      } else {
        toast({
          title: 'Error Loading History',
          description: 'Failed to load group assignment history',
          variant: 'destructive'
        });
      }
    } catch (error) {
      console.error('Failed to load group assignment history:', error);
      toast({
        title: 'Error Loading History',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive'
      });
    } finally {
      setLoading(false);
    }
  }, [filters, canViewAuditTrail, toast]);

  // Load data on mount and filter changes
  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  // Auto refresh
  useEffect(() => {
    if (!autoRefresh) return;
    
    const interval = setInterval(loadHistory, 30000); // 30 seconds
    return () => clearInterval(interval);
  }, [autoRefresh, loadHistory]);

  // Filter handlers
  const updateFilter = useCallback((key: keyof HistoryFilters, value: any) => {
    setFilters(prev => ({
      ...prev,
      [key]: value,
      page: key !== 'page' ? 1 : prev.page // Reset page when changing other filters
    }));
  }, []);

  const clearFilters = useCallback(() => {
    setFilters({
      page: 1,
      limit: 50
    });
  }, []);

  // Export functionality
  const exportHistory = useCallback(async () => {
    try {
      const allHistory = await groupManagementClient.getGroupAssignmentHistory({
        ...filters,
        limit: 10000, // Export all matching records
        offset: 0
      });
      
      if (allHistory.success && allHistory.data?.history) {
        const csv = convertHistoryToCSV(allHistory.data.history);
        downloadCSV(csv, `group-assignment-history-${new Date().toISOString().split('T')[0]}.csv`);
        
        toast({
          title: 'Export Complete',
          description: `Exported ${allHistory.data.history.length} history records`
        });
      }
    } catch (error) {
      toast({
        title: 'Export Failed',
        description: error instanceof Error ? error.message : 'Export failed',
        variant: 'destructive'
      });
    }
  }, [filters, toast]);

  // Helper functions
  const convertHistoryToCSV = (data: AssignmentHistoryEntry[]): string => {
    const headers = [
      'Timestamp', 'Operation', 'Source', 'User Email', 'User Name',
      'Group Name', 'Performed By', 'Reason', 'Expires At', 'Metadata'
    ];
    
    const rows = data.map(entry => [
      entry.created_at,
      entry.operation_type,
      entry.operation_source,
      entry.user_email || '',
      entry.user_name || '',
      entry.group_name || '',
      entry.performed_by_name || entry.performed_by || '',
      entry.reason || '',
      entry.expires_at || '',
      JSON.stringify(entry.metadata || {})
    ]);
    
    return [headers.join(','), ...rows.map(row => row.join(','))].join('\n');
  };

  const downloadCSV = (csv: string, filename: string) => {
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);
    link.setAttribute('href', url);
    link.setAttribute('download', filename);
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const getOperationIcon = (type: string, source: string) => {
    if (source === 'web3_automatic') return <Zap className="h-4 w-4 text-blue-600" />;
    if (source === 'system_cleanup') return <RefreshCw className="h-4 w-4 text-gray-600" />;
    if (type === 'assign') return <CheckCircle className="h-4 w-4 text-green-600" />;
    if (type === 'remove') return <XCircle className="h-4 w-4 text-red-600" />;
    if (type === 'expire') return <Clock className="h-4 w-4 text-orange-600" />;
    return <AlertCircle className="h-4 w-4 text-gray-600" />;
  };

  const getOperationBadge = (type: string, source: string) => {
    const baseClasses = "text-xs font-medium";
    
    if (source === 'web3_automatic') {
      return <Badge className={`${baseClasses} bg-blue-100 text-blue-800`}>Web3 Auto</Badge>;
    }
    if (source === 'system_cleanup') {
      return <Badge className={`${baseClasses} bg-gray-100 text-gray-800`}>System</Badge>;
    }
    if (source === 'bulk_operation') {
      return <Badge className={`${baseClasses} bg-purple-100 text-purple-800`}>Bulk</Badge>;
    }
    if (type === 'assign') {
      return <Badge className={`${baseClasses} bg-green-100 text-green-800`}>Assigned</Badge>;
    }
    if (type === 'remove') {
      return <Badge className={`${baseClasses} bg-red-100 text-red-800`}>Removed</Badge>;
    }
    return <Badge className={`${baseClasses} bg-gray-100 text-gray-800`}>Manual</Badge>;
  };

  if (!canViewAuditTrail) {
    return (
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardContent className="p-12 text-center">
          <Shield className="h-16 w-16 text-gray-400 mx-auto mb-4" />
          <h3 className="text-xl font-semibold text-gray-900 mb-2">
            Access Denied
          </h3>
          <p className="text-gray-600">
            You don't have permission to view the audit trail.
          </p>
        </CardContent>
      </Card>
    );
  }

  const totalPages = Math.ceil(totalCount / filters.limit);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Assignment History</h2>
          <p className="text-gray-600">
            Comprehensive audit trail of all group assignment activities
          </p>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={autoRefresh ? 'bg-green-50 text-green-700' : ''}
          >
            <RefreshCw className={cn("h-4 w-4 mr-2", autoRefresh && "animate-spin")} />
            Auto Refresh {autoRefresh ? 'ON' : 'OFF'}
          </Button>
          
          <Button
            variant="outline"
            size="sm"
            onClick={exportHistory}
            disabled={loading}
          >
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
          
          <Button
            variant="outline"
            size="sm"
            onClick={loadHistory}
            disabled={loading}
          >
            <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
          </Button>
        </div>
      </div>

      {/* Filters */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Filter className="h-5 w-5" />
            Filters
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div>
              <Label>Operation Type</Label>
              <Select 
                value={filters.operation_type || ''} 
                onValueChange={(value) => updateFilter('operation_type', value || undefined)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="All types" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">All Types</SelectItem>
                  {OPERATION_TYPE_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      <div className="flex items-center gap-2">
                        {option.icon}
                        {option.label}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Operation Source</Label>
              <Select 
                value={filters.operation_source || ''} 
                onValueChange={(value) => updateFilter('operation_source', value || undefined)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="All sources" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">All Sources</SelectItem>
                  {OPERATION_SOURCE_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      <div className="flex items-center gap-2">
                        {option.icon}
                        {option.label}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Group</Label>
              <Select 
                value={filters.group_id || ''} 
                onValueChange={(value) => updateFilter('group_id', value || undefined)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="All groups" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">All Groups</SelectItem>
                  {groups.map((group) => (
                    <SelectItem key={group.id} value={group.id}>
                      <div className="flex items-center gap-2">
                        {group.is_system_group ? (
                          <Shield className="h-3 w-3 text-yellow-600" />
                        ) : (
                          <Users className="h-3 w-3 text-blue-600" />
                        )}
                        {group.name}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>User Search</Label>
              <Input
                placeholder="Search by email or name"
                value={filters.user_search || ''}
                onChange={(e) => updateFilter('user_search', e.target.value || undefined)}
              />
            </div>
          </div>
          
          <div className="flex justify-between items-center mt-4">
            <div className="text-sm text-gray-600">
              Showing {history.length} of {totalCount} records
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={clearFilters}
            >
              Clear Filters
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* History List */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardContent className="p-0">
          {loading ? (
            <div className="p-12 text-center">
              <RefreshCw className="h-8 w-8 animate-spin text-gray-400 mx-auto mb-4" />
              <p className="text-gray-600">Loading history...</p>
            </div>
          ) : history.length === 0 ? (
            <div className="p-12 text-center">
              <History className="h-16 w-16 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                No History Found
              </h3>
              <p className="text-gray-600">
                No group assignment history matches your current filters.
              </p>
            </div>
          ) : (
            <div className="divide-y divide-gray-200">
              {history.map((entry) => (
                <div key={entry.id} className="p-6 hover:bg-gray-50">
                  <div className="flex items-start justify-between">
                    <div className="flex items-start gap-3 flex-1">
                      <div className="mt-1">
                        {getOperationIcon(entry.operation_type, entry.operation_source)}
                      </div>
                      
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <h4 className="text-sm font-medium text-gray-900">
                            {entry.operation_type === 'assign' ? 'Assigned to' : 
                             entry.operation_type === 'remove' ? 'Removed from' :
                             entry.operation_type === 'expire' ? 'Expired from' : 'Cleaned up from'} {entry.group_name}
                          </h4>
                          {getOperationBadge(entry.operation_type, entry.operation_source)}
                        </div>
                        
                        <div className="text-sm text-gray-600 mb-2">
                          <strong>User:</strong> {entry.user_email} 
                          {entry.user_name && ` (${entry.user_name})`}
                        </div>
                        
                        {entry.reason && (
                          <div className="text-sm text-gray-600 mb-2">
                            <strong>Reason:</strong> {entry.reason}
                          </div>
                        )}
                        
                        {entry.performed_by && (
                          <div className="text-sm text-gray-600 mb-2">
                            <strong>Performed by:</strong> {entry.performed_by_name || entry.performed_by}
                          </div>
                        )}
                        
                        <div className="flex items-center gap-4 text-xs text-gray-500">
                          <span className="flex items-center gap-1">
                            <Calendar className="h-3 w-3" />
                            {formatDateTime(entry.created_at)}
                          </span>
                          
                          {entry.expires_at && (
                            <span className="flex items-center gap-1">
                              <Clock className="h-3 w-3" />
                              Expires: {formatDateTime(entry.expires_at)}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                    
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => {
                        setSelectedEntry(entry);
                        setShowDetailsDialog(true);
                      }}
                    >
                      <Eye className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between">
          <div className="text-sm text-gray-600">
            Page {filters.page} of {totalPages}
          </div>
          
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => updateFilter('page', Math.max(1, filters.page - 1))}
              disabled={filters.page <= 1}
            >
              <ChevronLeft className="h-4 w-4" />
              Previous
            </Button>
            
            <Button
              variant="outline"
              size="sm"
              onClick={() => updateFilter('page', Math.min(totalPages, filters.page + 1))}
              disabled={filters.page >= totalPages}
            >
              Next
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      )}

      {/* Details Dialog */}
      <Dialog open={showDetailsDialog} onOpenChange={setShowDetailsDialog}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <History className="h-5 w-5" />
              Assignment History Details
            </DialogTitle>
          </DialogHeader>
          
          {selectedEntry && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-xs font-medium text-gray-500">Operation</Label>
                  <div className="flex items-center gap-2 mt-1">
                    {getOperationIcon(selectedEntry.operation_type, selectedEntry.operation_source)}
                    <span className="font-medium">
                      {selectedEntry.operation_type.charAt(0).toUpperCase() + selectedEntry.operation_type.slice(1)}
                    </span>
                    {getOperationBadge(selectedEntry.operation_type, selectedEntry.operation_source)}
                  </div>
                </div>
                
                <div>
                  <Label className="text-xs font-medium text-gray-500">Timestamp</Label>
                  <div className="mt-1">{formatDateTime(selectedEntry.created_at)}</div>
                </div>
                
                <div>
                  <Label className="text-xs font-medium text-gray-500">User</Label>
                  <div className="mt-1">
                    {selectedEntry.user_email}
                    {selectedEntry.user_name && (
                      <div className="text-sm text-gray-600">{selectedEntry.user_name}</div>
                    )}
                  </div>
                </div>
                
                <div>
                  <Label className="text-xs font-medium text-gray-500">Group</Label>
                  <div className="mt-1">{selectedEntry.group_name}</div>
                </div>
                
                {selectedEntry.performed_by && (
                  <div className="col-span-2">
                    <Label className="text-xs font-medium text-gray-500">Performed By</Label>
                    <div className="mt-1">{selectedEntry.performed_by_name || selectedEntry.performed_by}</div>
                  </div>
                )}
                
                {selectedEntry.reason && (
                  <div className="col-span-2">
                    <Label className="text-xs font-medium text-gray-500">Reason</Label>
                    <div className="mt-1">{selectedEntry.reason}</div>
                  </div>
                )}
                
                {selectedEntry.expires_at && (
                  <div>
                    <Label className="text-xs font-medium text-gray-500">Expires At</Label>
                    <div className="mt-1">{formatDateTime(selectedEntry.expires_at)}</div>
                  </div>
                )}
              </div>
              
              {selectedEntry.metadata && Object.keys(selectedEntry.metadata).length > 0 && (
                <div>
                  <Label className="text-xs font-medium text-gray-500">Metadata</Label>
                  <pre className="mt-1 text-sm bg-gray-50 p-3 rounded border overflow-x-auto">
                    {JSON.stringify(selectedEntry.metadata, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}