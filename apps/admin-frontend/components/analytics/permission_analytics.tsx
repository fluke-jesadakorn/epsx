'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, BarChart, Bar, PieChart, Pie, Cell } from 'recharts';
import { formatDate, formatRelativeTime } from '@/lib/utils';

// Format date and time utility function
const fmtDateTime = (date: string | Date): string => {
  return formatDate(date) + ' ' + new Date(date).toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit'
  });
};

interface PermissionUsageData {
  permission: string;
  resource: string;
  usageCount: number;
  successRate: number;
  avgResponseTime: number;
  lastUsed: string;
  category: string;
}

interface UserBehaviorData {
  userId: string;
  username: string;
  permissionRequests: number;
  deniedRequests: number;
  successRate: number;
  mostUsedPermissions: string[];
  riskScore: number;
  lastActivity: string;
}

interface PermissionTrendData {
  date: string;
  totalRequests: number;
  successfulRequests: number;
  deniedRequests: number;
  avgResponseTime: number;
}

interface CostAnalysisData {
  permissionProfile: string;
  userCount: number;
  monthlyRequests: number;
  costPerRequest: number;
  totalCost: number;
  efficiency: number;
}

const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884D8'];

const PermissionAnalytics: React.FC = () => {
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState('7d');
  const [permissionUsage, setPermissionUsage] = useState<PermissionUsageData[]>([]);
  const [userBehavior, setUserBehavior] = useState<UserBehaviorData[]>([]);
  const [permissionTrends, setPermissionTrends] = useState<PermissionTrendData[]>([]);
  const [costAnalysis, setCostAnalysis] = useState<CostAnalysisData[]>([]);

  useEffect(() => {
    fetchAnalyticsData();
  }, [timeRange]);

  const fetchAnalyticsData = async () => {
    setLoading(true);
    try {
      // Mock data for demonstration
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setPermissionUsage([
        {
          permission: 'read:posts',
          resource: 'posts',
          usageCount: 15420,
          successRate: 98.5,
          avgResponseTime: 45,
          lastUsed: '2025-07-25T10:30:00Z',
          category: 'Content'
        },
        {
          permission: 'write:comments',
          resource: 'comments',
          usageCount: 8760,
          successRate: 94.2,
          avgResponseTime: 120,
          lastUsed: '2025-07-25T10:25:00Z',
          category: 'Content'
        },
        {
          permission: 'admin:manage_users',
          resource: 'users',
          usageCount: 1230,
          successRate: 99.8,
          avgResponseTime: 200,
          lastUsed: '2025-07-25T09:45:00Z',
          category: 'Administration'
        },
        {
          permission: 'delete:posts',
          resource: 'posts',
          usageCount: 450,
          successRate: 88.9,
          avgResponseTime: 300,
          lastUsed: '2025-07-25T08:15:00Z',
          category: 'Content'
        }
      ]);

      setUserBehavior([
        {
          userId: 'user_123',
          username: 'john.doe@example.com',
          permissionRequests: 1840,
          deniedRequests: 23,
          successRate: 98.7,
          mostUsedPermissions: ['read:posts', 'write:comments', 'read:profile'],
          riskScore: 2.1,
          lastActivity: '2025-07-25T10:30:00Z'
        },
        {
          userId: 'user_456',
          username: 'admin@example.com',
          permissionRequests: 890,
          deniedRequests: 5,
          successRate: 99.4,
          mostUsedPermissions: ['admin:manage_users', 'admin:view_analytics', 'read:system_logs'],
          riskScore: 1.2,
          lastActivity: '2025-07-25T10:15:00Z'
        }
      ]);

      setPermissionTrends([
        { date: '2025-07-19', totalRequests: 18500, successfulRequests: 17820, deniedRequests: 680, avgResponseTime: 95 },
        { date: '2025-07-20', totalRequests: 19200, successfulRequests: 18640, deniedRequests: 560, avgResponseTime: 89 },
        { date: '2025-07-21', totalRequests: 17800, successfulRequests: 17230, deniedRequests: 570, avgResponseTime: 102 },
        { date: '2025-07-22', totalRequests: 20100, successfulRequests: 19450, deniedRequests: 650, avgResponseTime: 87 },
        { date: '2025-07-23', totalRequests: 18900, successfulRequests: 18340, deniedRequests: 560, avgResponseTime: 91 },
        { date: '2025-07-24', totalRequests: 21300, successfulRequests: 20680, deniedRequests: 620, avgResponseTime: 84 },
        { date: '2025-07-25', totalRequests: 16800, successfulRequests: 16290, deniedRequests: 510, avgResponseTime: 88 }
      ]);

      setCostAnalysis([
        {
          permissionProfile: 'Bronze Package',
          userCount: 1250,
          monthlyRequests: 450000,
          costPerRequest: 0.001,
          totalCost: 450,
          efficiency: 85.2
        },
        {
          permissionProfile: 'Silver Package',
          userCount: 680,
          monthlyRequests: 890000,
          costPerRequest: 0.0008,
          totalCost: 712,
          efficiency: 92.1
        },
        {
          permissionProfile: 'Gold Package',
          userCount: 320,
          monthlyRequests: 1200000,
          costPerRequest: 0.0006,
          totalCost: 720,
          efficiency: 96.8
        }
      ]);

    } catch (error) {
      console.error('Failed to fetch analytics data:', error);
    } finally {
      setLoading(false);
    }
  };

  const getRiskBadgeColor = (score: number) => {
    if (score < 2) return 'bg-green-100 text-green-800';
    if (score < 5) return 'bg-yellow-100 text-yellow-800';
    return 'bg-red-100 text-red-800';
  };


  const calculateSuccessRate = (data: PermissionTrendData[]) => {
    const totalRequests = data.reduce((sum, item) => sum + item.totalRequests, 0);
    const successfulRequests = data.reduce((sum, item) => sum + item.successfulRequests, 0);
    return totalRequests > 0 ? (successfulRequests / totalRequests * 100).toFixed(1) : '0';
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2">Loading analytics...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Permission Analytics</h1>
          <p className="text-gray-600">Real-time insights into permission usage and system performance</p>
        </div>
        <div className="flex gap-2">
          {['24h', '7d', '30d', '90d'].map((range) => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              className={`px-3 py-1 rounded-lg text-sm ${
                timeRange === range
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
              }`}
            >
              {range}
            </button>
          ))}
        </div>
      </div>

      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Requests</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {permissionTrends.reduce((sum, item) => sum + item.totalRequests, 0).toLocaleString()}
            </div>
            <p className="text-xs text-gray-600">
              Success Rate: {calculateSuccessRate(permissionTrends)}%
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Permissions</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{permissionUsage.length}</div>
            <p className="text-xs text-gray-600">Across {new Set(permissionUsage.map(p => p.resource)).size} resources</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Response Time</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {Math.round(permissionTrends.reduce((sum, item) => sum + item.avgResponseTime, 0) / permissionTrends.length)}ms
            </div>
            <p className="text-xs text-gray-600">7-day average</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">High Risk Users</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {userBehavior.filter(u => u.riskScore > 5).length}
            </div>
            <p className="text-xs text-gray-600">Require attention</p>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="usage" className="space-y-4">
        <TabsList>
          <TabsTrigger value="usage">Permission Usage</TabsTrigger>
          <TabsTrigger value="behavior">User Behavior</TabsTrigger>
          <TabsTrigger value="trends">Trends</TabsTrigger>
          <TabsTrigger value="cost">Cost Analysis</TabsTrigger>
        </TabsList>

        <TabsContent value="usage" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Permission Usage Distribution</CardTitle>
                <CardDescription>Most frequently used permissions</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={permissionUsage}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="permission" angle={-45} textAnchor="end" height={80} />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="usageCount" fill="#8884d8" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Permission Categories</CardTitle>
                <CardDescription>Usage by category</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={permissionUsage.reduce((acc, item) => {
                        const existing = acc.find(a => a.category === item.category);
                        if (existing) {
                          existing.value += item.usageCount;
                        } else {
                          acc.push({ category: item.category, value: item.usageCount });
                        }
                        return acc;
                      }, [] as {category: string, value: number}[])}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="value"
                      label={({ category, percent }) => `${category} ${(percent * 100).toFixed(0)}%`}
                    >
                      {permissionUsage.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Permission Performance Details</CardTitle>
              <CardDescription>Detailed performance metrics for each permission</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left p-2">Permission</th>
                      <th className="text-left p-2">Resource</th>
                      <th className="text-left p-2">Usage Count</th>
                      <th className="text-left p-2">Success Rate</th>
                      <th className="text-left p-2">Avg Response</th>
                      <th className="text-left p-2">Last Used</th>
                    </tr>
                  </thead>
                  <tbody>
                    {permissionUsage.map((permission) => (
                      <tr key={`${permission.permission}-${permission.resource}`} className="border-b hover:bg-gray-50">
                        <td className="p-2 font-mono text-sm">{permission.permission}</td>
                        <td className="p-2">{permission.resource}</td>
                        <td className="p-2">{permission.usageCount.toLocaleString()}</td>
                        <td className="p-2">
                          <div className="flex items-center gap-2">
                            <Progress value={permission.successRate} className="w-16" />
                            <span className="text-sm">{permission.successRate}%</span>
                          </div>
                        </td>
                        <td className="p-2">{permission.avgResponseTime}ms</td>
                        <td className="p-2 text-sm text-gray-600">
                          {fmtDateTime(permission.lastUsed)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="behavior" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>User Behavior Analysis</CardTitle>
              <CardDescription>User activity patterns and risk assessment</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left p-3">User</th>
                      <th className="text-left p-3">Requests</th>
                      <th className="text-left p-3">Success Rate</th>
                      <th className="text-left p-3">Most Used Permissions</th>
                      <th className="text-left p-3">Risk Score</th>
                      <th className="text-left p-3">Last Activity</th>
                    </tr>
                  </thead>
                  <tbody>
                    {userBehavior.map((user) => (
                      <tr key={user.userId} className="border-b hover:bg-gray-50">
                        <td className="p-3">
                          <div>
                            <div className="font-medium">{user.username}</div>
                            <div className="text-sm text-gray-600">{user.userId}</div>
                          </div>
                        </td>
                        <td className="p-3">
                          <div>
                            <div className="font-medium">{user.permissionRequests.toLocaleString()}</div>
                            <div className="text-sm text-red-600">{user.deniedRequests} denied</div>
                          </div>
                        </td>
                        <td className="p-3">
                          <div className="flex items-center gap-2">
                            <Progress value={user.successRate} className="w-16" />
                            <span className="text-sm">{user.successRate}%</span>
                          </div>
                        </td>
                        <td className="p-3">
                          <div className="flex flex-wrap gap-1">
                            {user.mostUsedPermissions.slice(0, 2).map((perm) => (
                              <Badge key={perm} variant="secondary" className="text-xs">
                                {perm}
                              </Badge>
                            ))}
                            {user.mostUsedPermissions.length > 2 && (
                              <Badge variant="outline" className="text-xs">
                                +{user.mostUsedPermissions.length - 2}
                              </Badge>
                            )}
                          </div>
                        </td>
                        <td className="p-3">
                          <Badge className={getRiskBadgeColor(user.riskScore)}>
                            {user.riskScore.toFixed(1)}
                          </Badge>
                        </td>
                        <td className="p-3 text-sm text-gray-600">
                          {fmtDateTime(user.lastActivity)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="trends" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Permission Request Trends</CardTitle>
              <CardDescription>Historical permission usage patterns</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={400}>
                <LineChart data={permissionTrends}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="date" />
                  <YAxis />
                  <Tooltip />
                  <Legend />
                  <Line type="monotone" dataKey="totalRequests" stroke="#8884d8" name="Total Requests" />
                  <Line type="monotone" dataKey="successfulRequests" stroke="#82ca9d" name="Successful" />
                  <Line type="monotone" dataKey="deniedRequests" stroke="#ffc658" name="Denied" />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Response Time Trends</CardTitle>
              <CardDescription>Average response time over time</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={permissionTrends}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="date" />
                  <YAxis />
                  <Tooltip />
                  <Line type="monotone" dataKey="avgResponseTime" stroke="#ff7300" name="Avg Response Time (ms)" />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="cost" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Cost Analysis & Optimization</CardTitle>
              <CardDescription>Permission profile cost analysis and recommendations</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left p-3">Permission Profile</th>
                      <th className="text-left p-3">Users</th>
                      <th className="text-left p-3">Monthly Requests</th>
                      <th className="text-left p-3">Cost/Request</th>
                      <th className="text-left p-3">Total Cost</th>
                      <th className="text-left p-3">Efficiency</th>
                    </tr>
                  </thead>
                  <tbody>
                    {costAnalysis.map((item, index) => (
                      <tr key={index} className="border-b hover:bg-gray-50">
                        <td className="p-3 font-medium">{item.permissionProfile}</td>
                        <td className="p-3">{item.userCount.toLocaleString()}</td>
                        <td className="p-3">{item.monthlyRequests.toLocaleString()}</td>
                        <td className="p-3">${item.costPerRequest.toFixed(4)}</td>
                        <td className="p-3 font-medium">${item.totalCost.toFixed(2)}</td>
                        <td className="p-3">
                          <div className="flex items-center gap-2">
                            <Progress value={item.efficiency} className="w-16" />
                            <span className="text-sm">{item.efficiency}%</span>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Cost Distribution</CardTitle>
                <CardDescription>Cost breakdown by permission profile</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={costAnalysis}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="totalCost"
                      label={({ permissionProfile, percent }) => `${permissionProfile} ${(percent * 100).toFixed(0)}%`}
                    >
                      {costAnalysis.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip formatter={(value) => [`$${value}`, 'Cost']} />
                  </PieChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Optimization Recommendations</CardTitle>
                <CardDescription>Cost reduction opportunities</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
                  <h4 className="font-medium text-green-800">High Efficiency</h4>
                  <p className="text-sm text-green-700">
                    Gold Package shows 96.8% efficiency. Consider promoting Bronze users with high usage.
                  </p>
                </div>
                <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                  <h4 className="font-medium text-yellow-800">Optimization Opportunity</h4>
                  <p className="text-sm text-yellow-700">
                    Bronze Package has 85.2% efficiency. Review permission usage patterns for optimization.
                  </p>
                </div>
                <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
                  <h4 className="font-medium text-blue-800">Cost Savings</h4>
                  <p className="text-sm text-blue-700">
                    Estimated monthly savings: $156 by optimizing permission assignments.
                  </p>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default PermissionAnalytics;