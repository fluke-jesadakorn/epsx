// Real-time payment tracking component using WebSocket and SSE

'use client';

import { useEffect, useState, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useAuth } from '@/context/auth-context';

interface PaymentEvent {
  type: 'payment_started' | 'payment_completed' | 'payment_failed';
  data: {
    event: {
      PaymentStarted?: {
        payment_id: string;
        user_id: string;
        amount: number;
        currency: string;
        timestamp: string;
      };
      PaymentCompleted?: {
        payment_id: string;
        user_id: string;
        amount: number;
        currency: string;
        transaction_id: string;
        timestamp: string;
      };
      PaymentFailed?: {
        payment_id: string;
        user_id: string;
        amount: number;
        currency: string;
        error_code: string;
        error_message: string;
        timestamp: string;
      };
    };
    metadata: {
      event_id: string;
      timestamp: string;
      source: string;
    };
  };
}

interface PaymentStatus {
  payment_id: string;
  status: 'started' | 'completed' | 'failed';
  amount: number;
  currency: string;
  timestamp: string;
  transaction_id?: string;
  error_message?: string;
}

export default function PaymentTracker() {
  const { user } = useAuth();
  const [payments, setPayments] = useState<PaymentStatus[]>([]);
  const [connectionType, setConnectionType] = useState<'none' | 'websocket' | 'sse'>('none');
  const [connectionStatus, setConnectionStatus] = useState<'disconnected' | 'connecting' | 'connected'>('disconnected');
  const [wsRef, setWsRef] = useState<WebSocket | null>(null);
  const [eventSourceRef, setEventSourceRef] = useState<EventSource | null>(null);

  // WebSocket connection
  const connectWebSocket = useCallback(() => {
    if (!user || connectionType === 'websocket') return;
    
    setConnectionStatus('connecting');
    setConnectionType('websocket');
    
    // Close existing SSE connection
    if (eventSourceRef) {
      eventSourceRef.close();
      setEventSourceRef(null);
    }
    
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/realtime/ws?events=payment,notification`;
    
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
      // WebSocket connected
      setConnectionStatus('connected');
    };
    
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as PaymentEvent;
        handlePaymentEvent(data);
      } catch (error) {
        console.error('Failed to parse WebSocket message', { error: error instanceof Error ? error.message : error, rawData: event.data });
      }
    };
    
    ws.onclose = () => {
      // WebSocket disconnected
      setConnectionStatus('disconnected');
      setWsRef(null);
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error', { error: error instanceof Error ? error.message : error });
      setConnectionStatus('disconnected');
    };
    
    setWsRef(ws);
  }, [user, connectionType, eventSourceRef]);

  // SSE connection
  const connectSSE = useCallback(() => {
    if (!user || connectionType === 'sse') return;
    
    setConnectionStatus('connecting');
    setConnectionType('sse');
    
    // Close existing WebSocket connection
    if (wsRef) {
      wsRef.close();
      setWsRef(null);
    }
    
    const sseUrl = `/realtime/events?events=payment,notification`;
    const eventSource = new EventSource(sseUrl);
    
    eventSource.onopen = () => {
      // SSE connected
      setConnectionStatus('connected');
    };
    
    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as PaymentEvent;
        handlePaymentEvent(data);
      } catch (error) {
        console.error('Failed to parse SSE message', { error: error instanceof Error ? error.message : error, rawData: event.data });
      }
    };
    
    eventSource.onerror = () => {
      console.error('SSE error');
      setConnectionStatus('disconnected');
      setEventSourceRef(null);
    };
    
    // Listen for specific event types
    eventSource.addEventListener('payment_started', (event) => {
      try {
        const data = JSON.parse(event.data) as PaymentEvent;
        handlePaymentEvent(data);
      } catch (error) {
        console.error('Failed to parse payment_started event', { error: error instanceof Error ? error.message : error, rawData: event.data });
      }
    });
    
    eventSource.addEventListener('payment_completed', (event) => {
      try {
        const data = JSON.parse(event.data) as PaymentEvent;
        handlePaymentEvent(data);
      } catch (error) {
        console.error('Failed to parse payment_completed event', { error: error instanceof Error ? error.message : error, rawData: event.data });
      }
    });
    
    eventSource.addEventListener('payment_failed', (event) => {
      try {
        const data = JSON.parse(event.data) as PaymentEvent;
        handlePaymentEvent(data);
      } catch (error) {
        console.error('Failed to parse payment_failed event', { error: error instanceof Error ? error.message : error, rawData: event.data });
      }
    });
    
    setEventSourceRef(eventSource);
  }, [user, connectionType, wsRef]);

  // Handle payment events
  const handlePaymentEvent = useCallback((data: PaymentEvent) => {
    // Payment event received
    
    let paymentStatus: PaymentStatus;
    
    if (data.data.event.PaymentStarted) {
      const payment = data.data.event.PaymentStarted;
      paymentStatus = {
        payment_id: payment.payment_id,
        status: 'started',
        amount: payment.amount,
        currency: payment.currency,
        timestamp: payment.timestamp,
      };
    } else if (data.data.event.PaymentCompleted) {
      const payment = data.data.event.PaymentCompleted;
      paymentStatus = {
        payment_id: payment.payment_id,
        status: 'completed',
        amount: payment.amount,
        currency: payment.currency,
        timestamp: payment.timestamp,
        transaction_id: payment.transaction_id,
      };
    } else if (data.data.event.PaymentFailed) {
      const payment = data.data.event.PaymentFailed;
      paymentStatus = {
        payment_id: payment.payment_id,
        status: 'failed',
        amount: payment.amount,
        currency: payment.currency,
        timestamp: payment.timestamp,
        error_message: payment.error_message,
      };
    } else {
      return; // Unknown event type
    }
    
    setPayments(prev => {
      const existing = prev.find(p => p.payment_id === paymentStatus.payment_id);
      if (existing) {
        // Update existing payment
        return prev.map(p => 
          p.payment_id === paymentStatus.payment_id ? paymentStatus : p
        );
      } else {
        // Add new payment
        return [paymentStatus, ...prev].slice(0, 10); // Keep only last 10
      }
    });
  }, []);

  // Disconnect
  const disconnect = useCallback(() => {
    if (wsRef) {
      wsRef.close();
      setWsRef(null);
    }
    if (eventSourceRef) {
      eventSourceRef.close();
      setEventSourceRef(null);
    }
    setConnectionType('none');
    setConnectionStatus('disconnected');
  }, [wsRef, eventSourceRef]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  // Get status color
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'started': return 'bg-blue-100 text-blue-800';
      case 'completed': return 'bg-green-100 text-green-800';
      case 'failed': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  // Get connection status color
  const getConnectionColor = (status: string) => {
    switch (status) {
      case 'connected': return 'bg-green-100 text-green-800';
      case 'connecting': return 'bg-yellow-100 text-yellow-800';
      case 'disconnected': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  if (!user) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Payment Tracker</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">Please log in to track payments.</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          Payment Tracker
          <div className="flex items-center gap-2">
            <Badge className={getConnectionColor(connectionStatus)}>
              {connectionStatus} ({connectionType})
            </Badge>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Connection Controls */}
        <div className="flex gap-2">
          <Button 
            onClick={connectWebSocket}
            disabled={connectionType === 'websocket'}
            size="sm"
          >
            Connect WebSocket
          </Button>
          <Button 
            onClick={connectSSE}
            disabled={connectionType === 'sse'}
            size="sm"
          >
            Connect SSE
          </Button>
          <Button 
            onClick={disconnect}
            disabled={connectionType === 'none'}
            variant="outline"
            size="sm"
          >
            Disconnect
          </Button>
        </div>

        {/* Payment Events */}
        <div className="space-y-2">
          <h3 className="text-lg font-medium">Recent Payments</h3>
          {payments.length === 0 ? (
            <p className="text-muted-foreground">No payment events yet.</p>
          ) : (
            <div className="space-y-2">
              {payments.map((payment) => (
                <div
                  key={payment.payment_id}
                  className="flex items-center justify-between p-3 border rounded-lg"
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <code className="text-sm bg-muted px-2 py-1 rounded">
                        {payment.payment_id}
                      </code>
                      <Badge className={getStatusColor(payment.status)}>
                        {payment.status}
                      </Badge>
                    </div>
                    <div className="text-sm text-muted-foreground mt-1">
                      {payment.amount} {payment.currency}
                      {payment.transaction_id && (
                        <span className="ml-2">• Transaction: {payment.transaction_id}</span>
                      )}
                      {payment.error_message && (
                        <span className="ml-2 text-red-600">• {payment.error_message}</span>
                      )}
                    </div>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {new Date(payment.timestamp).toLocaleTimeString()}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Debug Info */}
        <details className="text-xs">
          <summary className="cursor-pointer text-muted-foreground">Debug Info</summary>
          <pre className="mt-2 p-2 bg-muted rounded text-xs overflow-x-auto">
            {JSON.stringify({ connectionType, connectionStatus, paymentCount: payments.length }, null, 2)}
          </pre>
        </details>
      </CardContent>
    </Card>
  );
}