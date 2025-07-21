'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Activity,
  ChevronRight,
  Clock,
  Download,
  Eye,
  FileText,
  Filter,
  Search,
} from 'lucide-react';
import React, { useState } from 'react';

export const ActivityLogsDashboard: React.FC = () => {
  const [activeView, setActiveView] = useState('recent');

  const logViews = [
    {
      id: 'recent',
      label: 'Recent Activity',
      icon: Clock,
      description: 'Latest system and user activities',
    },
    {
      id: 'user',
      label: 'User Activities',
      icon: Eye,
      description: 'User login and action logs',
    },
    {
      id: 'system',
      label: 'System Logs',
      icon: Activity,
      description: 'System events and errors',
    },
    {
      id: 'audit',
      label: 'Audit Trail',
      icon: FileText,
      description: 'Security and compliance logs',
    },
  ];

  const mockLogs = [
    {
      id: 1,
      type: 'login',
      user: 'john.doe@example.com',
      action: 'User logged in',
      timestamp: '2 minutes ago',
      status: 'success',
    },
    {
      id: 2,
      type: 'system',
      user: 'System',
      action: 'Database backup completed',
      timestamp: '15 minutes ago',
      status: 'success',
    },
    {
      id: 3,
      type: 'permission',
      user: 'admin@example.com',
      action: 'Updated user permissions',
      timestamp: '1 hour ago',
      status: 'warning',
    },
    {
      id: 4,
      type: 'error',
      user: 'System',
      action: 'Failed API request to external service',
      timestamp: '2 hours ago',
      status: 'error',
    },
  ];

  const renderContent = () => {
    switch (activeView) {
      case 'recent':
        return (
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  <span className="flex items-center gap-2">
                    <Clock className="h-5 w-5 text-blue-600" />
                    Recent Activity
                  </span>
                  <div className="flex items-center gap-2">
                    <button className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
                      <Search className="h-4 w-4 text-gray-500" />
                    </button>
                    <button className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
                      <Filter className="h-4 w-4 text-gray-500" />
                    </button>
                  </div>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {mockLogs.map((log) => (
                    <div
                      key={log.id}
                      className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg"
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className={`w-2 h-2 rounded-full ${
                            log.status === 'success'
                              ? 'bg-green-500'
                              : log.status === 'warning'
                                ? 'bg-yellow-500'
                                : 'bg-red-500'
                          }`}
                        ></div>
                        <div>
                          <div className="font-medium text-gray-900 dark:text-white">
                            {log.action}
                          </div>
                          <div className="text-sm text-gray-500 dark:text-gray-400">
                            {log.user}
                          </div>
                        </div>
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">
                        {log.timestamp}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        );
      default:
        return (
          <div className="text-center py-12">
            <div className="text-gray-400 mb-4">
              <Activity className="h-12 w-12 mx-auto" />
            </div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
              {logViews.find((v) => v.id === activeView)?.label}
            </h3>
            <p className="text-gray-500 dark:text-gray-400">
              This section is under development
            </p>
          </div>
        );
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Activity Logs
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Monitor system activities, user actions, and audit trails
          </p>
        </div>
        <div className="flex items-center gap-3">
          <button className="flex items-center gap-2 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            <Filter className="h-4 w-4" />
            <span className="text-sm">Filter</span>
          </button>
          <button className="flex items-center gap-2 px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
            <Download className="h-4 w-4" />
            <span className="text-sm">Export</span>
          </button>
        </div>
      </div>

      {/* Log Views Navigation */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {logViews.map((view, index) => {
            const Icon = view.icon;
            const isActive = activeView === view.id;

            return (
              <button
                key={view.id}
                onClick={() => setActiveView(view.id)}
                className={`
                  submenu-item group p-4 rounded-lg text-left transition-all duration-200 border
                  ${
                    isActive
                      ? 'bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800 shadow-md'
                      : 'bg-gray-50 dark:bg-gray-700/50 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 border-gray-200 dark:border-gray-600'
                  }
                  transform hover:scale-[1.02] active:scale-[0.98]
                `}
                style={{
                  animationDelay: `${index * 100}ms`,
                }}
              >
                <div className="flex items-center gap-3 mb-3">
                  <div
                    className={`
                    p-2.5 rounded-lg transition-all duration-200
                    ${
                      isActive
                        ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                        : 'bg-white dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                    }
                  `}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors ${isActive ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'}`}
                    />
                  </div>
                  <ChevronRight
                    className={`h-4 w-4 transition-all duration-200 ${isActive ? 'text-blue-600 dark:text-blue-400 rotate-90' : 'text-gray-400'}`}
                  />
                </div>
                <div>
                  <div
                    className={`font-semibold text-sm mb-1 ${isActive ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'}`}
                  >
                    {view.label}
                  </div>
                  <div
                    className={`text-xs ${isActive ? 'text-blue-600/80 dark:text-blue-400/80' : 'text-gray-500 dark:text-gray-400'}`}
                  >
                    {view.description}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Content */}
      <div className="animate-fade-in">{renderContent()}</div>
    </div>
  );
};
