'use client';

import React, { useState, useEffect } from 'react';
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  Area,
  AreaChart
} from 'recharts';
import { 
  Calendar as _Calendar,
  TrendingUp,
  Users,
  DollarSign,
  Activity,
  Download,
  Filter as _Filter,
  RefreshCcw,
  AlertTriangle,
  CheckCircle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { FormField, Select } from '@/components/ui/form-components';
import { getModuleUsageAnalytics as _getModuleUsageAnalytics } from '@epsx/server-actions';
import { fmtCurrency } from '@epsx/shared-utils/formatting';
// import { useModuleAuth } from '@/auth/module-ctx'; // Removed - using OIDC auth
import { toast } from 'react-hot-toast';

interface UsageMetrics {
  totalRequests: number;
  totalUsers: number;
  totalRevenue: number;
  averageResponseTime: number;
  errorRate: number;
  activeApiKeys: number;
}

interface TimeSeriesData {
  date: string;
  requests: number;
  users: number;
  revenue: number;
  errors: number;
}

interface ModuleUsageData {
  moduleName: string;
  requests: number;
  users: number;
  revenue: number;
  quota: number;
  quotaUsed: number;
}

interface BillingData {
  currentPeriod: {
    startDate: string;
    endDate: string;
    totalCost: number;
    totalRequests: number;
  };
  upcomingInvoice: {
    amount: number;
    dueDate: string;
    status: string;
  };
  costBreakdown: Array<{
    module: string;
    cost: number;
    requests: number;
  }>;
}

const COLORS = ['#3B82F6', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6', '#06B6D4'];

export const ModuleAnalyticsDashboard: React.FC = () => {
  // TODO: Implement proper module auth check with OIDC
  const hasModuleAccess = () => true;
  const canPerformAction = () => true;
  const [loading, setLoading] = useState(true);
  const [dateRange, setDateRange] = useState('7d');
  const [selectedModule, setSelectedModule] = useState('all');
  const [activeTab, setActiveTab] = useState<'overview' | 'usage' | 'billing' | 'performance'>('overview');
  
  const [metrics, setMetrics] = useState<UsageMetrics>({
    totalRequests: 0,
    totalUsers: 0,
    totalRevenue: 0,
    averageResponseTime: 0,
    errorRate: 0,
    activeApiKeys: 0
  });
  
  const [timeSeriesData, setTimeSeriesData] = useState<TimeSeriesData[]>([]);
  const [moduleUsageData, setModuleUsageData] = useState<ModuleUsageData[]>([]);
  const [billingData, setBillingData] = useState<BillingData | null>(null);

  // Mock data for demonstration
  useEffect(() => {
    const fetchAnalytics = async () => {
      setLoading(true);
      try {
        // In a real implementation, this would call the actual API
        // const result = await getModuleUsageAnalytics({
        //   startDate: getStartDate(dateRange),
        //   endDate: new Date().toISOString(),
        //   groupBy: 'day'
        // });

        // Mock data for demonstration
        setMetrics({
          totalRequests: 245678,
          totalUsers: 1247,
          totalRevenue: 15420.50,
          averageResponseTime: 125,
          errorRate: 0.8,
          activeApiKeys: 89
        });

        setTimeSeriesData([
          { date: '2024-01-20', requests: 15420, users: 245, revenue: 1240.50, errors: 12 },
          { date: '2024-01-21', requests: 18650, users: 278, revenue: 1456.75, errors: 8 },
          { date: '2024-01-22', requests: 22100, users: 312, revenue: 1687.25, errors: 15 },
          { date: '2024-01-23', requests: 19800, users: 289, revenue: 1521.00, errors: 11 },
          { date: '2024-01-24', requests: 25400, users: 356, revenue: 1892.30, errors: 18 },
          { date: '2024-01-25', requests: 28900, users: 398, revenue: 2156.70, errors: 22 },
          { date: '2024-01-26', requests: 31200, users: 423, revenue: 2487.00, errors: 19 }
        ]);

        setModuleUsageData([
          { moduleName: 'Stock Ranking', requests: 125400, users: 523, revenue: 8420.50, quota: 150000, quotaUsed: 125400 },
          { moduleName: 'Market Data', requests: 89650, users: 412, revenue: 4200.25, quota: 100000, quotaUsed: 89650 },
          { moduleName: 'Portfolio Analysis', requests: 45200, users: 187, revenue: 1890.75, quota: 60000, quotaUsed: 45200 },
          { moduleName: 'Trading Signals', requests: 28100, users: 125, revenue: 909.00, quota: 40000, quotaUsed: 28100 }
        ]);

        setBillingData({
          currentPeriod: {
            startDate: '2024-01-01',
            endDate: '2024-01-31',
            totalCost: 15420.50,
            totalRequests: 288350
          },
          upcomingInvoice: {
            amount: 16250.75,
            dueDate: '2024-02-01',
            status: 'pending'
          },
          costBreakdown: [
            { module: 'Stock Ranking', cost: 8420.50, requests: 125400 },
            { module: 'Market Data', cost: 4200.25, requests: 89650 },
            { module: 'Portfolio Analysis', cost: 1890.75, requests: 45200 },
            { module: 'Trading Signals', cost: 909.00, requests: 28100 }
          ]
        });

      } catch (error) {
        console.error('Failed to fetch analytics:', error);
        toast.error('Failed to load analytics data');
      } finally {
        setLoading(false);
      }
    };

    fetchAnalytics();
  }, [dateRange, selectedModule]);


  const formatNumber = (num: number) => {
    return new Intl.NumberFormat('en-US').format(num);
  };

  const getQuotaColor = (used: number, total: number) => {
    const percentage = (used / total) * 100;
    if (percentage >= 90) return 'text-red-600 bg-red-50';
    if (percentage >= 75) return 'text-yellow-600 bg-yellow-50';
    return 'text-green-600 bg-green-50';
  };

  if (!hasModuleAccess('analytics') || !canPerformAction('analytics', 'read')) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-yellow-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Access Restricted</h3>
          <p className="text-gray-600">You don&apos;t have permission to view analytics data.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Module Analytics</h1>
          <p className="text-gray-600">Monitor usage, performance, and billing across all modules</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <FormField label="">
            <Select value={selectedModule} onChange={(e) => setSelectedModule(e.target.value)}>
              <option value="all">All Modules</option>
              <option value="stock-ranking">Stock Ranking</option>
              <option value="market-data">Market Data</option>
              <option value="portfolio-analysis">Portfolio Analysis</option>
              <option value="trading-signals">Trading Signals</option>
            </Select>
          </FormField>
          
          <FormField label="">
            <Select value={dateRange} onChange={(e) => setDateRange(e.target.value)}>
              <option value="1d">Last 24 hours</option>
              <option value="7d">Last 7 days</option>
              <option value="30d">Last 30 days</option>
              <option value="90d">Last 90 days</option>
            </Select>
          </FormField>
          
          <Button variant="outline" size="sm">
            <RefreshCcw className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          
          <Button variant="outline" size="sm">
            <Download className="w-4 h-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { id: 'overview', name: 'Overview', icon: Activity },
            { id: 'usage', name: 'Usage', icon: BarChart },
            { id: 'billing', name: 'Billing', icon: DollarSign },
            { id: 'performance', name: 'Performance', icon: TrendingUp }
          ].map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`group inline-flex items-center py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <Icon className="w-4 h-4 mr-2" />
                {tab.name}
              </button>
            );
          })}
        </nav>
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      ) : (
        <>
          {/* Overview Tab */}
          {activeTab === 'overview' && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-6 gap-4">
                {[
                  { label: 'Total Requests', value: formatNumber(metrics.totalRequests), icon: Activity, color: 'blue' },
                  { label: 'Active Users', value: formatNumber(metrics.totalUsers), icon: Users, color: 'green' },
                  { label: 'Revenue', value: fmtCurrency(metrics.totalRevenue), icon: DollarSign, color: 'yellow' },
                  { label: 'Avg Response', value: `${metrics.averageResponseTime}ms`, icon: TrendingUp, color: 'purple' },
                  { label: 'Error Rate', value: `${metrics.errorRate}%`, icon: AlertTriangle, color: 'red' },
                  { label: 'Active Keys', value: formatNumber(metrics.activeApiKeys), icon: CheckCircle, color: 'indigo' }
                ].map((metric, index) => {
                  const Icon = metric.icon;
                  return (
                    <div key={index} className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-4">
                      <div className="flex items-center">
                        <Icon className={`w-5 h-5 text-${metric.color}-600`} />
                        <span className="ml-2 text-sm font-medium text-gray-600">{metric.label}</span>
                      </div>
                      <div className="mt-2">
                        <span className="text-2xl font-bold text-gray-900">{metric.value}</span>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Usage Trends Chart */}
              <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Usage Trends</h3>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={timeSeriesData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" />
                    <YAxis />
                    <Tooltip />
                    <Area type="monotone" dataKey="requests" stackId="1" stroke="#3B82F6" fill="#3B82F6" fillOpacity={0.6} />
                    <Area type="monotone" dataKey="users" stackId="2" stroke="#10B981" fill="#10B981" fillOpacity={0.6} />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>
          )}

          {/* Usage Tab */}
          {activeTab === 'usage' && (
            <div className="space-y-6">
              {/* Module Usage Table */}
              <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg overflow-hidden">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h3 className="text-lg font-semibold text-gray-900">Module Usage Details</h3>
                </div>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Module</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Requests</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Users</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Revenue</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Quota Usage</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                      {moduleUsageData.map((module, index) => (
                        <tr key={index}>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div className="font-medium text-gray-900">{module.moduleName}</div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                            {formatNumber(module.requests)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                            {formatNumber(module.users)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                            {fmtCurrency(module.revenue)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div className="flex items-center">
                              <div className="flex-1">
                                <div className="w-full bg-gray-200 rounded-full h-2">
                                  <div 
                                    className={`h-2 rounded-full ${getQuotaColor(module.quotaUsed, module.quota)}`}
                                    style={{ width: `${(module.quotaUsed / module.quota) * 100}%` }}
                                  ></div>
                                </div>
                              </div>
                              <span className="ml-2 text-sm text-gray-600">
                                {formatNumber(module.quotaUsed)} / {formatNumber(module.quota)}
                              </span>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>

              {/* Usage Distribution Chart */}
              <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Usage Distribution</h3>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={moduleUsageData}
                      cx="50%"
                      cy="50%"
                      outerRadius={80}
                      dataKey="requests"
                      nameKey="moduleName"
                    >
                      {moduleUsageData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </div>
          )}

          {/* Billing Tab */}
          {activeTab === 'billing' && billingData && (
            <div className="space-y-6">
              {/* Billing Summary */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">Current Period</h3>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-gray-600">Total Cost:</span>
                      <span className="font-semibold">{fmtCurrency(billingData.currentPeriod.totalCost)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Total Requests:</span>
                      <span className="font-semibold">{formatNumber(billingData.currentPeriod.totalRequests)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Period:</span>
                      <span className="text-sm text-gray-500">
                        {new Date(billingData.currentPeriod.startDate).toLocaleDateString()} - 
                        {new Date(billingData.currentPeriod.endDate).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                </div>

                <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">Upcoming Invoice</h3>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-gray-600">Amount:</span>
                      <span className="font-semibold text-green-600">{fmtCurrency(billingData.upcomingInvoice.amount)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Due Date:</span>
                      <span className="font-semibold">{new Date(billingData.upcomingInvoice.dueDate).toLocaleDateString()}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Status:</span>
                      <span className="px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded-full">
                        {billingData.upcomingInvoice.status}
                      </span>
                    </div>
                  </div>
                </div>

                <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">Cost Per Request</h3>
                  <div className="text-3xl font-bold text-blue-600">
                    ${((billingData.currentPeriod.totalCost / billingData.currentPeriod.totalRequests) * 1000).toFixed(3)}
                  </div>
                  <p className="text-sm text-gray-600">per 1K requests</p>
                </div>
              </div>

              {/* Cost Breakdown */}
              <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Cost Breakdown by Module</h3>
                <div className="space-y-4">
                  {billingData.costBreakdown.map((item, index) => (
                    <div key={index} className="flex items-center justify-between p-4 border rounded">
                      <div>
                        <h4 className="font-medium text-gray-900">{item.module}</h4>
                        <p className="text-sm text-gray-600">{formatNumber(item.requests)} requests</p>
                      </div>
                      <div className="text-right">
                        <div className="font-semibold text-gray-900">{fmtCurrency(item.cost)}</div>
                        <div className="text-sm text-gray-600">
                          ${(item.cost / item.requests * 1000).toFixed(3)}/1K
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Performance Tab */}
          {activeTab === 'performance' && (
            <div className="space-y-6">
              {/* Response Time Chart */}
              <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Response Time Trends</h3>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={timeSeriesData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" />
                    <YAxis />
                    <Tooltip />
                    <Line type="monotone" dataKey="requests" stroke="#3B82F6" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </div>

              {/* Error Rate Chart */}
              <div className="bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Error Trends</h3>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={timeSeriesData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="errors" fill="#EF4444" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};