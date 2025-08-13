'use client';

import { Badge } from '@epsx/ui';
import { Button } from '@epsx/ui';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@epsx/ui';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui';
import { fmtDateTime } from '@epsx/shared-utils/formatting';
import React, { useEffect, useState } from 'react';
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Line,
  LineChart,
  Pie,
  PieChart,
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';

interface AssignmentRuleData {
  ruleId: string;
  ruleName: string;
  conditions: string[];
  targetProfile: string;
  totalEvaluations: number;
  successfulAssignments: number;
  successRate: number;
  avgEvaluationTime: number;
  lastTriggered: string;
  isActive: boolean;
  priority: number;
}

interface OnboardingData {
  date: string;
  newUsers: number;
  autoAssignments: number;
  manualAssignments: number;
  assignmentRate: number;
  avgOnboardingTime: number;
}

interface ConflictData {
  conflictType: string;
  occurrences: number;
  resolutionStrategy: string;
  avgResolutionTime: number;
  severity: 'low' | 'medium' | 'high';
}

interface RulePerformanceData {
  ruleId: string;
  ruleName: string;
  evaluationTime: number;
  successRate: number;
  userCount: number;
}

const COLORS = [
  '#0088FE',
  '#00C49F',
  '#FFBB28',
  '#FF8042',
  '#8884D8',
  '#82ca9d',
];

const AssignmentAnalytics: React.FC = () => {
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState('7d');
  const [assignmentRules, setAssignmentRules] = useState<AssignmentRuleData[]>(
    []
  );
  const [onboardingData, setOnboardingData] = useState<OnboardingData[]>([]);
  const [conflictData, setConflictData] = useState<ConflictData[]>([]);
  const [rulePerformance, setRulePerformance] = useState<RulePerformanceData[]>(
    []
  );

  useEffect(() => {
    fetchAssignmentAnalytics();
  }, [timeRange]);

  const fetchAssignmentAnalytics = async () => {
    setLoading(true);
    try {
      await new Promise(resolve => setTimeout(resolve, 1000));

      setAssignmentRules([
        {
          ruleId: 'rule_001',
          ruleName: 'New User Bronze Assignment',
          conditions: ['user.registration_date < 24h', 'user.tier == null'],
          targetProfile: 'Bronze Package',
          totalEvaluations: 2340,
          successfulAssignments: 2156,
          successRate: 92.1,
          avgEvaluationTime: 45,
          lastTriggered: '2025-07-25T10:15:00Z',
          isActive: true,
          priority: 1,
        },
        {
          ruleId: 'rule_002',
          ruleName: 'Premium User Auto-Upgrade',
          conditions: [
            'user.payment_verified == true',
            'user.subscription_type == premium',
          ],
          targetProfile: 'Gold Package',
          totalEvaluations: 450,
          successfulAssignments: 445,
          successRate: 98.9,
          avgEvaluationTime: 120,
          lastTriggered: '2025-07-25T09:30:00Z',
          isActive: true,
          priority: 2,
        },
        {
          ruleId: 'rule_003',
          ruleName: 'Admin Role Assignment',
          conditions: ['user.department == IT', 'user.role_level >= manager'],
          targetProfile: 'Admin Profile',
          totalEvaluations: 89,
          successfulAssignments: 87,
          successRate: 97.8,
          avgEvaluationTime: 200,
          lastTriggered: '2025-07-25T08:45:00Z',
          isActive: true,
          priority: 3,
        },
        {
          ruleId: 'rule_004',
          ruleName: 'Trial User Limited Access',
          conditions: [
            'user.account_type == trial',
            'user.trial_days_remaining > 0',
          ],
          targetProfile: 'Trial Package',
          totalEvaluations: 1200,
          successfulAssignments: 980,
          successRate: 81.7,
          avgEvaluationTime: 35,
          lastTriggered: '2025-07-25T10:00:00Z',
          isActive: false,
          priority: 4,
        },
      ]);

      setOnboardingData([
        {
          date: '2025-07-19',
          newUsers: 145,
          autoAssignments: 132,
          manualAssignments: 13,
          assignmentRate: 91.0,
          avgOnboardingTime: 4.2,
        },
        {
          date: '2025-07-20',
          newUsers: 189,
          autoAssignments: 175,
          manualAssignments: 14,
          assignmentRate: 92.6,
          avgOnboardingTime: 3.8,
        },
        {
          date: '2025-07-21',
          newUsers: 156,
          autoAssignments: 138,
          manualAssignments: 18,
          assignmentRate: 88.5,
          avgOnboardingTime: 5.1,
        },
        {
          date: '2025-07-22',
          newUsers: 203,
          autoAssignments: 195,
          manualAssignments: 8,
          assignmentRate: 96.1,
          avgOnboardingTime: 3.5,
        },
        {
          date: '2025-07-23',
          newUsers: 167,
          autoAssignments: 152,
          manualAssignments: 15,
          assignmentRate: 91.0,
          avgOnboardingTime: 4.0,
        },
        {
          date: '2025-07-24',
          newUsers: 234,
          autoAssignments: 218,
          manualAssignments: 16,
          assignmentRate: 93.2,
          avgOnboardingTime: 3.7,
        },
        {
          date: '2025-07-25',
          newUsers: 198,
          autoAssignments: 184,
          manualAssignments: 14,
          assignmentRate: 92.9,
          avgOnboardingTime: 3.9,
        },
      ]);

      setConflictData([
        {
          conflictType: 'Multiple Profile Match',
          occurrences: 23,
          resolutionStrategy: 'Most Permissive',
          avgResolutionTime: 150,
          severity: 'medium',
        },
        {
          conflictType: 'Condition Overlap',
          occurrences: 15,
          resolutionStrategy: 'Priority Based',
          avgResolutionTime: 95,
          severity: 'low',
        },
        {
          conflictType: 'Permission Escalation',
          occurrences: 8,
          resolutionStrategy: 'Manual Review',
          avgResolutionTime: 1200,
          severity: 'high',
        },
        {
          conflictType: 'Circular Dependency',
          occurrences: 3,
          resolutionStrategy: 'Dependency Break',
          avgResolutionTime: 450,
          severity: 'high',
        },
      ]);

      setRulePerformance([
        {
          ruleId: 'rule_001',
          ruleName: 'New User Bronze',
          evaluationTime: 45,
          successRate: 92.1,
          userCount: 2156,
        },
        {
          ruleId: 'rule_002',
          ruleName: 'Premium Upgrade',
          evaluationTime: 120,
          successRate: 98.9,
          userCount: 445,
        },
        {
          ruleId: 'rule_003',
          ruleName: 'Admin Assignment',
          evaluationTime: 200,
          successRate: 97.8,
          userCount: 87,
        },
        {
          ruleId: 'rule_004',
          ruleName: 'Trial Limited',
          evaluationTime: 35,
          successRate: 81.7,
          userCount: 980,
        },
      ]);
    } catch (error) {
      console.error('Failed to fetch assignment analytics:', error);
    } finally {
      setLoading(false);
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'high':
        return 'bg-red-100 text-red-800';
      case 'medium':
        return 'bg-yellow-100 text-yellow-800';
      case 'low':
        return 'bg-green-100 text-green-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const calculateOverallSuccessRate = () => {
    const totalEvaluations = assignmentRules.reduce(
      (sum, rule) => sum + rule.totalEvaluations,
      0
    );
    const totalSuccessful = assignmentRules.reduce(
      (sum, rule) => sum + rule.successfulAssignments,
      0
    );
    return totalEvaluations > 0
      ? ((totalSuccessful / totalEvaluations) * 100).toFixed(1)
      : '0';
  };

  const toggleRuleStatus = async (ruleId: string) => {
    setAssignmentRules(prev =>
      prev.map(rule =>
        rule.ruleId === ruleId ? { ...rule, isActive: !rule.isActive } : rule
      )
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2">Loading assignment analytics...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Assignment Analytics</h1>
          <p className="text-gray-600">
            Auto-assignment rule performance and user onboarding insights
          </p>
        </div>
        <div className="flex gap-2">
          {['24h', '7d', '30d', '90d'].map(range => (
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
            <CardTitle className="text-sm font-medium">Active Rules</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {assignmentRules.filter(r => r.isActive).length}
            </div>
            <p className="text-xs text-gray-600">
              of {assignmentRules.length} total rules
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {calculateOverallSuccessRate()}%
            </div>
            <p className="text-xs text-gray-600">Overall assignment success</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Auto-Assignments
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {onboardingData
                .reduce((sum, item) => sum + item.autoAssignments, 0)
                .toLocaleString()}
            </div>
            <p className="text-xs text-gray-600">Last 7 days</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Conflicts</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {conflictData.reduce((sum, item) => sum + item.occurrences, 0)}
            </div>
            <p className="text-xs text-gray-600">
              {conflictData
                .filter(c => c.severity === 'high')
                .reduce((sum, item) => sum + item.occurrences, 0)}{' '}
              high priority
            </p>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="rules" className="space-y-4">
        <TabsList>
          <TabsTrigger value="rules">Assignment Rules</TabsTrigger>
          <TabsTrigger value="onboarding">User Onboarding</TabsTrigger>
          <TabsTrigger value="conflicts">Conflict Analysis</TabsTrigger>
          <TabsTrigger value="performance">Performance</TabsTrigger>
        </TabsList>

        <TabsContent value="rules" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Assignment Rules Management</CardTitle>
              <CardDescription>
                Auto-assignment rule performance and management
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left p-3">Rule</th>
                      <th className="text-left p-3">Target Profile</th>
                      <th className="text-left p-3">Evaluations</th>
                      <th className="text-left p-3">Success Rate</th>
                      <th className="text-left p-3">Avg Time</th>
                      <th className="text-left p-3">Last Triggered</th>
                      <th className="text-left p-3">Status</th>
                      <th className="text-left p-3">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {assignmentRules.map((rule, index) => (
                      <tr key={index} className="border-b hover:bg-gray-50">
                        <td className="p-3">
                          <div>
                            <div className="font-medium">{rule.ruleName}</div>
                            <div className="text-sm text-gray-600 space-y-1">
                              {rule.conditions.map((condition, i) => (
                                <div
                                  key={i}
                                  className="font-mono text-xs bg-gray-100 px-2 py-1 rounded"
                                >
                                  {condition}
                                </div>
                              ))}
                            </div>
                          </div>
                        </td>
                        <td className="p-3">
                          <Badge variant="outline">{rule.targetProfile}</Badge>
                        </td>
                        <td className="p-3">
                          <div>
                            <div className="font-medium">
                              {rule.totalEvaluations.toLocaleString()}
                            </div>
                            <div className="text-sm text-green-600">
                              {rule.successfulAssignments.toLocaleString()}{' '}
                              successful
                            </div>
                          </div>
                        </td>
                        <td className="p-3">
                          <div className="flex items-center gap-2">
                            <Progress
                              value={rule.successRate}
                              className="w-16"
                            />
                            <span className="text-sm">{rule.successRate}%</span>
                          </div>
                        </td>
                        <td className="p-3">{rule.avgEvaluationTime}ms</td>
                        <td className="p-3 text-sm text-gray-600">
                          {fmtDateTime(rule.lastTriggered)}
                        </td>
                        <td className="p-3">
                          <Badge
                            className={
                              rule.isActive
                                ? 'bg-green-100 text-green-800'
                                : 'bg-gray-100 text-gray-800'
                            }
                          >
                            {rule.isActive ? 'Active' : 'Inactive'}
                          </Badge>
                        </td>
                        <td className="p-3">
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => toggleRuleStatus(rule.ruleId)}
                          >
                            {rule.isActive ? 'Disable' : 'Enable'}
                          </Button>
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
                <CardTitle>Rule Success Rates</CardTitle>
                <CardDescription>Assignment success by rule</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={assignmentRules}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                      dataKey="ruleName"
                      angle={-45}
                      textAnchor="end"
                      height={80}
                    />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="successRate" fill="#8884d8" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Evaluation Volume</CardTitle>
                <CardDescription>Total evaluations per rule</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={assignmentRules}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="totalEvaluations"
                      label={({ ruleName, percent }) =>
                        `${ruleName.split(' ')[0]} ${(percent * 100).toFixed(0)}%`
                      }
                    >
                      {assignmentRules.map((entry, index) => (
                        <Cell
                          key={`cell-${index}`}
                          fill={COLORS[index % COLORS.length]}
                        />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="onboarding" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>User Onboarding Trends</CardTitle>
                <CardDescription>
                  New user registration and assignment patterns
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={onboardingData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" />
                    <YAxis />
                    <Tooltip />
                    <Legend />
                    <Line
                      type="monotone"
                      dataKey="newUsers"
                      stroke="#8884d8"
                      name="New Users"
                    />
                    <Line
                      type="monotone"
                      dataKey="autoAssignments"
                      stroke="#82ca9d"
                      name="Auto Assignments"
                    />
                    <Line
                      type="monotone"
                      dataKey="manualAssignments"
                      stroke="#ffc658"
                      name="Manual Assignments"
                    />
                  </LineChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Assignment Rate Trends</CardTitle>
                <CardDescription>
                  Percentage of successful auto-assignments
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={onboardingData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" />
                    <YAxis domain={[75, 100]} />
                    <Tooltip />
                    <Line
                      type="monotone"
                      dataKey="assignmentRate"
                      stroke="#ff7300"
                      name="Assignment Rate %"
                    />
                  </LineChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Onboarding Performance Details</CardTitle>
              <CardDescription>
                Daily breakdown of user onboarding metrics
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left p-3">Date</th>
                      <th className="text-left p-3">New Users</th>
                      <th className="text-left p-3">Auto Assignments</th>
                      <th className="text-left p-3">Manual Assignments</th>
                      <th className="text-left p-3">Assignment Rate</th>
                      <th className="text-left p-3">Avg Onboarding Time</th>
                    </tr>
                  </thead>
                  <tbody>
                    {onboardingData.map((day, index) => (
                      <tr key={index} className="border-b hover:bg-gray-50">
                        <td className="p-3 font-medium">{day.date}</td>
                        <td className="p-3">{day.newUsers}</td>
                        <td className="p-3 text-green-600">
                          {day.autoAssignments}
                        </td>
                        <td className="p-3 text-yellow-600">
                          {day.manualAssignments}
                        </td>
                        <td className="p-3">
                          <div className="flex items-center gap-2">
                            <Progress
                              value={day.assignmentRate}
                              className="w-16"
                            />
                            <span className="text-sm">
                              {day.assignmentRate}%
                            </span>
                          </div>
                        </td>
                        <td className="p-3">{day.avgOnboardingTime} min</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="conflicts" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Conflict Types</CardTitle>
                <CardDescription>
                  Distribution of assignment conflicts
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={conflictData}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="occurrences"
                      label={({ conflictType, percent }) =>
                        `${conflictType} ${(percent * 100).toFixed(0)}%`
                      }
                    >
                      {conflictData.map((entry, index) => (
                        <Cell
                          key={`cell-${index}`}
                          fill={COLORS[index % COLORS.length]}
                        />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Resolution Times</CardTitle>
                <CardDescription>
                  Average time to resolve conflicts
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={conflictData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                      dataKey="conflictType"
                      angle={-45}
                      textAnchor="end"
                      height={80}
                    />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="avgResolutionTime" fill="#ff7300" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Conflict Analysis Details</CardTitle>
              <CardDescription>
                Detailed breakdown of assignment conflicts and resolutions
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left p-3">Conflict Type</th>
                      <th className="text-left p-3">Occurrences</th>
                      <th className="text-left p-3">Resolution Strategy</th>
                      <th className="text-left p-3">Avg Resolution Time</th>
                      <th className="text-left p-3">Severity</th>
                    </tr>
                  </thead>
                  <tbody>
                    {conflictData.map((conflict, index) => (
                      <tr key={index} className="border-b hover:bg-gray-50">
                        <td className="p-3 font-medium">
                          {conflict.conflictType}
                        </td>
                        <td className="p-3">{conflict.occurrences}</td>
                        <td className="p-3">{conflict.resolutionStrategy}</td>
                        <td className="p-3">{conflict.avgResolutionTime}ms</td>
                        <td className="p-3">
                          <Badge
                            className={getSeverityColor(conflict.severity)}
                          >
                            {conflict.severity}
                          </Badge>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="performance" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Rule Performance Matrix</CardTitle>
                <CardDescription>
                  Success rate vs evaluation time
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <ScatterChart data={rulePerformance}>
                    <CartesianGrid />
                    <XAxis
                      dataKey="evaluationTime"
                      name="Evaluation Time (ms)"
                    />
                    <YAxis dataKey="successRate" name="Success Rate %" />
                    <Tooltip cursor={{ strokeDasharray: '3 3' }} />
                    <Scatter dataKey="successRate" fill="#8884d8" />
                  </ScatterChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>User Impact</CardTitle>
                <CardDescription>
                  Number of users affected by each rule
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={rulePerformance}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                      dataKey="ruleName"
                      angle={-45}
                      textAnchor="end"
                      height={80}
                    />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="userCount" fill="#82ca9d" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Performance Optimization Recommendations</CardTitle>
              <CardDescription>
                Suggestions for improving assignment rule performance
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                <h4 className="font-medium text-green-800">High Performance</h4>
                <p className="text-sm text-green-700">
                  Premium User Auto-Upgrade rule shows excellent 98.9% success
                  rate. Consider using this as a template for other rules.
                </p>
              </div>
              <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
                <h4 className="font-medium text-yellow-800">
                  Optimization Needed
                </h4>
                <p className="text-sm text-yellow-700">
                  Trial User Limited Access rule has 81.7% success rate. Review
                  conditions for potential improvements.
                </p>
              </div>
              <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
                <h4 className="font-medium text-blue-800">
                  Performance Insight
                </h4>
                <p className="text-sm text-blue-700">
                  Average evaluation time: 100ms. Consider optimizing rules with{' '}
                  {'>'}150ms evaluation time.
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default AssignmentAnalytics;
