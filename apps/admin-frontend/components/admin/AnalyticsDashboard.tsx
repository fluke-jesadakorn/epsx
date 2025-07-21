'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Activity,
  BarChart3,
  ChevronRight,
  Download,
  Filter,
  TrendingUp,
  Users,
} from 'lucide-react';
import React, { useState } from 'react';

export const AnalyticsDashboard: React.FC = () => {
  const [activeView, setActiveView] = useState('overview');

  const analyticsViews = [
    {
      id: 'overview',
      label: 'Analytics Overview',
      icon: BarChart3,
      description: 'Key metrics and performance indicators',
    },
    {
      id: 'users',
      label: 'User Analytics',
      icon: Users,
      description: 'User behavior and engagement',
    },
    {
      id: 'performance',
      label: 'Performance',
      icon: TrendingUp,
      description: 'System performance metrics',
    },
    {
      id: 'activity',
      label: 'Activity Analytics',
      icon: Activity,
      description: 'Platform activity and usage',
    },
  ];

  const renderContent = () => {
    switch (activeView) {
      case 'overview':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <BarChart3 className="h-5 w-5 text-blue-600" />
                  Total Users
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-gray-900 dark:text-white">
                  12,459
                </div>
                <p className="text-sm text-green-600">+15% from last month</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <TrendingUp className="h-5 w-5 text-green-600" />
                  Revenue
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-gray-900 dark:text-white">
                  $45,231
                </div>
                <p className="text-sm text-green-600">+22% from last month</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5 text-orange-600" />
                  Active Sessions
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-gray-900 dark:text-white">
                  1,429
                </div>
                <p className="text-sm text-orange-600">-5% from last hour</p>
              </CardContent>
            </Card>
          </div>
        );
      default:
        return (
          <div className="text-center py-12">
            <div className="text-gray-400 mb-4">
              <BarChart3 className="h-12 w-12 mx-auto" />
            </div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
              {analyticsViews.find((v) => v.id === activeView)?.label}
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
            Analytics Dashboard
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Monitor performance, user behavior, and system metrics
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

      {/* Analytics Navigation */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {analyticsViews.map((view, index) => {
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
