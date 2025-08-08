"use client";

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Lightbulb, 
  TrendingUp, 
  Shield, 
  Users, 
  Clock,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Sparkles
} from 'lucide-react';

interface User {
  id: string;
  email: string;
  currentPermissions: string[];
  role: string;
  department: string;
}

interface PermissionRecommendation {
  id: string;
  type: 'add' | 'remove' | 'upgrade' | 'temporary';
  permission: string;
  resource: string;
  action: string;
  confidence: number;
  reasoning: string;
  category: 'security' | 'efficiency' | 'compliance' | 'role-based';
  impact: 'low' | 'medium' | 'high';
  similarUsers: string[];
  estimatedBenefit: string;
  risks: string[];
  prerequisites: string[];
}

interface SmartRecommendationsData {
  recommendations: PermissionRecommendation[];
  insights: {
    overPrivilegedUsers: number;
    underPrivilegedUsers: number;
    obsoletePermissions: number;
    securityRisks: number;
  };
  trends: {
    category: string;
    count: number;
    change: number;
  }[];
}

interface SmartPermissionRecommendationsProps {
  user: User;
  className?: string;
  onApplyRecommendation?: (recommendation: PermissionRecommendation) => Promise<void>;
}

export function SmartPermissionRecommendations({
  user,
  className = '',
  onApplyRecommendation
}: SmartPermissionRecommendationsProps) {
  const [data, setData] = useState<SmartRecommendationsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'all' | 'security' | 'efficiency' | 'compliance'>('all');
  const [applyingIds, setApplyingIds] = useState<Set<string>>(new Set());

  const loadRecommendations = async () => {
    setLoading(true);
    try {
      // Mock AI-powered recommendation data
      const mockData: SmartRecommendationsData = {
        recommendations: [
          {
            id: '1',
            type: 'add',
            permission: 'analytics:read',
            resource: 'dashboard',
            action: 'view',
            confidence: 92,
            reasoning: 'User frequently requests analytics reports and works in similar role as 85% of users who have this permission',
            category: 'efficiency',
            impact: 'medium',
            similarUsers: ['jane@example.com', 'bob@example.com', 'alice@example.com'],
            estimatedBenefit: 'Reduce approval wait time by 75%, increase productivity',
            risks: [],
            prerequisites: []
          },
          {
            id: '2',
            type: 'remove',
            permission: 'admin:delete',
            resource: 'system',
            action: 'delete',
            confidence: 88,
            reasoning: 'Permission not used in 90+ days and exceeds role requirements',
            category: 'security',
            impact: 'high',
            similarUsers: [],
            estimatedBenefit: 'Reduce security risk exposure by 40%',
            risks: ['High privilege level', 'Unused for extended period'],
            prerequisites: []
          },
          {
            id: '3',
            type: 'upgrade',
            permission: 'reports:basic',
            resource: 'reports',
            action: 'generate',
            confidence: 85,
            reasoning: 'User role typically requires advanced reporting capabilities',
            category: 'role-based',
            impact: 'medium',
            similarUsers: ['manager1@example.com', 'manager2@example.com'],
            estimatedBenefit: 'Enable advanced reporting features needed for role',
            risks: [],
            prerequisites: ['Complete advanced reporting training']
          },
          {
            id: '4',
            type: 'temporary',
            permission: 'project:admin',
            resource: 'project-alpha',
            action: 'manage',
            confidence: 78,
            reasoning: 'Temporary project leadership role detected from calendar and task assignments',
            category: 'efficiency',
            impact: 'low',
            similarUsers: [],
            estimatedBenefit: 'Enable project management for 30-day sprint cycle',
            risks: ['Temporary elevated access'],
            prerequisites: ['Project lead approval required']
          }
        ],
        insights: {
          overPrivilegedUsers: 12,
          underPrivilegedUsers: 8,
          obsoletePermissions: 5,
          securityRisks: 3
        },
        trends: [
          { category: 'Security', count: 15, change: -8 },
          { category: 'Efficiency', count: 23, change: 12 },
          { category: 'Compliance', count: 7, change: 2 },
          { category: 'Role-based', count: 19, change: 5 }
        ]
      };

      await new Promise(resolve => setTimeout(resolve, 1200)); // Simulate AI processing
      setData(mockData);
    } catch (error) {
      console.error('Failed to load recommendations:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleApplyRecommendation = async (recommendation: PermissionRecommendation) => {
    setApplyingIds(prev => new Set(prev).add(recommendation.id));
    
    try {
      await onApplyRecommendation?.(recommendation);
      // Remove applied recommendation from list
      setData(prev => prev ? {
        ...prev,
        recommendations: prev.recommendations.filter(r => r.id !== recommendation.id)
      } : null);
    } catch (error) {
      console.error('Failed to apply recommendation:', error);
    } finally {
      setApplyingIds(prev => {
        const newSet = new Set(prev);
        newSet.delete(recommendation.id);
        return newSet;
      });
    }
  };

  const getRecommendationIcon = (type: PermissionRecommendation['type']) => {
    switch (type) {
      case 'add': return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'remove': return <XCircle className="h-4 w-4 text-red-500" />;
      case 'upgrade': return <TrendingUp className="h-4 w-4 text-blue-500" />;
      case 'temporary': return <Clock className="h-4 w-4 text-orange-500" />;
    }
  };

  const getCategoryIcon = (category: PermissionRecommendation['category']) => {
    switch (category) {
      case 'security': return <Shield className="h-4 w-4 text-red-500" />;
      case 'efficiency': return <Sparkles className="h-4 w-4 text-blue-500" />;
      case 'compliance': return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      case 'role-based': return <Users className="h-4 w-4 text-purple-500" />;
    }
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 90) return 'text-green-600 bg-green-100';
    if (confidence >= 75) return 'text-blue-600 bg-blue-100';
    if (confidence >= 60) return 'text-orange-600 bg-orange-100';
    return 'text-red-600 bg-red-100';
  };

  const getImpactColor = (impact: PermissionRecommendation['impact']) => {
    switch (impact) {
      case 'low': return 'text-gray-600 bg-gray-100';
      case 'medium': return 'text-orange-600 bg-orange-100';
      case 'high': return 'text-red-600 bg-red-100';
    }
  };

  const filteredRecommendations = data?.recommendations.filter(rec => 
    activeTab === 'all' || rec.category === activeTab
  ) || [];

  useEffect(() => {
    loadRecommendations();
  }, [user.id]);

  if (loading) {
    return (
      <div className={`space-y-6 ${className}`}>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-center h-32">
              <div className="text-center">
                <Sparkles className="h-8 w-8 text-blue-500 mx-auto animate-pulse mb-2" />
                <p className="text-sm text-gray-600">AI is analyzing permissions...</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!data) {
    return (
      <div className={`${className}`}>
        <Card>
          <CardContent className="p-6 text-center">
            <AlertTriangle className="h-8 w-8 text-gray-400 mx-auto mb-2" />
            <p className="text-gray-600">Unable to load recommendations</p>
            <Button variant="outline" onClick={loadRecommendations} className="mt-4">
              Try Again
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header with insights */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Lightbulb className="h-5 w-5 text-yellow-500" />
            Smart Permission Recommendations
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <p className="text-2xl font-bold text-red-600">{data.insights.overPrivilegedUsers}</p>
              <p className="text-sm text-gray-600">Over-privileged</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-orange-600">{data.insights.underPrivilegedUsers}</p>
              <p className="text-sm text-gray-600">Under-privileged</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-gray-600">{data.insights.obsoletePermissions}</p>
              <p className="text-sm text-gray-600">Obsolete</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-red-600">{data.insights.securityRisks}</p>
              <p className="text-sm text-gray-600">Security Risks</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Recommendations */}
      <Card>
        <CardHeader>
          <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as any)}>
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="all">All ({data.recommendations.length})</TabsTrigger>
              <TabsTrigger value="security">Security</TabsTrigger>
              <TabsTrigger value="efficiency">Efficiency</TabsTrigger>
              <TabsTrigger value="compliance">Compliance</TabsTrigger>
            </TabsList>
          </Tabs>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {filteredRecommendations.map((recommendation) => (
              <Card key={recommendation.id} className="border-l-4 border-l-blue-500">
                <CardContent className="p-4">
                  <div className="flex justify-between items-start mb-3">
                    <div className="flex items-start gap-3">
                      {getRecommendationIcon(recommendation.type)}
                      <div>
                        <h4 className="font-semibold flex items-center gap-2">
                          {recommendation.type.charAt(0).toUpperCase() + recommendation.type.slice(1)} 
                          <code className="text-sm bg-gray-100 px-2 py-1 rounded">
                            {recommendation.permission}
                          </code>
                          <Badge className={getConfidenceColor(recommendation.confidence)}>
                            {recommendation.confidence}% confidence
                          </Badge>
                        </h4>
                        <p className="text-sm text-gray-600 mt-1">
                          {recommendation.resource} → {recommendation.action}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {getCategoryIcon(recommendation.category)}
                      <Badge className={getImpactColor(recommendation.impact)}>
                        {recommendation.impact} impact
                      </Badge>
                    </div>
                  </div>

                  <div className="space-y-3">
                    <p className="text-sm">{recommendation.reasoning}</p>
                    
                    <div className="bg-green-50 p-3 rounded-lg">
                      <p className="text-sm font-medium text-green-800">Expected Benefit:</p>
                      <p className="text-sm text-green-700">{recommendation.estimatedBenefit}</p>
                    </div>

                    {recommendation.risks.length > 0 && (
                      <div className="bg-red-50 p-3 rounded-lg">
                        <p className="text-sm font-medium text-red-800">Risks:</p>
                        <ul className="text-sm text-red-700 list-disc list-inside">
                          {recommendation.risks.map((risk, index) => (
                            <li key={index}>{risk}</li>
                          ))}
                        </ul>
                      </div>
                    )}

                    {recommendation.prerequisites.length > 0 && (
                      <div className="bg-yellow-50 p-3 rounded-lg">
                        <p className="text-sm font-medium text-yellow-800">Prerequisites:</p>
                        <ul className="text-sm text-yellow-700 list-disc list-inside">
                          {recommendation.prerequisites.map((prereq, index) => (
                            <li key={index}>{prereq}</li>
                          ))}
                        </ul>
                      </div>
                    )}

                    {recommendation.similarUsers.length > 0 && (
                      <div className="bg-blue-50 p-3 rounded-lg">
                        <p className="text-sm font-medium text-blue-800">Similar Users:</p>
                        <div className="flex flex-wrap gap-1 mt-1">
                          {recommendation.similarUsers.slice(0, 3).map((email, index) => (
                            <Badge key={index} variant="secondary" className="text-xs">
                              {email}
                            </Badge>
                          ))}
                          {recommendation.similarUsers.length > 3 && (
                            <Badge variant="secondary" className="text-xs">
                              +{recommendation.similarUsers.length - 3} more
                            </Badge>
                          )}
                        </div>
                      </div>
                    )}

                    <div className="flex justify-end gap-2 pt-3 border-t">
                      <Button 
                        variant="outline" 
                        size="sm"
                        onClick={() => {
                          // Remove from recommendations without applying
                          setData(prev => prev ? {
                            ...prev,
                            recommendations: prev.recommendations.filter(r => r.id !== recommendation.id)
                          } : null);
                        }}
                      >
                        Dismiss
                      </Button>
                      <Button 
                        size="sm"
                        onClick={() => handleApplyRecommendation(recommendation)}
                        disabled={applyingIds.has(recommendation.id)}
                      >
                        {applyingIds.has(recommendation.id) ? (
                          <>
                            <div className="animate-spin rounded-full h-3 w-3 border-b-2 border-white mr-2" />
                            Applying...
                          </>
                        ) : (
                          'Apply Recommendation'
                        )}
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}

            {filteredRecommendations.length === 0 && (
              <div className="text-center py-8">
                <Sparkles className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">
                  No recommendations available
                </h3>
                <p className="text-gray-600">
                  All permissions appear to be optimally configured for this user.
                </p>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Trends */}
      <Card>
        <CardHeader>
          <CardTitle>Recommendation Trends</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {data.trends.map((trend, index) => (
              <div key={index} className="text-center">
                <p className="text-2xl font-bold">{trend.count}</p>
                <p className="text-sm text-gray-600">{trend.category}</p>
                <p className={`text-xs ${trend.change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {trend.change >= 0 ? '+' : ''}{trend.change} this week
                </p>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}