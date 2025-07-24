/**
 * Server-sent events utilities for real-time features
 */

export interface SSEConnection {
  close: () => void;
  send: (data: any) => void;
}

export interface SSEMessage {
  type: string;
  data: any;
  timestamp: number;
}

/**
 * Create server-sent events stream for real-time updates
 */
export function createSSEStream(
  onMessage: (message: SSEMessage) => void,
  onError?: (error: Error) => void,
  onOpen?: () => void
): SSEConnection {
  let eventSource: EventSource | null = null;
  let isConnected = false;

  const connect = () => {
    if (typeof window === 'undefined') return;

    const url = new URL('/api/stream', window.location.origin);
    eventSource = new EventSource(url.toString());

    eventSource.onopen = () => {
      isConnected = true;
      onOpen?.();
    };

    eventSource.onmessage = (event) => {
      try {
        const message: SSEMessage = JSON.parse(event.data);
        onMessage(message);
      } catch (error) {
        console.error('Failed to parse SSE message:', error);
        onError?.(error as Error);
      }
    };

    eventSource.onerror = (error) => {
      isConnected = false;
      onError?.(new Error('SSE connection error'));
      
      // Reconnect after 5 seconds
      setTimeout(() => {
        if (!isConnected) {
          connect();
        }
      }, 5000);
    };
  };

  // Initial connection
  connect();

  return {
    close: () => {
      isConnected = false;
      eventSource?.close();
    },
    send: (data: any) => {
      // For bidirectional communication, you'd implement WebSocket
      // For now, use regular HTTP requests
      fetch('/api/stream', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      }).catch(console.error);
    },
  };
}

/**
 * React hook for server-sent events
 */
export function useSSE(
  eventTypes: string[] = [],
  options: {
    onMessage?: (message: SSEMessage) => void;
    onError?: (error: Error) => void;
    onOpen?: () => void;
    autoConnect?: boolean;
  } = {}
) {
  const { onMessage, onError, onOpen, autoConnect = true } = options;
  
  if (typeof window === 'undefined') {
    return {
      connection: null,
      isConnected: false,
      connect: () => {},
      disconnect: () => {},
    };
  }

  let connection: SSEConnection | null = null;
  let isConnected = false;

  const connect = () => {
    if (connection) return;

    connection = createSSEStream(
      (message) => {
        if (eventTypes.length === 0 || eventTypes.includes(message.type)) {
          onMessage?.(message);
        }
      },
      onError,
      () => {
        isConnected = true;
        onOpen?.();
      }
    );
  };

  const disconnect = () => {
    if (connection) {
      connection.close();
      connection = null;
      isConnected = false;
    }
  };

  // Auto-connect on mount
  if (autoConnect && typeof window !== 'undefined') {
    connect();
  }

  // Cleanup on unmount
  if (typeof window !== 'undefined') {
    window.addEventListener('beforeunload', disconnect);
  }

  return {
    connection,
    isConnected,
    connect,
    disconnect,
  };
}

/**
 * Performance monitoring for streaming content
 */
export class StreamingPerformanceMonitor {
  private static instance: StreamingPerformanceMonitor;
  private metrics: Map<string, number[]> = new Map();

  public static getInstance(): StreamingPerformanceMonitor {
    if (!StreamingPerformanceMonitor.instance) {
      StreamingPerformanceMonitor.instance = new StreamingPerformanceMonitor();
    }
    return StreamingPerformanceMonitor.instance;
  }

  startTiming(identifier: string): void {
    if (!this.metrics.has(identifier)) {
      this.metrics.set(identifier, []);
    }
    this.metrics.get(identifier)!.push(performance.now());
  }

  endTiming(identifier: string): number | null {
    const times = this.metrics.get(identifier);
    if (!times || times.length === 0) return null;

    const startTime = times.pop()!;
    const duration = performance.now() - startTime;

    // Log slow streaming components
    if (duration > 1000) {
      console.warn(`Slow streaming component: ${identifier} took ${duration.toFixed(2)}ms`);
    }

    return duration;
  }

  getAverageTime(identifier: string): number | null {
    const times = this.metrics.get(identifier);
    if (!times || times.length === 0) return null;

    const sum = times.reduce((a, b) => a + b, 0);
    return sum / times.length;
  }

  getMetrics(): Record<string, { average: number; count: number }> {
    const result: Record<string, { average: number; count: number }> = {};
    
    for (const [identifier, times] of this.metrics.entries()) {
      if (times.length > 0) {
        const average = times.reduce((a, b) => a + b, 0) / times.length;
        result[identifier] = { average, count: times.length };
      }
    }

    return result;
  }
}