'use client';

import React, { useState, useEffect } from 'react';
import { 
  Users, 
  Shield, 
  Activity, 
  Settings,
  Plus,
  UserPlus,
  Crown,
  AlertCircle
} from 'lucide-react';
import { 
  UserManagement, 
  PermissionTemplates, 
  ActivityLogs, 
  IAMSettings, 
  StatsCard 
} from './iam';
import { AddUserModal } from './iam/AddUserModal';
import { iamService } from '../../services/iamService';

interface DashboardStats {
  totalUsers: number;
  activeSubscriptions: number;
  customPermissions: number;
  recentActivity: number;
}

export const IAMDashboardNew: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'users' | 'permissions' | 'activity' | 'settings'>('users');
  const [stats, setStats] = useState<DashboardStats>({
    totalUsers: 0,
    activeSubscriptions: 0,
    customPermissions: 0,
    recentActivity: 0,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddUserModal, setShowAddUserModal] = useState(false);

  useEffect(() => {
    loadDashboardStats();
  }, []);

  const loadDashboardStats = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const users = await iamService.getUsers();
      
      setStats({
        totalUsers: users.length,
        activeSubscriptions: users.filter(u => u.subscriptionStatus === 'active').length,
        customPermissions: users.reduce((acc, u) => acc + (u.customPermissions?.length || 0), 0),
        recentActivity: 12, // This would come from audit logs
      });
    } catch (err) {
      console.error('Failed to load dashboard stats:', err);
      setError('Failed to load dashboard data');
      // Set mock data for demo
      setStats({
        totalUsers: 156,
        activeSubscriptions: 89,
        customPermissions: 23,
        recentActivity: 12,
      });
    } finally {
      setLoading(false);
    }
  };

  const tabs = [
    {
      id: 'users' as const,
      name: 'User Management',
      icon: Users,
      description: 'Manage users, packages, and permissions'
    },
    {
      id: 'permissions' as const,
      name: 'Permission Templates',
      icon: Shield,
      description: 'Create and manage permission templates'
    },
    {
      id: 'activity' as const,
      name: 'Activity Logs',
      icon: Activity,
      description: 'View system activity and audit trails'
    },
    {
      id: 'settings' as const,
      name: 'Settings',
      icon: Settings,
      description: 'Configure IAM system settings'
    }
  ];

  const renderTabContent = () => {
    switch (activeTab) {
      case 'users':
        return <UserManagement onStatsUpdate={loadDashboardStats} />;
      case 'permissions':
        return <PermissionTemplates />;
      case 'activity':
        return <ActivityLogs />;
      case 'settings':
        return <IAMSettings />;
      default:
        return <UserManagement onStatsUpdate={loadDashboardStats} />;
    }
  };

  return (
    <div className="min-h-screen">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 mb-6">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="py-6">
            <div className="flex items-center justify-between">
              <div className="animate-fade-in">
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                  <Shield className="h-7 w-7 text-blue-600" />
                  Identity & Access Management
                </h1>
                <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
                  Manage user access, permissions, and security settings
                </p>
              </div>
              
              <div className="flex gap-3 animate-fade-in-delayed">
                <button className="inline-flex items-center px-3 py-2 border border-gray-300 dark:border-gray-600 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors">
                  <Plus className="h-4 w-4 mr-2" />
                  Quick Action
                </button>
                <button 
                  onClick={() => setShowAddUserModal(true)}
                  className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                >
                  <UserPlus className="h-4 w-4 mr-2" />
                  Add User
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Error Banner */}
        {error && (
          <div className="mb-6 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 animate-fade-in">
            <div className="flex items-center">
              <AlertCircle className="h-5 w-5 text-red-400 mr-3" />
              <div>
                <h3 className="text-sm font-medium text-red-800 dark:text-red-200">Connection Issue</h3>
                <p className="text-sm text-red-700 dark:text-red-300 mt-1">{error}. Using demo data.</p>
              </div>
            </div>
          </div>
        )}

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="animate-fade-in">
            <StatsCard
              title="Total Users"
              value={stats.totalUsers}
              icon={Users}
              color="blue"
              loading={loading}
              trend={{ value: 12, type: 'increase' }}
            />
          </div>
          <div className="animate-fade-in-delayed">
            <StatsCard
              title="Active Subscriptions"
              value={stats.activeSubscriptions}
              icon={Crown}
              color="green"
              loading={loading}
              trend={{ value: 8, type: 'increase' }}
            />
          </div>
          <div className="animate-fade-in-delayed-2">
            <StatsCard
              title="Custom Permissions"
              value={stats.customPermissions}
              icon={Shield}
              color="purple"
              loading={loading}
            />
          </div>
          <div className="animate-fade-in-delayed-3">
            <StatsCard
              title="Recent Activity"
              value={stats.recentActivity}
              icon={Activity}
              color="orange"
              loading={loading}
              subtitle="Last 24 hours"
            />
          </div>
        </div>

        {/* Navigation Tabs */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 mb-6 overflow-hidden">
          <div className="border-b border-gray-200 dark:border-gray-700">
            <nav className="-mb-px flex">
              {tabs.map((tab) => {
                const Icon = tab.icon;
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`group relative min-w-0 flex-1 overflow-hidden py-4 px-6 text-sm font-medium text-center focus:z-10 focus:outline-none transition-colors ${
                      activeTab === tab.id
                        ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 bg-blue-50 dark:bg-blue-900/20'
                        : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700/50'
                    }`}
                  >
                    <div className="flex items-center justify-center">
                      <Icon className={`h-5 w-5 mr-2 transition-colors ${
                        activeTab === tab.id ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400 group-hover:text-gray-500 dark:group-hover:text-gray-300'
                      }`} />
                      <span className="hidden sm:inline">{tab.name}</span>
                      <span className="sm:hidden">{tab.name.split(' ')[0]}</span>
                    </div>
                    {activeTab !== tab.id && (
                      <div className="hidden lg:block mt-1 text-xs text-gray-400 dark:text-gray-500 group-hover:text-gray-500 dark:group-hover:text-gray-400">
                        {tab.description}
                      </div>
                    )}
                  </button>
                );
              })}
            </nav>
          </div>

          {/* Tab Content */}
          <div className="p-6 min-h-[400px]">
            {renderTabContent()}
          </div>
        </div>
        
        {/* Add User Modal */}
        <AddUserModal
          isOpen={showAddUserModal}
          onClose={() => setShowAddUserModal(false)}
          onSuccess={() => {
            loadDashboardStats();
            // You could also trigger a refresh in the UserManagement component
          }}
        />
      </div>
    </div>
  );
};
