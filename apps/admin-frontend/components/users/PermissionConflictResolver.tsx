'use client';

import { useState, useEffect } from 'react';
import { 
  AlertTriangle, 
  CheckCircle, 
  XCircle, 
  Info, 
  Shield, 
  Users, 
  Clock,
  RefreshCw,
  Eye,
  EyeOff,
  Filter,
  AlertCircle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/form-components';
import { useToast } from '@/components/ui/use-toast';

interface ValidationResult {
  isValid: boolean;
  conflicts: PermissionConflict[];
  warnings: PermissionWarning[];
  suggestions: PermissionSuggestion[];
  securityRisks: SecurityRisk[];
  businessRuleViolations: BusinessRuleViolation[];
}

interface PermissionConflict {
  id: string;
  type: 'duplicate' | 'overriding' | 'contradictory' | 'circular';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  affectedPermissions: string[];
  recommendedAction: string;
  canAutoResolve: boolean;
}

interface PermissionWarning {
  id: string;
  type: 'overprivileged' | 'redundant' | 'deprecated' | 'temporary_expired';
  message: string;
  affectedPermissions: string[];
  impact: string;
}

interface PermissionSuggestion {
  id: string;
  type: 'consolidation' | 'profile_replacement' | 'role_upgrade' | 'optimization';
  title: string;
  description: string;
  expectedBenefit: string;
  implementation: string[];
}

interface SecurityRisk {
  id: string;
  riskLevel: 'low' | 'medium' | 'high' | 'critical';
  category: 'privilege_escalation' | 'data_exposure' | 'system_access' | 'audit_bypass';
  description: string;
  affectedResources: string[];
  mitigationSteps: string[];
}

interface BusinessRuleViolation {
  id: string;
  rule: string;
  violation: string;
  impact: string;
  requiredApproval: boolean;
  approvalLevel: 'manager' | 'admin' | 'security_officer';
}

interface PermissionConflictResolverProps {
  userId: string;
  proposedChanges?: {
    addPermissions?: string[];
    removePermissions?: string[];
    addRoles?: string[];
    removeRoles?: string[];
    addProfiles?: string[];
    removeProfiles?: string[];
  };
  onValidationComplete?: (result: ValidationResult) => void;
  autoValidate?: boolean;
}

export function PermissionConflictResolver({ 
  userId, 
  proposedChanges,
  onValidationComplete,
  autoValidate = false 
}: PermissionConflictResolverProps) {
  const { toast } = useToast();
  
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'conflicts' | 'warnings' | 'suggestions' | 'security' | 'business'>('conflicts');
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [showResolved, setShowResolved] = useState(false);
  const [resolvedItems, setResolvedItems] = useState<Set<string>>(new Set());
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
  
  // Mock validation function - in production would call API
  const performValidation = async (): Promise<ValidationResult> => {
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1500));
    
    return {
      isValid: false,
      conflicts: [
        {
          id: 'conf-1',
          type: 'overriding',
          severity: 'high',
          description: 'User has both read-only and admin permissions for user management',
          affectedPermissions: ['users.read', 'users.admin'],
          recommendedAction: 'Remove read-only permission as admin permission includes it',
          canAutoResolve: true,
        },
        {
          id: 'conf-2',
          type: 'contradictory',
          severity: 'critical',
          description: 'Permission profile "basic-user" conflicts with role "admin"',
          affectedPermissions: ['profile:basic-user', 'role:admin'],
          recommendedAction: 'Remove basic-user profile or admin role',
          canAutoResolve: false,
        },
        {
          id: 'conf-3',
          type: 'duplicate',
          severity: 'low',
          description: 'Trading permission exists in both role and custom permissions',
          affectedPermissions: ['trading.execute', 'role:trader'],
          recommendedAction: 'Remove redundant custom permission',
          canAutoResolve: true,
        }
      ],
      warnings: [
        {
          id: 'warn-1',
          type: 'overprivileged',
          message: 'User has broader permissions than typically required for their role',
          affectedPermissions: ['system.admin', 'db.admin'],
          impact: 'Potential security risk if account is compromised',
        },
        {
          id: 'warn-2',
          type: 'temporary_expired',
          message: '2 temporary permissions have expired but are still active',
          affectedPermissions: ['temp:analytics.emergency', 'temp:trading.override'],
          impact: 'Cleanup required to maintain security posture',
        }
      ],
      suggestions: [
        {
          id: 'sugg-1',
          type: 'profile_replacement',
          title: 'Replace multiple permissions with Power User profile',
          description: 'Current permissions match 90% of Power User profile capabilities',
          expectedBenefit: 'Simplified management and consistent permissions',
          implementation: ['Add Power User profile', 'Remove 8 individual permissions'],
        },
        {
          id: 'sugg-2',
          type: 'consolidation',
          title: 'Consolidate analytics permissions',
          description: 'Multiple analytics permissions can be combined into Analytics role',
          expectedBenefit: 'Easier permission management',
          implementation: ['Add Analytics role', 'Remove individual analytics permissions'],
        }
      ],
      securityRisks: [
        {
          id: 'risk-1',
          riskLevel: 'high',
          category: 'privilege_escalation',
          description: 'User can modify their own permissions through role management access',
          affectedResources: ['user_roles', 'permission_profiles'],
          mitigationSteps: [
            'Add approval requirement for self-permission changes',
            'Implement separation of duties'
          ],
        },
        {
          id: 'risk-2',
          riskLevel: 'medium',
          category: 'data_exposure',
          description: 'Access to user data without audit logging enabled',
          affectedResources: ['user_data', 'personal_info'],
          mitigationSteps: [
            'Enable audit logging for data access',
            'Add data classification labels'
          ],
        }
      ],
      businessRuleViolations: [
        {
          id: 'rule-1',
          rule: 'Segregation of Duties: Trading and Compliance',
          violation: 'User has both trading and compliance oversight permissions',
          impact: 'Regulatory compliance violation',
          requiredApproval: true,
          approvalLevel: 'security_officer',
        }
      ]
    };
  };

  const runValidation = async () => {
    setLoading(true);
    try {
      const result = await performValidation();
      setValidationResult(result);
      onValidationComplete?.(result);
      
      if (result.conflicts.length === 0 && result.securityRisks.length === 0) {
        toast({
          title: 'Validation Complete',
          description: 'No conflicts or security risks found',
          variant: 'default',
        });
      } else {
        toast({
          title: 'Issues Found',
          description: `Found ${result.conflicts.length} conflicts and ${result.securityRisks.length} security risks`,
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Validation failed:', error);
      toast({
        title: 'Validation Error',
        description: 'Failed to validate permissions',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (autoValidate && proposedChanges) {
      runValidation();
    }
  }, [proposedChanges, autoValidate]);

  const handleAutoResolve = async (conflictId: string) => {
    // Mock auto-resolve
    setResolvedItems(prev => new Set([...prev, conflictId]));
    toast({
      title: 'Conflict Resolved',
      description: 'Automatic resolution applied',
      variant: 'default',
    });
  };

  const toggleExpanded = (itemId: string) => {
    setExpandedItems(prev => {
      const newSet = new Set(prev);
      if (newSet.has(itemId)) {
        newSet.delete(itemId);
      } else {
        newSet.add(itemId);
      }
      return newSet;
    });
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-100 text-red-800 border-red-200';
      case 'high': return 'bg-orange-100 text-orange-800 border-orange-200';
      case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case 'low': return 'bg-blue-100 text-blue-800 border-blue-200';
      default: return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical': return <XCircle className="h-4 w-4" />;
      case 'high': return <AlertTriangle className="h-4 w-4" />;
      case 'medium': return <AlertCircle className="h-4 w-4" />;
      case 'low': return <Info className="h-4 w-4" />;
      default: return <Info className="h-4 w-4" />;
    }
  };

  const filteredConflicts = validationResult?.conflicts.filter(conflict => {
    if (filterSeverity !== 'all' && conflict.severity !== filterSeverity) return false;
    if (!showResolved && resolvedItems.has(conflict.id)) return false;
    return true;
  }) || [];

  const filteredSecurityRisks = validationResult?.securityRisks.filter(risk => {
    if (filterSeverity !== 'all' && risk.riskLevel !== filterSeverity) return false;
    return true;
  }) || [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Permission Validation & Conflict Resolution
          </h3>
          <p className="text-sm text-muted-foreground">
            Analyze and resolve permission conflicts, security risks, and business rule violations
          </p>
        </div>
        <Button onClick={runValidation} disabled={loading}>
          {loading ? (
            <>
              <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
              Validating...
            </>
          ) : (
            <>
              <Shield className="h-4 w-4 mr-2" />
              Run Validation
            </>
          )}
        </Button>
      </div>

      {/* Overall Status */}
      {validationResult && (
        <Alert className={validationResult.isValid ? 'border-green-200 bg-green-50' : 'border-red-200 bg-red-50'}>
          <div className="flex items-center gap-2">
            {validationResult.isValid ? (
              <CheckCircle className="h-4 w-4 text-green-600" />
            ) : (
              <XCircle className="h-4 w-4 text-red-600" />
            )}
            <AlertDescription className={validationResult.isValid ? 'text-green-800' : 'text-red-800'}>
              {validationResult.isValid 
                ? 'All permissions are valid with no conflicts detected'
                : `Found ${validationResult.conflicts.length} conflicts, ${validationResult.warnings.length} warnings, ${validationResult.securityRisks.length} security risks`
              }
            </AlertDescription>
          </div>
        </Alert>
      )}

      {validationResult && (
        <>
          {/* Filters */}
          <div className="flex items-center gap-4 p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-2">
              <Label htmlFor="severity-filter">Filter by Severity:</Label>
              <Select value={filterSeverity} onValueChange={setFilterSeverity}>
                <SelectTrigger className="w-32">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All</SelectItem>
                  <SelectItem value="critical">Critical</SelectItem>
                  <SelectItem value="high">High</SelectItem>
                  <SelectItem value="medium">Medium</SelectItem>
                  <SelectItem value="low">Low</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="flex items-center gap-2">
              <Checkbox
                id="show-resolved"
                checked={showResolved}
                onChange={(e) => setShowResolved(e.target.checked)}
              />
              <Label htmlFor="show-resolved">Show Resolved</Label>
            </div>
          </div>

          {/* Results Tabs */}
          <Tabs value={activeTab} onValueChange={(value: any) => setActiveTab(value)}>
            <TabsList className="grid w-full grid-cols-5">
              <TabsTrigger value="conflicts" className="flex items-center gap-2">
                <AlertTriangle className="h-4 w-4" />
                Conflicts ({validationResult.conflicts.length})
              </TabsTrigger>
              <TabsTrigger value="warnings" className="flex items-center gap-2">
                <AlertCircle className="h-4 w-4" />
                Warnings ({validationResult.warnings.length})
              </TabsTrigger>
              <TabsTrigger value="suggestions" className="flex items-center gap-2">
                <Info className="h-4 w-4" />
                Suggestions ({validationResult.suggestions.length})
              </TabsTrigger>
              <TabsTrigger value="security" className="flex items-center gap-2">
                <Shield className="h-4 w-4" />
                Security ({validationResult.securityRisks.length})
              </TabsTrigger>
              <TabsTrigger value="business" className="flex items-center gap-2">
                <Users className="h-4 w-4" />
                Business Rules ({validationResult.businessRuleViolations.length})
              </TabsTrigger>
            </TabsList>

            {/* Conflicts Tab */}
            <TabsContent value="conflicts" className="space-y-4">
              {filteredConflicts.length === 0 ? (
                <Card>
                  <CardContent className="p-8 text-center">
                    <CheckCircle className="h-12 w-12 mx-auto mb-4 text-green-500" />
                    <p className="text-muted-foreground">No permission conflicts found</p>
                  </CardContent>
                </Card>
              ) : (
                <div className="space-y-4">
                  {filteredConflicts.map((conflict) => (
                    <Card key={conflict.id} className={`border-l-4 ${getSeverityColor(conflict.severity)}`}>
                      <CardHeader>
                        <div className="flex items-start justify-between">
                          <div className="flex items-center gap-3">
                            {getSeverityIcon(conflict.severity)}
                            <div>
                              <CardTitle className="text-sm font-medium">
                                {conflict.type.charAt(0).toUpperCase() + conflict.type.slice(1)} Conflict
                              </CardTitle>
                              <Badge variant="outline" className={getSeverityColor(conflict.severity)}>
                                {conflict.severity}
                              </Badge>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            {conflict.canAutoResolve && !resolvedItems.has(conflict.id) && (
                              <Button 
                                size="sm" 
                                variant="outline"
                                onClick={() => handleAutoResolve(conflict.id)}
                              >
                                Auto-Resolve
                              </Button>
                            )}
                            {resolvedItems.has(conflict.id) && (
                              <Badge variant="outline" className="text-green-600 border-green-300">
                                Resolved
                              </Badge>
                            )}
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => toggleExpanded(conflict.id)}
                            >
                              {expandedItems.has(conflict.id) ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                            </Button>
                          </div>
                        </div>
                      </CardHeader>
                      
                      <CardContent className="space-y-3">
                        <p className="text-sm">{conflict.description}</p>
                        
                        {expandedItems.has(conflict.id) && (
                          <div className="space-y-3 pt-3 border-t">
                            <div>
                              <Label className="text-xs font-semibold">Affected Permissions:</Label>
                              <div className="flex flex-wrap gap-1 mt-1">
                                {conflict.affectedPermissions.map((perm, index) => (
                                  <Badge key={index} variant="secondary" className="text-xs">
                                    {perm}
                                  </Badge>
                                ))}
                              </div>
                            </div>
                            
                            <div>
                              <Label className="text-xs font-semibold">Recommended Action:</Label>
                              <p className="text-xs text-muted-foreground mt-1">{conflict.recommendedAction}</p>
                            </div>
                          </div>
                        )}
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </TabsContent>

            {/* Warnings Tab */}
            <TabsContent value="warnings" className="space-y-4">
              {validationResult.warnings.length === 0 ? (
                <Card>
                  <CardContent className="p-8 text-center">
                    <CheckCircle className="h-12 w-12 mx-auto mb-4 text-green-500" />
                    <p className="text-muted-foreground">No warnings found</p>
                  </CardContent>
                </Card>
              ) : (
                <div className="space-y-4">
                  {validationResult.warnings.map((warning) => (
                    <Card key={warning.id} className="border-l-4 border-yellow-300">
                      <CardHeader>
                        <div className="flex items-center gap-3">
                          <AlertTriangle className="h-4 w-4 text-yellow-600" />
                          <div>
                            <CardTitle className="text-sm font-medium">
                              {warning.type.replace('_', ' ').charAt(0).toUpperCase() + warning.type.slice(1).replace('_', ' ')}
                            </CardTitle>
                          </div>
                        </div>
                      </CardHeader>
                      <CardContent className="space-y-2">
                        <p className="text-sm">{warning.message}</p>
                        <p className="text-xs text-muted-foreground">Impact: {warning.impact}</p>
                        <div className="flex flex-wrap gap-1">
                          {warning.affectedPermissions.map((perm, index) => (
                            <Badge key={index} variant="secondary" className="text-xs">
                              {perm}
                            </Badge>
                          ))}
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </TabsContent>

            {/* Suggestions Tab */}
            <TabsContent value="suggestions" className="space-y-4">
              {validationResult.suggestions.length === 0 ? (
                <Card>
                  <CardContent className="p-8 text-center">
                    <Info className="h-12 w-12 mx-auto mb-4 text-blue-500" />
                    <p className="text-muted-foreground">No optimization suggestions available</p>
                  </CardContent>
                </Card>
              ) : (
                <div className="space-y-4">
                  {validationResult.suggestions.map((suggestion) => (
                    <Card key={suggestion.id} className="border-l-4 border-blue-300">
                      <CardHeader>
                        <div className="flex items-start justify-between">
                          <div className="flex items-center gap-3">
                            <Info className="h-4 w-4 text-blue-600" />
                            <div>
                              <CardTitle className="text-sm font-medium">{suggestion.title}</CardTitle>
                              <Badge variant="outline" className="text-blue-600 border-blue-300">
                                {suggestion.type.replace('_', ' ')}
                              </Badge>
                            </div>
                          </div>
                          <Button size="sm" variant="outline">
                            Apply Suggestion
                          </Button>
                        </div>
                      </CardHeader>
                      <CardContent className="space-y-3">
                        <p className="text-sm">{suggestion.description}</p>
                        <div className="space-y-2">
                          <div>
                            <Label className="text-xs font-semibold">Expected Benefit:</Label>
                            <p className="text-xs text-muted-foreground">{suggestion.expectedBenefit}</p>
                          </div>
                          <div>
                            <Label className="text-xs font-semibold">Implementation Steps:</Label>
                            <ul className="text-xs text-muted-foreground list-disc list-inside ml-2">
                              {suggestion.implementation.map((step, index) => (
                                <li key={index}>{step}</li>
                              ))}
                            </ul>
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </TabsContent>

            {/* Security Tab */}
            <TabsContent value="security" className="space-y-4">
              {filteredSecurityRisks.length === 0 ? (
                <Card>
                  <CardContent className="p-8 text-center">
                    <Shield className="h-12 w-12 mx-auto mb-4 text-green-500" />
                    <p className="text-muted-foreground">No security risks identified</p>
                  </CardContent>
                </Card>
              ) : (
                <div className="space-y-4">
                  {filteredSecurityRisks.map((risk) => (
                    <Card key={risk.id} className={`border-l-4 ${getSeverityColor(risk.riskLevel)}`}>
                      <CardHeader>
                        <div className="flex items-center gap-3">
                          <Shield className="h-4 w-4 text-red-600" />
                          <div>
                            <CardTitle className="text-sm font-medium">
                              {risk.category.replace('_', ' ').charAt(0).toUpperCase() + risk.category.slice(1).replace('_', ' ')}
                            </CardTitle>
                            <Badge variant="outline" className={getSeverityColor(risk.riskLevel)}>
                              {risk.riskLevel} risk
                            </Badge>
                          </div>
                        </div>
                      </CardHeader>
                      <CardContent className="space-y-3">
                        <p className="text-sm">{risk.description}</p>
                        
                        <div>
                          <Label className="text-xs font-semibold">Affected Resources:</Label>
                          <div className="flex flex-wrap gap-1 mt-1">
                            {risk.affectedResources.map((resource, index) => (
                              <Badge key={index} variant="secondary" className="text-xs">
                                {resource}
                              </Badge>
                            ))}
                          </div>
                        </div>
                        
                        <div>
                          <Label className="text-xs font-semibold">Mitigation Steps:</Label>
                          <ul className="text-xs text-muted-foreground list-disc list-inside ml-2 mt-1">
                            {risk.mitigationSteps.map((step, index) => (
                              <li key={index}>{step}</li>
                            ))}
                          </ul>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </TabsContent>

            {/* Business Rules Tab */}
            <TabsContent value="business" className="space-y-4">
              {validationResult.businessRuleViolations.length === 0 ? (
                <Card>
                  <CardContent className="p-8 text-center">
                    <Users className="h-12 w-12 mx-auto mb-4 text-green-500" />
                    <p className="text-muted-foreground">No business rule violations found</p>
                  </CardContent>
                </Card>
              ) : (
                <div className="space-y-4">
                  {validationResult.businessRuleViolations.map((violation) => (
                    <Card key={violation.id} className="border-l-4 border-purple-300">
                      <CardHeader>
                        <div className="flex items-start justify-between">
                          <div className="flex items-center gap-3">
                            <Users className="h-4 w-4 text-purple-600" />
                            <div>
                              <CardTitle className="text-sm font-medium">{violation.rule}</CardTitle>
                              {violation.requiredApproval && (
                                <Badge variant="outline" className="text-purple-600 border-purple-300">
                                  Requires {violation.approvalLevel.replace('_', ' ')} approval
                                </Badge>
                              )}
                            </div>
                          </div>
                          {violation.requiredApproval && (
                            <Button size="sm" variant="outline">
                              Request Approval
                            </Button>
                          )}
                        </div>
                      </CardHeader>
                      <CardContent className="space-y-2">
                        <div>
                          <Label className="text-xs font-semibold">Violation:</Label>
                          <p className="text-sm">{violation.violation}</p>
                        </div>
                        <div>
                          <Label className="text-xs font-semibold">Business Impact:</Label>
                          <p className="text-xs text-muted-foreground">{violation.impact}</p>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </TabsContent>
          </Tabs>
        </>
      )}
    </div>
  );
}