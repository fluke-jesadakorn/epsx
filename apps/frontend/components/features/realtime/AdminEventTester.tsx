// Admin component for testing real-time events

'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { useAuth } from '@/context/auth-context';
import { apiClient } from '@/lib/api-client';

interface SimulatePaymentData {
  payment_id: string;
  user_id: string;
  amount: number;
  currency: string;
  event_type: 'started' | 'completed' | 'failed';
  transaction_id?: string;
  error_code?: string;
  error_message?: string;
}

interface BroadcastNotificationData {
  title: string;
  message: string;
  level: 'info' | 'warning' | 'error' | 'success';
  target_user?: string;
}

interface SimulateStockData {
  symbol: string;
  price: number;
  change: number;
  change_percent: number;
  volume: number;
}

export default function AdminEventTester() {
  const { user } = useAuth();
  const [loading, setLoading] = useState<string | null>(null);
  const [result, setResult] = useState<string>('');

  // Payment simulation form
  const [paymentData, setPaymentData] = useState<SimulatePaymentData>({
    payment_id: `pay_${Date.now()}`,
    user_id: 'test_user',
    amount: 99.99,
    currency: 'USD',
    event_type: 'started',
  });

  // Notification form
  const [notificationData, setNotificationData] = useState<BroadcastNotificationData>({
    title: 'Test Notification',
    message: 'This is a test notification from the admin panel.',
    level: 'info',
  });

  // Stock simulation form
  const [stockData, setStockData] = useState<SimulateStockData>({
    symbol: 'AAPL',
    price: 150.0,
    change: 2.5,
    change_percent: 1.67,
    volume: 1000000,
  });

  // Simulate payment event
  const simulatePayment = async () => {
    setLoading('payment');
    try {
      const response = await apiClient.post('/realtime/admin/simulate/payment', paymentData);
      
      if (response.error) {
        setResult(`Error: ${response.error}`);
      } else {
        setResult(`Payment ${paymentData.event_type} event simulated successfully. Event ID: ${response.data?.event_id}`);
        // Generate new payment ID for next test
        setPaymentData(prev => ({
          ...prev,
          payment_id: `pay_${Date.now()}`,
        }));
      }
    } catch (error) {
      setResult(`Failed to simulate payment: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  // Broadcast notification
  const broadcastNotification = async () => {
    setLoading('notification');
    try {
      const response = await apiClient.post('/realtime/admin/broadcast', notificationData);
      
      if (response.error) {
        setResult(`Error: ${response.error}`);
      } else {
        setResult(`Notification broadcasted successfully. Event ID: ${response.data?.event_id}`);
      }
    } catch (error) {
      setResult(`Failed to broadcast notification: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  // Simulate stock update
  const simulateStock = async () => {
    setLoading('stock');
    try {
      const response = await apiClient.post('/realtime/admin/simulate/stock', stockData);
      
      if (response.error) {
        setResult(`Error: ${response.error}`);
      } else {
        setResult(`Stock update simulated successfully. Event ID: ${response.data?.event_id}`);
      }
    } catch (error) {
      setResult(`Failed to simulate stock update: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  // Get connection stats
  const getStats = async () => {
    setLoading('stats');
    try {
      const response = await apiClient.get('/realtime/admin/stats');
      
      if (response.error) {
        setResult(`Error: ${response.error}`);
      } else {
        setResult(`Connection Stats:\n${JSON.stringify(response.data, null, 2)}`);
      }
    } catch (error) {
      setResult(`Failed to get stats: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  if (!user || (user.role !== 'admin' && user.role !== 'super_admin')) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Admin Event Tester</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">Admin access required.</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Admin Event Tester</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={getStats} disabled={loading === 'stats'} size="sm">
              {loading === 'stats' ? 'Loading...' : 'Get Connection Stats'}
            </Button>
          </div>
          
          {result && (
            <div className="p-3 bg-muted rounded">
              <pre className="text-sm whitespace-pre-wrap">{result}</pre>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Payment Simulation */}
      <Card>
        <CardHeader>
          <CardTitle>Simulate Payment Event</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="payment-id">Payment ID</Label>
              <Input
                id="payment-id"
                value={paymentData.payment_id}
                onChange={(e) => setPaymentData(prev => ({ ...prev, payment_id: e.target.value }))}
              />
            </div>
            <div>
              <Label htmlFor="user-id">User ID</Label>
              <Input
                id="user-id"
                value={paymentData.user_id}
                onChange={(e) => setPaymentData(prev => ({ ...prev, user_id: e.target.value }))}
              />
            </div>
            <div>
              <Label htmlFor="amount">Amount</Label>
              <Input
                id="amount"
                type="number"
                step="0.01"
                value={paymentData.amount}
                onChange={(e) => setPaymentData(prev => ({ ...prev, amount: parseFloat(e.target.value) || 0 }))}
              />
            </div>
            <div>
              <Label htmlFor="currency">Currency</Label>
              <Input
                id="currency"
                value={paymentData.currency}
                onChange={(e) => setPaymentData(prev => ({ ...prev, currency: e.target.value }))}
              />
            </div>
          </div>
          
          <div>
            <Label htmlFor="event-type">Event Type</Label>
            <Select
              value={paymentData.event_type}
              onValueChange={(value: 'started' | 'completed' | 'failed') => 
                setPaymentData(prev => ({ ...prev, event_type: value }))
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="started">Started</SelectItem>
                <SelectItem value="completed">Completed</SelectItem>
                <SelectItem value="failed">Failed</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {paymentData.event_type === 'completed' && (
            <div>
              <Label htmlFor="transaction-id">Transaction ID (optional)</Label>
              <Input
                id="transaction-id"
                value={paymentData.transaction_id || ''}
                onChange={(e) => setPaymentData(prev => ({ ...prev, transaction_id: e.target.value }))}
              />
            </div>
          )}

          {paymentData.event_type === 'failed' && (
            <>
              <div>
                <Label htmlFor="error-code">Error Code (optional)</Label>
                <Input
                  id="error-code"
                  value={paymentData.error_code || ''}
                  onChange={(e) => setPaymentData(prev => ({ ...prev, error_code: e.target.value }))}
                />
              </div>
              <div>
                <Label htmlFor="error-message">Error Message (optional)</Label>
                <Input
                  id="error-message"
                  value={paymentData.error_message || ''}
                  onChange={(e) => setPaymentData(prev => ({ ...prev, error_message: e.target.value }))}
                />
              </div>
            </>
          )}

          <Button onClick={simulatePayment} disabled={loading === 'payment'}>
            {loading === 'payment' ? 'Simulating...' : 'Simulate Payment'}
          </Button>
        </CardContent>
      </Card>

      {/* Notification Broadcasting */}
      <Card>
        <CardHeader>
          <CardTitle>Broadcast Notification</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="notif-title">Title</Label>
            <Input
              id="notif-title"
              value={notificationData.title}
              onChange={(e) => setNotificationData(prev => ({ ...prev, title: e.target.value }))}
            />
          </div>
          
          <div>
            <Label htmlFor="notif-message">Message</Label>
            <Textarea
              id="notif-message"
              value={notificationData.message}
              onChange={(e) => setNotificationData(prev => ({ ...prev, message: e.target.value }))}
            />
          </div>
          
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="notif-level">Level</Label>
              <Select
                value={notificationData.level}
                onValueChange={(value: 'info' | 'warning' | 'error' | 'success') => 
                  setNotificationData(prev => ({ ...prev, level: value }))
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="info">Info</SelectItem>
                  <SelectItem value="warning">Warning</SelectItem>
                  <SelectItem value="error">Error</SelectItem>
                  <SelectItem value="success">Success</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label htmlFor="target-user">Target User (optional)</Label>
              <Input
                id="target-user"
                value={notificationData.target_user || ''}
                onChange={(e) => setNotificationData(prev => ({ ...prev, target_user: e.target.value || undefined }))}
                placeholder="Leave empty for broadcast"
              />
            </div>
          </div>

          <Button onClick={broadcastNotification} disabled={loading === 'notification'}>
            {loading === 'notification' ? 'Broadcasting...' : 'Broadcast Notification'}
          </Button>
        </CardContent>
      </Card>

      {/* Stock Simulation */}
      <Card>
        <CardHeader>
          <CardTitle>Simulate Stock Update</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="symbol">Symbol</Label>
              <Input
                id="symbol"
                value={stockData.symbol}
                onChange={(e) => setStockData(prev => ({ ...prev, symbol: e.target.value }))}
              />
            </div>
            <div>
              <Label htmlFor="price">Price</Label>
              <Input
                id="price"
                type="number"
                step="0.01"
                value={stockData.price}
                onChange={(e) => setStockData(prev => ({ ...prev, price: parseFloat(e.target.value) || 0 }))}
              />
            </div>
            <div>
              <Label htmlFor="change">Change</Label>
              <Input
                id="change"
                type="number"
                step="0.01"
                value={stockData.change}
                onChange={(e) => setStockData(prev => ({ ...prev, change: parseFloat(e.target.value) || 0 }))}
              />
            </div>
            <div>
              <Label htmlFor="change-percent">Change %</Label>
              <Input
                id="change-percent"
                type="number"
                step="0.01"
                value={stockData.change_percent}
                onChange={(e) => setStockData(prev => ({ ...prev, change_percent: parseFloat(e.target.value) || 0 }))}
              />
            </div>
          </div>
          
          <div>
            <Label htmlFor="volume">Volume</Label>
            <Input
              id="volume"
              type="number"
              value={stockData.volume}
              onChange={(e) => setStockData(prev => ({ ...prev, volume: parseInt(e.target.value) || 0 }))}
            />
          </div>

          <Button onClick={simulateStock} disabled={loading === 'stock'}>
            {loading === 'stock' ? 'Simulating...' : 'Simulate Stock Update'}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}