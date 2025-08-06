'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Bell, 
  Plus, 
  Edit, 
  Trash2,
  Mail,
  Smartphone,
  AlertTriangle,
  CheckCircle,
  Clock
} from 'lucide-react';

interface Alert {
  id: string;
  name: string;
  symbol: string;
  patternType: string;
  minConfidence: number;
  isActive: boolean;
  triggerCount: number;
  lastTriggered?: string;
  notificationMethods: string[];
}

interface AlertSystemProps {
  patterns: any[];
}

const mockAlerts: Alert[] = [
  {
    id: '1',
    name: 'AAPL Breakout Alert',
    symbol: 'AAPL',
    patternType: 'breakout',
    minConfidence: 80,
    isActive: true,
    triggerCount: 3,
    lastTriggered: '2024-01-15T10:30:00Z',
    notificationMethods: ['email', 'push']
  },
  {
    id: '2',
    name: 'Tech Reversal Patterns',
    symbol: 'MSFT,GOOGL,AMZN',
    patternType: 'reversal',
    minConfidence: 75,
    isActive: true,
    triggerCount: 1,
    lastTriggered: '2024-01-14T14:20:00Z',
    notificationMethods: ['email']
  },
  {
    id: '3',
    name: 'High Confidence Signals',
    symbol: '*',
    patternType: 'all',
    minConfidence: 90,
    isActive: false,
    triggerCount: 0,
    notificationMethods: ['email', 'push', 'sms']
  }
];

export function AlertSystem({ patterns: _patterns }: AlertSystemProps) {
  const [alerts, setAlerts] = useState<Alert[]>(mockAlerts);
  const [_editingAlert, setEditingAlert] = useState<Alert | null>(null);
  const [newAlert, setNewAlert] = useState({
    name: '',
    symbol: '',
    patternType: 'all',
    minConfidence: 70,
    notificationMethods: ['email']
  });

  const handleCreateAlert = () => {
    const alert: Alert = {
      id: Date.now().toString(),
      ...newAlert,
      isActive: true,
      triggerCount: 0
    };
    setAlerts([...alerts, alert]);
    setNewAlert({
      name: '',
      symbol: '',
      patternType: 'all',
      minConfidence: 70,
      notificationMethods: ['email']
    });
  };

  const handleToggleAlert = (id: string) => {
    setAlerts(alerts.map(alert => 
      alert.id === id ? { ...alert, isActive: !alert.isActive } : alert
    ));
  };

  const handleDeleteAlert = (id: string) => {
    setAlerts(alerts.filter(alert => alert.id !== id));
  };

  const getStatusBadge = (alert: Alert) => {
    if (!alert.isActive) return <Badge variant="secondary">Inactive</Badge>;
    if (alert.triggerCount > 0) return <Badge variant="default">Active</Badge>;
    return <Badge variant="outline">Waiting</Badge>;
  };

  const getNotificationIcon = (method: string) => {
    switch (method) {
      case 'email': return <Mail className="h-4 w-4" />;
      case 'push': return <Smartphone className="h-4 w-4" />;
      case 'sms': return <Smartphone className="h-4 w-4" />;
      default: return <Bell className="h-4 w-4" />;
    }
  };

  return (
    <div className="space-y-6">
      <Tabs defaultValue="alerts" className="w-full">
        <TabsList>
          <TabsTrigger value="alerts">Active Alerts</TabsTrigger>
          <TabsTrigger value="create">Create Alert</TabsTrigger>
          <TabsTrigger value="history">Alert History</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>

        <TabsContent value="alerts" className="space-y-4">
          {/* Alert Summary */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm text-muted-foreground">Active Alerts</p>
                    <p className="text-2xl font-bold">{alerts.filter(a => a.isActive).length}</p>
                  </div>
                  <CheckCircle className="h-8 w-8 text-green-600" />
                </div>
              </CardContent>
            </Card>
            
            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm text-muted-foreground">Triggered Today</p>
                    <p className="text-2xl font-bold">5</p>
                  </div>
                  <AlertTriangle className="h-8 w-8 text-orange-600" />
                </div>
              </CardContent>
            </Card>
            
            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm text-muted-foreground">Total Alerts</p>
                    <p className="text-2xl font-bold">{alerts.length}</p>
                  </div>
                  <Bell className="h-8 w-8 text-blue-600" />
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Alert List */}
          <div className="space-y-3">
            {alerts.map((alert) => (
              <Card key={alert.id}>
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="font-medium">{alert.name}</h3>
                        {getStatusBadge(alert)}
                        <Switch
                          checked={alert.isActive}
                          onCheckedChange={() => handleToggleAlert(alert.id)}
                        />
                      </div>
                      
                      <div className="text-sm text-muted-foreground space-y-1">
                        <div>Symbol(s): {alert.symbol}</div>
                        <div>Pattern: {alert.patternType} | Min Confidence: {alert.minConfidence}%</div>
                        <div className="flex items-center gap-2">
                          <span>Notifications:</span>
                          {alert.notificationMethods.map((method, index) => (
                            <div key={index} className="flex items-center gap-1">
                              {getNotificationIcon(method)}
                              <span className="text-xs">{method}</span>
                            </div>
                          ))}
                        </div>
                        {alert.lastTriggered && (
                          <div className="flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            Last triggered: {new Date(alert.lastTriggered).toLocaleString()}
                          </div>
                        )}
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <div className="text-right mr-4">
                        <div className="text-sm font-medium">Triggers</div>
                        <div className="text-xl font-bold">{alert.triggerCount}</div>
                      </div>
                      
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setEditingAlert(alert)}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                      
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleDeleteAlert(alert.id)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="create" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Plus className="h-5 w-5" />
                Create New Alert
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="alert-name">Alert Name</Label>
                  <Input
                    id="alert-name"
                    placeholder="My Pattern Alert"
                    value={newAlert.name}
                    onChange={(e) => setNewAlert({ ...newAlert, name: e.target.value })}
                  />
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="alert-symbol">Symbol(s)</Label>
                  <Input
                    id="alert-symbol"
                    placeholder="AAPL, MSFT or * for all"
                    value={newAlert.symbol}
                    onChange={(e) => setNewAlert({ ...newAlert, symbol: e.target.value })}
                  />
                </div>
                
                <div className="space-y-2">
                  <Label>Pattern Type</Label>
                  <Select 
                    value={newAlert.patternType} 
                    onValueChange={(value) => setNewAlert({ ...newAlert, patternType: value })}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Patterns</SelectItem>
                      <SelectItem value="breakout">Breakout</SelectItem>
                      <SelectItem value="reversal">Reversal</SelectItem>
                      <SelectItem value="trend">Trend</SelectItem>
                      <SelectItem value="continuation">Continuation</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <Label>Minimum Confidence</Label>
                  <Select 
                    value={newAlert.minConfidence.toString()} 
                    onValueChange={(value) => setNewAlert({ ...newAlert, minConfidence: parseInt(value) })}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="50">50% - Low</SelectItem>
                      <SelectItem value="60">60% - Medium Low</SelectItem>
                      <SelectItem value="70">70% - Medium</SelectItem>
                      <SelectItem value="80">80% - High</SelectItem>
                      <SelectItem value="90">90% - Very High</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              
              <div className="space-y-2">
                <Label>Notification Methods</Label>
                <div className="flex gap-4">
                  {['email', 'push', 'sms'].map((method) => (
                    <div key={method} className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        id={method}
                        checked={newAlert.notificationMethods.includes(method)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setNewAlert({
                              ...newAlert,
                              notificationMethods: [...newAlert.notificationMethods, method]
                            });
                          } else {
                            setNewAlert({
                              ...newAlert,
                              notificationMethods: newAlert.notificationMethods.filter(m => m !== method)
                            });
                          }
                        }}
                      />
                      <Label htmlFor={method} className="flex items-center gap-1">
                        {getNotificationIcon(method)}
                        {method.charAt(0).toUpperCase() + method.slice(1)}
                      </Label>
                    </div>
                  ))}
                </div>
              </div>
              
              <Button onClick={handleCreateAlert} className="w-full">
                <Plus className="h-4 w-4 mr-2" />
                Create Alert
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="history" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Alert History</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {[
                  { time: '10:30 AM', alert: 'AAPL Breakout Alert', pattern: 'Ascending Triangle', confidence: 85 },
                  { time: '09:15 AM', alert: 'Tech Reversal Patterns', pattern: 'Double Bottom', confidence: 78 },
                  { time: '08:45 AM', alert: 'High Confidence Signals', pattern: 'Bull Flag', confidence: 92 }
                ].map((item, index) => (
                  <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                    <div>
                      <div className="font-medium">{item.alert}</div>
                      <div className="text-sm text-muted-foreground">{item.pattern} detected</div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm font-medium">{item.confidence}% confidence</div>
                      <div className="text-sm text-muted-foreground">{item.time}</div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="settings" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Notification Settings</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label>Email Notifications</Label>
                  <p className="text-sm text-muted-foreground">Receive alerts via email</p>
                </div>
                <Switch defaultChecked />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label>Push Notifications</Label>
                  <p className="text-sm text-muted-foreground">Receive alerts on your device</p>
                </div>
                <Switch defaultChecked />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label>SMS Notifications</Label>
                  <p className="text-sm text-muted-foreground">Receive alerts via SMS</p>
                </div>
                <Switch />
              </div>
              
              <div className="space-y-2">
                <Label>Quiet Hours</Label>
                <div className="flex gap-2">
                  <Input placeholder="22:00" className="w-24" />
                  <span className="self-center">to</span>
                  <Input placeholder="07:00" className="w-24" />
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}