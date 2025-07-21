'use client';

import React, { useState } from 'react';
import { 
  Calendar, 
  User, 
  Shield, 
  Search, 
  Filter,
  ChevronDown,
  Eye,
  Clock,
  AlertCircle,
  CheckCircle,
  XCircle,
  Info
} from 'lucide-react';

interface ActivityLog {
  id: string;
  timestamp: Date;
  action: string;
  resource: string;
  userId: string;
  userName: string;
  performedBy: string;
  performedByName: string;
  status: 'success' | 'warning' | 'error' | 'info';
  details: string;
  metadata?: Record<string, any>;
}

export const ActivityLogs: React.FC = () => {
  const [logs] = useState<ActivityLog[]>([
    {
      id: '1',
      timestamp: new Date(),
      action: 'package_upgrade',
      resource: 'user_package',
      userId: 'user123',
      userName: 'john.doe@example.com',
      performedBy: 'admin001',
      performedByName: 'Admin User',
      status: 'success',
      details: 'Package upgraded from Bronze to Gold',
      metadata: { oldTier: 'bronze', newTier: 'gold' }
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 3600000),
      action: 'permission_granted',
      resource: 'custom_permission',
      userId: 'user456',
      userName: 'jane.smith@example.com',
      performedBy: 'admin001',
      performedByName: 'Admin User',
      status: 'success',
      details: 'Custom permission granted for API access',
      metadata: { permission: 'api_premium_access', duration: '30 days' }
    },
    {
      id: '3',
      timestamp: new Date(Date.now() - 7200000),
      action: 'login_attempt',
      resource: 'authentication',
      userId: 'user789',
      userName: 'bob.wilson@example.com',
      performedBy: 'user789',
      performedByName: 'Bob Wilson',
      status: 'error',
      details: 'Failed login attempt - invalid credentials',
      metadata: { ip: '192.168.1.100', userAgent: 'Mozilla/5.0...' }
    },
    {
      id: '4',
      timestamp: new Date(Date.now() - 10800000),
      action: 'subscription_renewal',
      resource: 'subscription',
      userId: 'user101',
      userName: 'alice.brown@example.com',
      performedBy: 'system',
      performedByName: 'System',
      status: 'success',
      details: 'Subscription automatically renewed',
      metadata: { amount: 99.99, currency: 'USD' }
    },
    {
      id: '5',
      timestamp: new Date(Date.now() - 14400000),
      action: 'permission_revoked',
      resource: 'custom_permission',
      userId: 'user202',
      userName: 'charlie.green@example.com',
      performedBy: 'admin002',
      performedByName: 'Support Admin',
      status: 'warning',
      details: 'Custom permission revoked due to policy violation',
      metadata: { permission: 'bulk_export', reason: 'policy_violation' }
    }
  ]);
  
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedAction, setSelectedAction] = useState('');
  const [selectedStatus, setSelectedStatus] = useState('');
  const [showFilters, setShowFilters] = useState(false);

  const actions = ['All Actions', 'package_upgrade', 'permission_granted', 'permission_revoked', 'login_attempt', 'subscription_renewal'];
  const statuses = ['All Statuses', 'success', 'warning', 'error', 'info'];

  const filteredLogs = logs.filter(log => {
    const matchesSearch = !searchQuery || 
      log.userName.toLowerCase().includes(searchQuery.toLowerCase()) ||
      log.details.toLowerCase().includes(searchQuery.toLowerCase()) ||
      log.action.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesAction = !selectedAction || 
      selectedAction === 'All Actions' || 
      log.action === selectedAction;
    
    const matchesStatus = !selectedStatus || 
      selectedStatus === 'All Statuses' || 
      log.status === selectedStatus;
    
    return matchesSearch && matchesAction && matchesStatus;
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success':
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'warning':
        return <AlertCircle className="h-4 w-4 text-yellow-500" />;
      case 'error':
        return <XCircle className="h-4 w-4 text-red-500" />;
      default:
        return <Info className="h-4 w-4 text-blue-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'success':
        return 'bg-green-100 text-green-800';
      case 'warning':
        return 'bg-yellow-100 text-yellow-800';
      case 'error':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-blue-100 text-blue-800';
    }
  };

  const formatAction = (action: string) => {
    return action.split('_').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  const formatTimestamp = (timestamp: Date) => {
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - timestamp.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) {
      const diffInMinutes = Math.floor((now.getTime() - timestamp.getTime()) / (1000 * 60));
      return `${diffInMinutes} minutes ago`;
    } else if (diffInHours < 24) {
      return `${diffInHours} hours ago`;
    } else {
      return timestamp.toLocaleDateString();
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row gap-4 justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900">Activity Logs</h3>
          <p className="text-sm text-gray-500 mt-1">
            Track all system activities and user actions
          </p>
        </div>
        
        <div className="flex gap-2">
          <button className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50">
            <Calendar className="h-4 w-4 mr-2" />
            Export
          </button>
        </div>
      </div>

      {/* Search and Filter */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
          <input
            type="text"
            placeholder="Search by user, action, or details..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        
        <button
          onClick={() => setShowFilters(!showFilters)}
          className={`inline-flex items-center px-4 py-2 border rounded-lg text-sm font-medium transition-colors ${
            showFilters 
              ? 'border-blue-300 bg-blue-50 text-blue-700' 
              : 'border-gray-300 bg-white text-gray-700 hover:bg-gray-50'
          }`}
        >
          <Filter className="h-4 w-4 mr-2" />
          Filters
          <ChevronDown className={`h-4 w-4 ml-2 transition-transform ${showFilters ? 'rotate-180' : ''}`} />
        </button>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Action Type
              </label>
              <select
                value={selectedAction}
                onChange={(e) => setSelectedAction(e.target.value)}
                className="w-full border border-gray-300 rounded-lg px-3 py-2"
              >
                {actions.map((action) => (
                  <option key={action} value={action === 'All Actions' ? '' : action}>
                    {action === 'All Actions' ? action : formatAction(action)}
                  </option>
                ))}
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Status
              </label>
              <select
                value={selectedStatus}
                onChange={(e) => setSelectedStatus(e.target.value)}
                className="w-full border border-gray-300 rounded-lg px-3 py-2"
              >
                {statuses.map((status) => (
                  <option key={status} value={status === 'All Statuses' ? '' : status}>
                    {status === 'All Statuses' ? status : status.charAt(0).toUpperCase() + status.slice(1)}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>
      )}

      {/* Activity Logs */}
      <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Timestamp
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Action
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  User
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Performed By
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Details
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filteredLogs.map((log) => (
                <tr key={log.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    <div className="flex items-center">
                      <Clock className="h-4 w-4 mr-2 text-gray-400" />
                      {formatTimestamp(log.timestamp)}
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <Shield className="h-4 w-4 mr-2 text-gray-400" />
                      <span className="text-sm font-medium text-gray-900">
                        {formatAction(log.action)}
                      </span>
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <div className="h-8 w-8 rounded-full bg-blue-100 flex items-center justify-center mr-3">
                        <User className="h-4 w-4 text-blue-600" />
                      </div>
                      <div>
                        <div className="text-sm font-medium text-gray-900">
                          {log.userName.split('@')[0]}
                        </div>
                        <div className="text-sm text-gray-500">{log.userName}</div>
                      </div>
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {log.performedByName}
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      {getStatusIcon(log.status)}
                      <span className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(log.status)}`}>
                        {log.status}
                      </span>
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 text-sm text-gray-900 max-w-xs">
                    <div className="truncate" title={log.details}>
                      {log.details}
                    </div>
                  </td>
                  
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <button className="text-blue-600 hover:text-blue-900">
                      <Eye className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        
        {filteredLogs.length === 0 && (
          <div className="text-center py-12">
            <Calendar className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 mb-2">No activity logs found</h3>
            <p className="text-gray-500">
              {searchQuery || selectedAction || selectedStatus 
                ? 'Try adjusting your search or filters' 
                : 'Activity logs will appear here as users interact with the system'
              }
            </p>
          </div>
        )}
      </div>

      {/* Pagination would go here */}
      <div className="flex items-center justify-between">
        <div className="text-sm text-gray-700">
          Showing {filteredLogs.length} of {logs.length} entries
        </div>
        {/* Pagination controls would go here */}
      </div>
    </div>
  );
};
