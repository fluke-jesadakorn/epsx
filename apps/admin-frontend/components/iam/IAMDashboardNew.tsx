'use client';

import {
  Activity,
  ArrowLeft,
  BarChart3,
  ChevronRight,
  Home,
  Shield,
  Users,
} from 'lucide-react';
import Link from 'next/link';
import React, { useState } from 'react';
import { DashboardOverview } from './dashboard/DashboardOverview';
import { ActivityLogs } from './logs/ActivityLogs';
import { PermissionTemplates } from './templates/PermissionTemplates';
import { UserManagement } from './users/UserManagement';

export const IAMDashboardNew: React.FC = () => {
  const [activeSection, setActiveSection] = useState('overview');
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const menuItems = [
    {
      id: 'overview',
      label: 'Overview',
      icon: BarChart3,
      description: 'Dashboard overview and statistics',
    },
    {
      id: 'users',
      label: 'User Management',
      icon: Users,
      description: 'Manage user accounts and permissions',
    },
    {
      id: 'templates',
      label: 'Permission Templates',
      icon: Shield,
      description: 'Create and manage permission templates',
    },
    {
      id: 'logs',
      label: 'Activity Logs',
      icon: Activity,
      description: 'View system activity and audit logs',
    },
  ];

  const renderContent = () => {
    switch (activeSection) {
      case 'overview':
        return <DashboardOverview />;
      case 'users':
        return <UserManagement />;
      case 'templates':
        return <PermissionTemplates />;
      case 'logs':
        return <ActivityLogs />;
      default:
        return <DashboardOverview />;
    }
  };

  return (
    <div className="flex min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Mobile Menu Button */}
      <div className="lg:hidden fixed top-4 left-4 z-50">
        <button
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
          className="p-2 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700"
        >
          <Home className="h-5 w-5 text-gray-600 dark:text-gray-400" />
        </button>
      </div>

      {/* Mobile Overlay */}
      {mobileMenuOpen && (
        <div
          className="lg:hidden fixed inset-0 z-40 bg-black/50"
          onClick={() => setMobileMenuOpen(false)}
        />
      )}

      {/* Sidebar Submenu */}
      <div
        className={`
        w-72 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col
        lg:relative lg:translate-x-0 
        fixed inset-y-0 left-0 z-50 transform transition-transform duration-300 ease-in-out
        ${mobileMenuOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
      `}
      >
        {/* Header */}
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between mb-3">
            <Link
              href="/"
              className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
            >
              <ArrowLeft className="h-4 w-4" />
              <span>Back to Dashboard</span>
            </Link>
            {/* Mobile close button */}
            <button
              onClick={() => setMobileMenuOpen(false)}
              className="lg:hidden p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              <svg
                className="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
              <Shield className="h-6 w-6 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
                IAM Console
              </h1>
            </div>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Manage users, permissions, and access control
          </p>
        </div>

        {/* Navigation Menu */}
        <nav className="flex-1 p-4 space-y-2 submenu-scroll overflow-y-auto">
          {menuItems.map((item, index) => {
            const Icon = item.icon;
            const isActive = activeSection === item.id;

            return (
              <button
                key={item.id}
                onClick={() => {
                  setActiveSection(item.id);
                  setMobileMenuOpen(false);
                }}
                className={`
                  submenu-item w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all duration-200 group
                  ${
                    isActive
                      ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-800/50 shadow-sm'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent hover:border-gray-200 dark:hover:border-gray-600'
                  }
                  transform hover:scale-[1.02] active:scale-[0.98]
                `}
                style={{
                  animationDelay: `${index * 50}ms`,
                }}
              >
                <div
                  className={`
                  p-2.5 rounded-lg transition-all duration-200
                  ${
                    isActive
                      ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                      : 'bg-gray-100 dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                  }
                `}
                >
                  <Icon
                    className={`h-4 w-4 transition-colors ${isActive ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'}`}
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <div
                    className={`font-medium text-sm transition-colors ${isActive ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'}`}
                  >
                    {item.label}
                  </div>
                  <div
                    className={`text-xs mt-0.5 truncate transition-colors ${isActive ? 'text-blue-600/80 dark:text-blue-400/80' : 'text-gray-500 dark:text-gray-400'}`}
                  >
                    {item.description}
                  </div>
                </div>
                <ChevronRight
                  className={`h-4 w-4 transition-all duration-200 ${isActive ? 'text-blue-600 dark:text-blue-400 rotate-90 scale-110' : 'text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300'}`}
                />
              </button>
            );
          })}
        </nav>

        {/* Footer */}
        <div className="p-4 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
              <Shield className="h-3 w-3" />
              <span>IAM Console v2.0</span>
            </div>
            <div className="flex items-center gap-1">
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              <span className="text-xs text-green-600 dark:text-green-400">
                Online
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Breadcrumb Header */}
        <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
              <Home className="h-4 w-4" />
              <ChevronRight className="h-3 w-3" />
              <span className="text-gray-900 dark:text-white font-medium">
                Admin
              </span>
              <ChevronRight className="h-3 w-3" />
              <span className="text-blue-600 dark:text-blue-400 font-medium">
                {menuItems.find((item) => item.id === activeSection)?.label}
              </span>
            </div>
            <div className="flex items-center gap-3">
              <div className="text-sm text-gray-500 dark:text-gray-400">
                {new Date().toLocaleDateString()}
              </div>
              <div className="h-6 w-px bg-gray-300 dark:bg-gray-600"></div>
              <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                <Activity className="h-4 w-4" />
                <span className="hidden sm:inline">Real-time</span>
              </div>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 p-6 overflow-auto bg-gray-50 dark:bg-gray-900">
          <div className="max-w-7xl mx-auto">
            <div className="animate-fade-in">{renderContent()}</div>
          </div>
        </div>
      </div>
    </div>
  );
};
