'use client';

import { useState } from 'react';
import { Textarea } from '@/components/ui/textarea';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { 
  Plus, 
  Edit2, 
  Trash2, 
  Shield, 
  Lock,
  Unlock,
  Search,
  // Filter - removed unused import
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  Globe,
  MapPin,
  Smartphone
} from 'lucide-react';
interface AccessRule {
  id: string;
  name: string;
  description: string;
  effect: 'allow' | 'deny';
  principal: {
    type: 'user' | 'role' | 'group';
    id: string;
    name: string;
  };
  resource: {
    type: string;
    path: string;
    actions: string[];
  };
  conditions: AccessCondition[];
  priority: number;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
}
interface AccessCondition {
  id: string;
  type: 'ip_range' | 'time_range' | 'location' | 'device_type' | 'mfa_required';
  operator: 'equals' | 'not_equals' | 'in' | 'not_in' | 'between';
  value: string | string[];
  description: string;
}
const MOCK_ACCESS_RULES: AccessRule[] = [
  {
    id: '1',
    name: 'Admin Full Access',
    description: 'Full system access for administrators',
    effect: 'allow',
    principal: { type: 'role', id: '1', name: 'Admin' },
    resource: { type: 'system', path: '/*', actions: ['*'] },
    conditions: [
      {
        id: '1',
        type: 'mfa_required',
        operator: 'equals',
        value: 'true',
        description: 'Multi-factor authentication required'
      }
    ],
    priority: 1,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-15T00:00:00Z',
    createdBy: 'system'
  },
  {
    id: '2',
    name: 'Analyst Analytics Access',
    description: 'Allow analysts to access analytics features',
    effect: 'allow',
    principal: { type: 'role', id: '2', name: 'Analyst' },
    resource: { type: 'analytics', path: '/analytics/*', actions: ['read', 'export'] },
    conditions: [
      {
        id: '2',
        type: 'time_range',
        operator: 'between',
        value: ['09:00', '18:00'],
        description: 'Business hours only'
      }
    ],
    priority: 2,
    isActive: true,
    createdAt: '2024-01-05T00:00:00Z',
    updatedAt: '2024-01-20T00:00:00Z',
    createdBy: 'admin'
  },
  {
    id: '3',
    name: 'Block External IP Access',
    description: 'Deny access from external IP ranges',
import { Card, CardContent, CardHeader, CardTitle, Button, Input, Badge, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui';
    effect: 'deny',
    principal: { type: 'user', id: '*', name: 'All Users' },
    resource: { type: 'system', path: '/admin/*', actions: ['*'] },
    conditions: [
      {
        id: '3',
        type: 'ip_range',
        operator: 'not_in',
        value: ['192.168.1.0/24', '10.0.0.0/8'],
        description: 'Block non-internal IP addresses'
      }
    ],
    priority: 10,
    isActive: true,
    createdAt: '2024-01-10T00:00:00Z',
    updatedAt: '2024-01-25T00:00:00Z',
    createdBy: 'security-admin'
  }
];
export function AccessControlList() {
  const [rules, setRules] = useState<AccessRule[]>(MOCK_ACCESS_RULES);
  const [searchTerm, setSearchTerm] = useState('');
  const [effectFilter, setEffectFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [selectedRule, setSelectedRule] = useState<AccessRule | null>(null);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const filteredRules = rules.filter(rule => {
    const matchesSearch = rule.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         rule.description.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesEffect = effectFilter === 'all' || rule.effect === effectFilter;
    const matchesStatus = statusFilter === 'all' || 
                         (statusFilter === 'active' ? rule.isActive : !rule.isActive);
    return matchesSearch && matchesEffect && matchesStatus;
  });
  const getEffectColor = (effect: string) => {
    return effect === 'allow' ? 'text-green-600 bg-green-50' : 'text-red-600 bg-red-50';
  };
  const getEffectIcon = (effect: string) => {
    return effect === 'allow' ? <CheckCircle className="h-4 w-4" /> : <XCircle className="h-4 w-4" />;
  };
  const getConditionIcon = (type: string) => {
    switch (type) {
      case 'ip_range': return <Globe className="h-4 w-4" />;
      case 'time_range': return <Clock className="h-4 w-4" />;
      case 'location': return <MapPin className="h-4 w-4" />;
      case 'device_type': return <Smartphone className="h-4 w-4" />;
      case 'mfa_required': return <Shield className="h-4 w-4" />;
      default: return <AlertTriangle className="h-4 w-4" />;
    }
  };
  const toggleRuleStatus = (ruleId: string) => {
    setRules(rules.map(rule =>
      rule.id === ruleId
        ? { ...rule, isActive: !rule.isActive, updatedAt: new Date().toISOString() }
        : rule
    ));
  };
  const deleteRule = (ruleId: string) => {
    setRules(rules.filter(rule => rule.id !== ruleId));
  };
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Access Control List</h2>
          <p className="text-muted-foreground">
            Manage fine-grained access control rules and conditions
          </p>
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-4 w-4 mr-2" />
              Create Rule
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-4xl">
            <DialogHeader>
              <DialogTitle>Create Access Control Rule</DialogTitle>
            </DialogHeader>
            <AccessRuleForm
              onSave={() => setIsCreateDialogOpen(false)}
              onCancel={() => setIsCreateDialogOpen(false)}
            />
          </DialogContent>
        </Dialog>
      </div>
      {/* Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search rules..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-9"
              />
            </div>
            <Select value={effectFilter} onValueChange={setEffectFilter}>
              <SelectTrigger className="w-32">
                <SelectValue placeholder="Effect" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Effects</SelectItem>
                <SelectItem value="allow">Allow</SelectItem>
                <SelectItem value="deny">Deny</SelectItem>
              </SelectContent>
            </Select>
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="w-32">
                <SelectValue placeholder="Status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="active">Active</SelectItem>
                <SelectItem value="inactive">Inactive</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>
      {/* Rules Table */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Lock className="h-5 w-5" />
            Access Rules ({filteredRules.length})
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Rule</TableHead>
                <TableHead>Effect</TableHead>
                <TableHead>Principal</TableHead>
                <TableHead>Resource</TableHead>
                <TableHead>Conditions</TableHead>
                <TableHead>Priority</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredRules.map((rule) => (
                <TableRow key={rule.id}>
                  <TableCell>
                    <div>
                      <div className="font-medium">{rule.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {rule.description}
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs ${getEffectColor(rule.effect)}`}>
                      {getEffectIcon(rule.effect)}
                      {rule.effect.toUpperCase()}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium text-sm">{rule.principal.name}</div>
                      <Badge variant="outline" className="text-xs">
                        {rule.principal.type}
                      </Badge>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium text-sm">{rule.resource.path}</div>
                      <div className="text-xs text-muted-foreground">
                        {rule.resource.actions.join(', ')}
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {rule.conditions.map((condition) => (
                        <Badge
                          key={condition.id}
                          variant="secondary"
                          className="text-xs flex items-center gap-1"
                        >
                          {getConditionIcon(condition.type)}
                          {condition.type.replace('_', ' ')}
                        </Badge>
                      ))}
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline">{rule.priority}</Badge>
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => toggleRuleStatus(rule.id)}
                      className={rule.isActive ? 'text-green-600' : 'text-red-600'}
                    >
                      {rule.isActive ? <Unlock className="h-4 w-4" /> : <Lock className="h-4 w-4" />}
                    </Button>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setSelectedRule(rule)}
                      >
                        <Edit2 className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => deleteRule(rule.id)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
      {/* Rule Details Dialog */}
      {selectedRule && (
        <Dialog open={!!selectedRule} onOpenChange={() => setSelectedRule(null)}>
          <DialogContent className="max-w-4xl">
            <DialogHeader>
              <DialogTitle>Edit Access Control Rule</DialogTitle>
            </DialogHeader>
            <AccessRuleForm
              rule={selectedRule}
              onSave={() => setSelectedRule(null)}
              onCancel={() => setSelectedRule(null)}
            />
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
}
// Access Rule Form Component
interface AccessRuleFormProps {
  rule?: AccessRule;
  onSave: () => void;
  onCancel: () => void;
}
function AccessRuleForm({ rule, onSave, onCancel }: AccessRuleFormProps) {
  const [formData, setFormData] = useState<Partial<AccessRule>>(
    rule || {
      name: '',
      description: '',
      effect: 'allow',
      principal: { type: 'user', id: '', name: '' },
      resource: { type: '', path: '', actions: [] },
      conditions: [],
      priority: 1,
      isActive: true
    }
  );
  const handleSave = () => {
    // Save logic would go here
    // Rule data will be saved
    onSave();
  };
  return (
    <Tabs defaultValue="basic" className="space-y-6">
      <TabsList>
        <TabsTrigger value="basic">Basic Info</TabsTrigger>
        <TabsTrigger value="principal">Principal</TabsTrigger>
        <TabsTrigger value="resource">Resource</TabsTrigger>
        <TabsTrigger value="conditions">Conditions</TabsTrigger>
      </TabsList>
      <TabsContent value="basic" className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-sm font-medium">Rule Name</label>
            <Input
              value={formData.name || ''}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="Enter rule name"
            />
          </div>
          <div>
            <label className="text-sm font-medium">Effect</label>
            <Select
              value={formData.effect}
              onValueChange={(value: 'allow' | 'deny') => 
                setFormData({ ...formData, effect: value })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="allow">Allow</SelectItem>
                <SelectItem value="deny">Deny</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <div>
          <label className="text-sm font-medium">Description</label>
          <Textarea
            value={formData.description || ''}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            placeholder="Enter rule description"
            rows={3}
          />
        </div>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-sm font-medium">Priority</label>
            <Input
              type="number"
              value={formData.priority || 1}
              onChange={(e) => setFormData({ ...formData, priority: parseInt(e.target.value) })}
              min={1}
              max={100}
            />
          </div>
        </div>
      </TabsContent>
      <TabsContent value="principal" className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-sm font-medium">Principal Type</label>
            <Select
              value={formData.principal?.type}
              onValueChange={(value: 'user' | 'role' | 'group') =>
                setFormData({
                  ...formData,
                  principal: { ...formData.principal!, type: value }
                })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="user">User</SelectItem>
                <SelectItem value="role">Role</SelectItem>
                <SelectItem value="group">Group</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div>
            <label className="text-sm font-medium">Principal Name</label>
            <Input
              value={formData.principal?.name || ''}
              onChange={(e) => setFormData({
                ...formData,
                principal: { ...formData.principal!, name: e.target.value }
              })}
              placeholder="Enter principal name or ID"
            />
          </div>
        </div>
      </TabsContent>
      <TabsContent value="resource" className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-sm font-medium">Resource Type</label>
            <Select
              value={formData.resource?.type}
              onValueChange={(value) =>
                setFormData({
                  ...formData,
                  resource: { ...formData.resource!, type: value }
                })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">System</SelectItem>
                <SelectItem value="analytics">Analytics</SelectItem>
                <SelectItem value="users">Users</SelectItem>
                <SelectItem value="billing">Billing</SelectItem>
                <SelectItem value="patterns">Patterns</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div>
            <label className="text-sm font-medium">Resource Path</label>
            <Input
              value={formData.resource?.path || ''}
              onChange={(e) => setFormData({
                ...formData,
                resource: { ...formData.resource!, path: e.target.value }
              })}
              placeholder="e.g., /analytics/* or /users/123"
            />
          </div>
        </div>
        <div>
          <label className="text-sm font-medium">Actions</label>
          <Input
            value={formData.resource?.actions?.join(', ') || ''}
            onChange={(e) => setFormData({
              ...formData,
              resource: {
                ...formData.resource!,
                actions: e.target.value.split(',').map(s => s.trim())
              }
            })}
            placeholder="e.g., read, write, delete (comma-separated)"
          />
        </div>
      </TabsContent>
      <TabsContent value="conditions" className="space-y-4">
        <div className="text-sm text-muted-foreground">
          Add conditions to restrict when this rule applies
        </div>
        {/* Conditions would be dynamically added here */}
        <div className="space-y-3">
          <Button variant="outline" size="sm">
            <Plus className="h-4 w-4 mr-2" />
            Add Condition
          </Button>
        </div>
      </TabsContent>
      {/* Actions */}
      <div className="flex justify-end gap-3 pt-4 border-t">
        <Button variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={handleSave}>
          {rule ? 'Update Rule' : 'Create Rule'}
        </Button>
      </div>
    </Tabs>
  );
}